//! Modal Dialog Framework Scaffold, Message Reflection & Shared GDI Painters
//!
//! Cung cấp toàn bộ cơ chế nền tảng cho các hộp thoại Modal Win32:
//! - Quản lý vòng đời cửa sổ và state (`open_modal`, `open_client_modal`, `attach_state`, `state`, `detach_state`, `close_modal`).
//! - Phản xạ thông điệp tự động cho các component con (`reflect_dialog_messages`, `forward_draw_item`).
//! - Điều hướng bàn phím chuẩn Dialog (`pre_translate_message`, `resolve_enter_command`).
//! - Các hàm vẽ GDI và nút Footer chuẩn hóa (`create_dialog_footer_buttons`, `fill_dialog_bg`, `draw_dialog_callout`, `draw_dialog_label`, `draw_dialog_footer`, `draw_key_box`).

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::components::button::{ButtonVariant, PushButton};
use crate::ui_framework::theme::{
    cached_font, get_app_font_bold, get_app_font_normal, is_current_dark,
};
use crate::ui_framework::tokens::{borders, radii};
use crate::ui_framework::win32::ctlcolor::{
    handle_ctlcolor_edit_listbox, handle_ctlcolor_static, handle_setcursor,
};
use crate::ui_framework::win32::*;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

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
            if !STATE_ATTACHED.swap(false, Ordering::SeqCst) {
                drop(Box::from_raw(raw));
            }
            return 0;
        }
        STATE_ATTACHED.store(false, Ordering::SeqCst);
        apply_modern_window_styling(hwnd, is_current_dark());
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        hwnd
    }
}

static STATE_ATTACHED: AtomicBool = AtomicBool::new(false);

