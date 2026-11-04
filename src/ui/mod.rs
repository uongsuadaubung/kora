//! Kora Windows UI System
//! Powered by egui Windows 11 Fluent Design & lightweight Win32 Tray integration.

pub mod controller;
pub mod fluent;
pub mod notification;
pub mod theme;
pub mod tray;
pub mod win32;
pub mod wnd_proc;

pub use fluent::show_fluent_control_panel;
pub use theme::resolve_is_dark;
pub use win32::*;

use crate::ui::tray::TrayHandler;
use crate::ui::wnd_proc::tray_helper_wnd_proc;
use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicIsize, Ordering};

thread_local! {
    /// UI state, created once by `init_ui` and kept for the process lifetime.
    static UI: Cell<Option<&'static UiState>> = const { Cell::new(None) };
}

/// The UI state; `None` before `init_ui` finished or when called off the UI thread.
pub fn ui() -> Option<&'static UiState> {
    UI.with(Cell::get)
}

/// Hidden helper window (tray owner). Used as the target for messages posted from the hook.
static HELPER_HWND: AtomicIsize = AtomicIsize::new(0);

/// Retrieves the handle of the helper message window.
pub fn helper_hwnd() -> isize {
    HELPER_HWND.load(Ordering::Relaxed)
}

/// Posted to the helper window when the typing mode toggles (wparam = 1 for Vietnamese).
pub const WM_APP_MODE_CHANGED: u32 = 0x8000 + 120;

/// Posted to the control panel by the update-check worker; lparam = `Box<UpdateStatus>` raw ptr.
pub const WM_APP_UPDATE_CHECKED: u32 = 0x8000 + 121;

/// Posted to the control panel by the soundpack installer worker; lparam = `Box<SoundpackInstallStatus>` raw ptr.
pub const WM_APP_SOUNDPACK_INSTALLED: u32 = 0x8000 + 122;

/// Id of the root ("Mặc định") profile in `ProfileManager`.
pub const GLOBAL_PROFILE_ID: &str = "global";

pub struct UiState {
    pub tray: RefCell<TrayHandler>,
}

/// Mode-change callback for the keyboard hook. It runs inside the hook while ENGINE_INSTANCE
/// is locked, so it must not touch UI state synchronously: just post to the UI message queue.
pub fn post_mode_change(is_vietnamese: bool) {
    let h = HELPER_HWND.load(Ordering::Relaxed);
    if h != 0 {
        unsafe {
            PostMessageW(h, WM_APP_MODE_CHANGED, is_vietnamese as usize, 0);
        }
    }
}

pub fn show_control_panel() {
    crate::ui::fluent::show_fluent_control_panel();
}

/// Hides the control panel and trims the process working set back to ~1-2 MB.
pub fn hide_control_panel() {
    crate::platform::win32::trim_working_set();
}

/// Syncs the tray icon after a mode change.
pub fn update_tray_icon(is_vietnamese: bool) {
    if let Some(ui) = ui() {
        ui.tray.borrow_mut().set_vietnamese_mode(is_vietnamese);
    }
}

/// Initializes the lightweight Win32 UI system (Tray + Helper window)
pub fn init_ui() -> bool {
    let class_helper_name = to_wide("KoraHelperClass");

    let cfg_enabled = crate::platform::win32::ENGINE_INSTANCE
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|e| e.effective_config().enabled))
        .unwrap_or(true);

    unsafe {
        let instance = GetModuleHandleW(null_mut()) as isize;

        // 1. Register Helper Window Class
        let mut wc_helper: WNDCLASSEXW = std::mem::zeroed();
        wc_helper.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc_helper.lpfn_wnd_proc = Some(tray_helper_wnd_proc);
        wc_helper.lpsz_class_name = class_helper_name.as_ptr();
        wc_helper.h_cursor = LoadCursorW(0, IDC_ARROW);
        wc_helper.h_instance = instance;
        RegisterClassExW(&wc_helper);

        // 2. Create Hidden Helper Window under Per-Monitor V2 DPI awareness
        let prev_dpi_ctx = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let h_helper = CreateWindowExW(
            0,
            class_helper_name.as_ptr(),
            to_wide(crate::language::current().helper_window_title).as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            instance,
            null_mut(),
        );
        SetThreadDpiAwarenessContext(prev_dpi_ctx);

        if h_helper == 0 {
            eprintln!("[UI Error] Không thể tạo cửa sổ Helper!");
            return false;
        }

        // Register helper window for Windows session change notifications (lock, unlock, logon)
        WTSRegisterSessionNotification(h_helper, NOTIFY_FOR_THIS_SESSION);

        // 3. Create Tray Handler
        let tray = TrayHandler::new(h_helper, cfg_enabled);

        let state: &'static UiState = Box::leak(Box::new(UiState {
            tray: RefCell::new(tray),
        }));
        UI.with(|cell| cell.set(Some(state)));
        HELPER_HWND.store(h_helper, Ordering::Relaxed);

        true
    }
}
