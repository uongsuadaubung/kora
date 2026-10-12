//! System Tray Menu Commands & Helper Window Procedure

use crate::engine::config::InputMethod;
use crate::engine::config_store;
use crate::ui::controller;
use crate::ui::tray::handler::*;
use crate::ui::win32::*;
use crate::ui::{WM_APP_MODE_CHANGED, show_control_panel, update_tray_icon};
use std::sync::OnceLock;

const WM_DESTROY: u32 = 0x0002;
const WM_LBUTTONUP: usize = 0x0202;
const WM_LBUTTONDBLCLK: usize = 0x0203;
const WM_RBUTTONUP: usize = 0x0205;

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterWindowMessageW(lp_string: *const u16) -> u32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn ShellExecuteW(
        hwnd: isize,
        lp_operation: *const u16,
        lp_file: *const u16,
        lp_parameters: *const u16,
        lp_directory: *const u16,
        n_show_cmd: i32,
    ) -> isize;
}

/// Broadcast by Explorer when the taskbar is (re)created; tray icons must be re-added.
fn taskbar_created_msg() -> u32 {
    static MSG: OnceLock<u32> = OnceLock::new();
    *MSG.get_or_init(|| unsafe { RegisterWindowMessageW(to_wide("TaskbarCreated").as_ptr()) })
}

pub fn open_url(url: &str) {
    unsafe {
        ShellExecuteW(
            0,
            to_wide("open").as_ptr(),
            to_wide(url).as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        );
    }
}

/// Toggles V/E, persists, then syncs the tray (after the engine lock is released).
fn toggle_vietnamese() {
    let is_viet = controller::with_engine(|engine| {
        let res = engine.toggle_enabled();
        controller::save(engine);
        res
    });
    if let Some(is_viet) = is_viet {
        update_tray_icon(is_viet);
    }
}

fn set_input_method(method: InputMethod) {
    controller::with_engine(|engine| {
        let cfg = engine.config_mut();
        // "Simple Telex" covers both variants: don't downgrade SimpleTelex2
        if !(method == InputMethod::SimpleTelex1 && cfg.method == InputMethod::SimpleTelex2) {
            cfg.method = method;
        }
        config_store::save_config_and_macros_debounced(engine.config(), &engine.macro_table);
    });
}

pub fn handle_menu_command(cmd: u32) {
    match cmd {
        IDM_TOGGLE_VIET => toggle_vietnamese(),
        IDM_TELEX => set_input_method(InputMethod::Telex),
        IDM_VNI => set_input_method(InputMethod::Vni),
        IDM_SIMPLE_TELEX => set_input_method(InputMethod::SimpleTelex1),
        IDM_CONTROL_PANEL => show_control_panel(),
        IDM_EXIT => unsafe {
            config_store::flush_config_debounced();
            PostQuitMessage(0);
        },
        _ => {}
    }
}

fn show_tray_menu(hwnd: isize) {
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut pt);
    }

    // Snapshot plain values; no lock may be held while TrackPopupMenu pumps messages.
    let (is_viet, method) =
        controller::with_engine(|e| (e.effective_config().enabled, e.config().method))
            .unwrap_or((true, InputMethod::Telex));

    let cmd = show_context_menu(hwnd, pt.x, pt.y, is_viet, method);
    handle_menu_command(cmd);
}

const TIMER_ID_RESUME_SYNC: usize = 9901;

/// Helper window procedure for System Tray events
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn tray_helper_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    match msg {
        WM_TRAY_MESSAGE => {
            match lparam as usize {
                WM_LBUTTONUP => toggle_vietnamese(),
                WM_LBUTTONDBLCLK => show_control_panel(),
                WM_RBUTTONUP => show_tray_menu(hwnd),
                _ => {}
            }
            0
        }
        WM_APP_MODE_CHANGED => {
            // Posted by the keyboard hook after a hotkey toggle
            update_tray_icon(wparam != 0);
            0
        }
        WM_POWERBROADCAST => {
            match wparam {
                PBT_APMSUSPEND => {
                    crate::platform::reset_modifier_states();
                    controller::with_engine(|engine| engine.reset());
                }
                PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
                    crate::platform::reinstall_hooks();
                    unsafe {
                        SetTimer(hwnd, TIMER_ID_RESUME_SYNC, 300, None);
                    }
                }
                _ => {}
            }
            0
        }
        WM_WTSSESSION_CHANGE => {
            match wparam {
                WTS_SESSION_LOCK => {
                    crate::platform::reset_modifier_states();
                    controller::with_engine(|engine| engine.reset());
                }
                WTS_SESSION_UNLOCK | WTS_SESSION_LOGON | WTS_CONSOLE_CONNECT => {
                    crate::platform::reinstall_hooks();
                    unsafe {
                        SetTimer(hwnd, TIMER_ID_RESUME_SYNC, 300, None);
                    }
                }
                _ => {}
            }
            0
        }
        WM_TIMER => {
            if wparam == TIMER_ID_RESUME_SYNC {
                unsafe {
                    KillTimer(hwnd, TIMER_ID_RESUME_SYNC);
                }
                crate::platform::reinstall_hooks();
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                WTSUnRegisterSessionNotification(hwnd);
            }
            config_store::flush_config_debounced();
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ if msg == taskbar_created_msg() => {
            if let Some(ui) = crate::ui::ui() {
                ui.tray.borrow_mut().readd();
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