/// `WM_CREATE`: moves the state pointer passed to [`open_modal`] into `GWLP_USERDATA`.
///
/// # Safety
/// `lparam` must be the `CREATESTRUCTW*` of a window created by [`open_modal::<T>`].
pub unsafe fn attach_state<T>(hwnd: isize, lparam: isize) {
    unsafe {
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
/// so components like [`crate::ui_framework::components::PushButton`] paint themselves.
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

/// Shared message reflector for all modal dialogs.
///
/// Handles `OCM_DRAWITEM`, `OCM_NOTIFY`, `OCM_COMMAND` reflection, plus `Label` / `TextBox` /
/// `ListBox` theme coloring (`WM_CTLCOLOR*`) and link cursor (`WM_SETCURSOR`).
///
/// # Safety
/// Must be called from a Win32 window procedure with valid `hwnd`, `msg`, `wparam`, `lparam`.
pub unsafe fn reflect_dialog_messages(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> Option<isize> {
    const WM_SETCURSOR: u32 = 0x0020;
    const WM_DRAWITEM: u32 = 0x002B;
    const WM_NOTIFY: u32 = 0x004E;
    const WM_COMMAND: u32 = 0x0111;
    const WM_CTLCOLOREDIT: u32 = 0x0133;
    const WM_CTLCOLORLISTBOX: u32 = 0x0134;
    const WM_CTLCOLORSTATIC: u32 = 0x0138;

    unsafe {
        match msg {
            WM_DRAWITEM => Some(forward_draw_item(wparam, lparam)),
            WM_NOTIFY => {
                let nmhdr = &*(lparam as *const NMHDR);
                if nmhdr.hwnd_from != 0 {
                    let res = SendMessageW(nmhdr.hwnd_from, OCM_NOTIFY, wparam, lparam);
                    if res != 0 {
                        return Some(res);
                    }
                }
                Some(0)
            }
            WM_COMMAND => {
                if lparam != 0 {
                    let res = SendMessageW(lparam, OCM_COMMAND, wparam, lparam);
                    if res != 0 {
                        return Some(0);
                    }
                }
                None
            }
            WM_CTLCOLORSTATIC => Some(handle_ctlcolor_static(hwnd, msg, wparam, lparam)),
            WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                Some(handle_ctlcolor_edit_listbox(hwnd, msg, wparam, lparam))
            }
            WM_SETCURSOR => Some(handle_setcursor(hwnd, msg, wparam, lparam)),
            _ => None,
        }
    }
}

const DIALOG_CLASS_SUFFIX: &str = "DialogClass";

static LAST_ACTIVE: AtomicIsize = AtomicIsize::new(0);
static LAST_ACTIVE_IS_DIALOG: AtomicBool = AtomicBool::new(false);

fn is_dialog_window(hwnd: isize) -> bool {
    if LAST_ACTIVE.load(Ordering::Relaxed) == hwnd {
        return LAST_ACTIVE_IS_DIALOG.load(Ordering::Relaxed);
    }
    let mut buf = [0u16; 64];
    let len = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    let is_dialog =
        len > 0 && String::from_utf16_lossy(&buf[..len as usize]).ends_with(DIALOG_CLASS_SUFFIX);
    LAST_ACTIVE.store(hwnd, Ordering::Relaxed);
    LAST_ACTIVE_IS_DIALOG.store(is_dialog, Ordering::Relaxed);
    is_dialog
}

/// Message-loop pre-translation: gives the active dialog standard keyboard
/// navigation (Tab / Shift+Tab, Enter → `IDOK`, Esc → `IDCANCEL`).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn pre_translate_message(msg: *const std::ffi::c_void) -> bool {
    unsafe {
        let active = GetActiveWindow();
        active != 0 && is_dialog_window(active) && IsDialogMessageW(active, msg) != 0
    }
}

/// Resolves the `IDOK` that `IsDialogMessageW` sends on Enter.
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
        if len <= 0
            || !String::from_utf16_lossy(&buf[..len as usize]).eq_ignore_ascii_case("button")
        {
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

/// Lấy font Segoe UI Bold cỡ lớn (cached) dùng cho các ô hiển thị phím bấm trong Dialog
pub fn create_display_font() -> isize {
    cached_font(14, FW_BOLD)
}

/// Khởi tạo cửa sổ hộp thoại Modal chuẩn Win32 với kích thước vùng Client (`client_width x client_height`)
/// và gắn `state: Box<T>` vào cửa sổ để lấy ra qua [`attach_state`] / [`state`] / [`detach_state`].
#[allow(clippy::too_many_arguments)]
pub fn open_client_modal<T>(
    parent: isize,
    title: &str,
    client_width: i32,
    client_height: i32,
    class_name: &str,
    wnd_proc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    registered: &std::sync::Once,
    state: Box<T>,
) -> isize {
    register_class_once(registered, class_name, wnd_proc, 0);
    let style = 0x00080000 | 0x10000000 | 0x00C00000 | 0x02000000; // WS_SYSMENU | WS_VISIBLE | WS_CAPTION | WS_CLIPCHILDREN
    let ex_style = 0x00010000 | 0x00000001; // WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME
    let mut rc = RECT {
        left: 0,
        top: 0,
        right: client_width,
        bottom: client_height,
    };
    unsafe {
        AdjustWindowRect(&mut rc, style, 0);
    }
    open_modal(
        parent,
        &ModalSpec {
            class_name,
            title,
            width: rc.right - rc.left,
            height: rc.bottom - rc.top,
            ex_style,
            style,
        },
        state,
    )
}

/// Tạo cặp nút Hủy (`ButtonVariant::Secondary`, tự đóng hộp thoại khi bấm) và Lưu/Áp dụng ở đáy Dialog.
pub fn create_dialog_footer_buttons<F>(
    hwnd: isize,
    (cancel_text, cancel_x, cancel_w): (&str, i32, i32),
    (submit_text, submit_x, submit_w, submit_variant): (&str, i32, i32, ButtonVariant),
    y: i32,
    h: i32,
    bg_erase: Option<u32>,
    on_submit: F,
) where
    F: Fn() + Send + Sync + 'static,
{
    if let Some(btn) = PushButton::create_with_variant(
        hwnd,
        0,
        cancel_text,
        cancel_x,
        y,
        cancel_w,
        h,
        get_app_font_normal(),
        ButtonVariant::Secondary,
        bg_erase,
    ) {
        btn.set_on_click(move || unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        });
    }
    if let Some(btn) = PushButton::create_with_variant(
        hwnd,
        0,
        submit_text,
        submit_x,
        y,
        submit_w,
        h,
        get_app_font_bold(),
        submit_variant,
        bg_erase,
    ) {
        btn.set_on_click(on_submit);
    }
}

/// Tô màu nền cửa sổ hộp thoại (`palette.bg_window`), bật `TRANSPARENT` cho text và trả về `client_rc`.
pub fn fill_dialog_bg(hwnd: isize, hdc: isize, palette: &ThemePalette) -> RECT {
    unsafe {
        let mut rc_client: RECT = std::mem::zeroed();
        GetClientRect(hwnd, &mut rc_client);
        fill_rect_color(hdc, &rc_client, palette.bg_window);
        SetBkMode(hdc, TRANSPARENT);
        rc_client
    }
}

/// Vẽ khung hướng dẫn bo góc (Callout tip box) ở đầu Dialog
pub fn draw_dialog_callout(hdc: isize, palette: &ThemePalette, rc: RECT, text: &str) {
    unsafe {
        let brush_card = CreateSolidBrush(palette.bg_card);
        let pen_card = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
        let prev_brush = SelectObject(hdc, brush_card);
        let prev_pen = SelectObject(hdc, pen_card);
        RoundRect(
            hdc,
            rc.left,
            rc.top,
            rc.right,
            rc.bottom,
            radii::RADIUS_MD,
            radii::RADIUS_MD,
        );
        SelectObject(hdc, prev_brush);
        SelectObject(hdc, prev_pen);
        DeleteObject(brush_card);
        DeleteObject(pen_card);

        let mut rc_text = RECT {
            left: rc.left + 10,
            top: rc.top + 2,
            right: rc.right - 10,
            bottom: rc.bottom - 2,
        };
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, palette.text_secondary);
        let prev_font = SelectObject(hdc, get_app_font_normal());
        let wide = to_wide(text);
        DrawTextW(
            hdc,
            wide.as_ptr(),
            -1,
            &mut rc_text,
            DT_WORDBREAK | DT_VCENTER,
        );
        SelectObject(hdc, prev_font);
    }
}

