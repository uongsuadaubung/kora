//! Modal Dialog Views for Kora UI

pub mod add_profile;
pub mod base;
pub mod confirm;
pub mod disable_key;
pub mod hotkey;
pub mod macro_editor;
pub mod remap_key;

pub use add_profile::show_add_profile_dialog;
pub use base::*;
pub use confirm::{show_confirm_dialog, ConfirmDialogOptions};
pub use disable_key::show_disable_dialog;
pub use hotkey::show_hotkey_dialog;
pub use macro_editor::{show_macro_dialog, MacroEditData};
pub use remap_key::{show_remap_dialog, show_remap_dialog_with_keys};

use crate::ui::win32::*;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

// ---------------------------------------------------------------------------
// Modal scaffold shared by all dialogs
// ---------------------------------------------------------------------------

/// Window geometry/styles for [`open_modal`].
pub struct ModalSpec<'a> {
    pub class_name: &'a str,
    pub title: &'a str,
    pub width: i32,
    pub height: i32,
    pub ex_style: u32,
    pub style: u32,
}

/// Creates a dialog window centered over `parent`, disables `parent` (modal
/// behavior without a nested message loop) and hands `state` to the window: it is
/// stored in `GWLP_USERDATA` by [`attach_state`] in `WM_CREATE` and must be
/// released with [`detach_state`] in `WM_NCDESTROY`.
/// Returns the dialog HWND, or 0 on failure (parent re-enabled, state dropped).
pub fn open_modal<T>(parent: isize, spec: &ModalSpec, state: Box<T>) -> isize {
    unsafe {
        let mut rc = RECT::default();
        GetWindowRect(parent, &mut rc);
        let x = rc.left + ((rc.right - rc.left) - spec.width) / 2;
        let y = rc.top + ((rc.bottom - rc.top) - spec.height) / 2;

        let raw = Box::into_raw(state);
        EnableWindow(parent, 0);
        let class = to_wide(spec.class_name);
        let title = to_wide(spec.title);
        let hwnd = CreateWindowExW(
            spec.ex_style,
            class.as_ptr(),
            title.as_ptr(),
            spec.style,
            x,
            y,
            spec.width,
            spec.height,
            parent,
            0,
            0,
            raw as *mut std::ffi::c_void,
        );
        if hwnd == 0 {
            EnableWindow(parent, 1);
            // WM_NCDESTROY may already have reclaimed the state if WM_CREATE ran and
            // then failed; only free it when it was never attached.
            if !STATE_ATTACHED.swap(false, Ordering::SeqCst) {
                drop(Box::from_raw(raw));
            }
            return 0;
        }
        STATE_ATTACHED.store(false, Ordering::SeqCst);
        apply_modern_window_styling(hwnd, crate::ui::theme::is_current_dark());
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        hwnd
    }
}

/// Set while a state pointer created by [`open_modal`] is owned by a window.
static STATE_ATTACHED: AtomicBool = AtomicBool::new(false);

/// `WM_CREATE`: moves the state pointer passed to [`open_modal`] into `GWLP_USERDATA`.
///
/// # Safety
/// `lparam` must be the `CREATESTRUCTW*` of a window created by [`open_modal::<T>`].
pub unsafe fn attach_state<T>(hwnd: isize, lparam: isize) {
    unsafe {
        // CREATESTRUCTW.lpCreateParams is the first field.
        let ptr = *(lparam as *const *mut T);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);
    }
    STATE_ATTACHED.store(true, Ordering::SeqCst);
}

/// Borrows the window's state (None before `WM_CREATE` / after `WM_NCDESTROY`).
///
/// # Safety
/// `hwnd` must belong to a dialog created with [`open_modal::<T>`] (same `T`), and
/// the returned reference must not outlive the current message.
pub unsafe fn state<'a, T>(hwnd: isize) -> Option<&'a mut T> {
    unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut T).as_mut() }
}

/// `WM_NCDESTROY`: takes back ownership of the state (dropped by the caller).
///
/// # Safety
/// Same as [`state`].
pub unsafe fn detach_state<T>(hwnd: isize) -> Option<Box<T>> {
    unsafe {
        let ptr = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut T;
        (!ptr.is_null()).then(|| Box::from_raw(ptr))
    }
}

/// Closes a modal dialog: re-enables and activates the owner *before* destroying
/// the dialog, so Windows hands activation back to it instead of another app.
pub fn close_modal(hwnd: isize, parent: isize) {
    unsafe {
        if parent != 0 {
            EnableWindow(parent, 1);
            SetForegroundWindow(parent);
        }
        DestroyWindow(hwnd);
    }
}

/// `WM_DRAWITEM`: reflects owner-draw requests back to the control (`OCM_DRAWITEM`)
/// so components like [`crate::ui::components::button::PushButton`] paint themselves.
///
/// # Safety
/// `lparam` must be the `DRAWITEMSTRUCT*` received with `WM_DRAWITEM`.
pub unsafe fn forward_draw_item(wparam: usize, lparam: isize) -> isize {
    unsafe {
        let dis = &*(lparam as *const DRAWITEMSTRUCT);
        if dis.hwnd_item != 0 {
            SendMessageW(dis.hwnd_item, OCM_DRAWITEM, wparam, lparam)
        } else {
            0
        }
    }
}

/// All dialog window classes end with this suffix (`Kora*DialogClass`).
const DIALOG_CLASS_SUFFIX: &str = "DialogClass";

static LAST_ACTIVE: AtomicIsize = AtomicIsize::new(0);
static LAST_ACTIVE_IS_DIALOG: AtomicBool = AtomicBool::new(false);

fn is_dialog_window(hwnd: isize) -> bool {
    if LAST_ACTIVE.load(Ordering::Relaxed) == hwnd {
        return LAST_ACTIVE_IS_DIALOG.load(Ordering::Relaxed);
    }
    let mut buf = [0u16; 64];
    let len = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    let is_dialog = len > 0
        && String::from_utf16_lossy(&buf[..len as usize]).ends_with(DIALOG_CLASS_SUFFIX);
    LAST_ACTIVE.store(hwnd, Ordering::Relaxed);
    LAST_ACTIVE_IS_DIALOG.store(is_dialog, Ordering::Relaxed);
    is_dialog
}

/// Message-loop pre-translation: gives the active Kora dialog standard keyboard
/// navigation (Tab / Shift+Tab, Enter → `IDOK`, Esc → `IDCANCEL`).
/// Returns `true` when the message was consumed.
///
/// # Safety
/// `msg` must be a valid pointer to a Win32 `MSG` structure from `GetMessageW`.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn pre_translate_message(msg: *const std::ffi::c_void) -> bool {
    unsafe {
        let active = GetActiveWindow();
        active != 0 && is_dialog_window(active) && IsDialogMessageW(active, msg) != 0
    }
}

/// Resolves the `IDOK` that `IsDialogMessageW` sends on Enter: if a push button
/// of the dialog has focus, Enter activates *that* button (owner-drawn buttons
/// don't report `DLGC_DEFPUSHBUTTON`, so Windows would otherwise always send
/// `IDOK`). Returns `default_id` when focus is elsewhere (e.g. in an edit box).
pub fn resolve_enter_command(hwnd_dlg: isize, default_id: u32) -> u32 {
    const BS_TYPEMASK: i32 = 0x0F;
    const BS_PUSHBUTTON: i32 = 0x00;
    const BS_DEFPUSHBUTTON: i32 = 0x01;
    const BS_OWNERDRAW: i32 = 0x0B;
    unsafe {
        let focus = GetFocus();
        if focus == 0 || GetParent(focus) != hwnd_dlg {
            return default_id;
        }
        let mut buf = [0u16; 16];
        let len = GetClassNameW(focus, buf.as_mut_ptr(), buf.len() as i32);
        if len <= 0 || !String::from_utf16_lossy(&buf[..len as usize]).eq_ignore_ascii_case("button") {
            return default_id;
        }
        let kind = GetWindowLongW(focus, GWL_STYLE) & BS_TYPEMASK;
        if matches!(kind, BS_PUSHBUTTON | BS_DEFPUSHBUTTON | BS_OWNERDRAW) {
            GetDlgCtrlID(focus) as u32
        } else {
            default_id
        }
    }
}