/// Vẽ thanh Footer chuẩn ở đáy Dialog (nền `bg_sidebar`, đường kẻ ngang phân cách, và dòng chữ bên trái nếu có)
pub fn draw_dialog_footer(
    hdc: isize,
    palette: &ThemePalette,
    client_w: i32,
    footer_y: i32,
    client_h: i32,
    footer_text: Option<&str>,
) {
    let rc_footer = RECT {
        left: 0,
        top: footer_y,
        right: client_w,
        bottom: client_h,
    };
    fill_rect_color(hdc, &rc_footer, palette.bg_sidebar);
    draw_line(
        hdc,
        0,
        footer_y,
        client_w,
        footer_y,
        palette.border_separator,
    );

    if let Some(text) = footer_text {
        unsafe {
            let mut rc = RECT {
                left: 20,
                top: footer_y + 16,
                right: 250,
                bottom: footer_y + 48,
            };
            let prev_font = SelectObject(hdc, get_app_font_normal());
            SetTextColor(hdc, palette.text_secondary);
            let wide = to_wide(text);
            DrawTextW(
                hdc,
                wide.as_ptr(),
                -1,
                &mut rc,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
            );
            SelectObject(hdc, prev_font);
        }
    }
}

/// Vẽ ô hiển thị phím bấm tương tác chuẩn hóa (KeyBox) dùng chung cho các Dialogs
pub fn draw_key_box(
    dis: &DRAWITEMSTRUCT,
    is_dark: bool,
    is_listening: bool,
    text: &str,
    is_placeholder: bool,
    is_danger: bool,
    font: isize,
) {
    let palette = ThemePalette::get(is_dark);
    unsafe {
        let br_bg = CreateSolidBrush(palette.bg_window);
        FillRect(dis.hdc, &dis.rc_item, br_bg);
        DeleteObject(br_bg);

        let (bg_color, border_color) = if is_danger {
            let bg = if is_listening {
                palette.danger_bg
            } else {
                palette.bg_input
            };
            (bg, palette.danger_border)
        } else {
            let bg = if is_listening {
                if is_dark { 0x003A2810 } else { 0x00F8EFEA }
            } else {
                palette.bg_input
            };
            (bg, palette.accent)
        };

        let brush = CreateSolidBrush(bg_color);
        let pen = CreatePen(PS_SOLID, borders::WIDTH_FOCUS, border_color);
        let prev_brush = SelectObject(dis.hdc, brush);
        let prev_pen = SelectObject(dis.hdc, pen);

        RoundRect(
            dis.hdc,
            dis.rc_item.left,
            dis.rc_item.top,
            dis.rc_item.right,
            dis.rc_item.bottom,
            radii::RADIUS_MD,
            radii::RADIUS_MD,
        );

        SelectObject(dis.hdc, prev_brush);
        SelectObject(dis.hdc, prev_pen);
        DeleteObject(brush);
        DeleteObject(pen);

        SetBkMode(dis.hdc, TRANSPARENT);
        let text_color = if is_placeholder {
            palette.text_secondary
        } else if is_danger {
            palette.danger_border
        } else {
            palette.accent
        };
        SetTextColor(dis.hdc, text_color);

        let prev_font = if font != 0 {
            SelectObject(dis.hdc, font)
        } else {
            0
        };

        let wide_text = to_wide(text);
        let mut rc = dis.rc_item;
        DrawTextW(
            dis.hdc,
            wide_text.as_ptr(),
            -1,
            &mut rc,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        if prev_font != 0 {
            SelectObject(dis.hdc, prev_font);
        }
    }
}
