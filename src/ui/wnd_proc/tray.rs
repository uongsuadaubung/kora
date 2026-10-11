//! System Tray Menu Commands & Helper Window Procedure

use crate::engine::config::InputMethod;
use crate::engine::{EngineConfig, config_store};
use crate::platform::win32::ENGINE_INSTANCE;
use crate::ui::win32::*;
use crate::ui::tray::handler::*;
use crate::ui::{UI_MANAGER, show_control_panel, update_tray_icon};

const WM_DESTROY: u32 = 0x0002;
const WM_SETTINGCHANGE: u32 = 0x001A;
const WM_LBUTTONUP: usize = 0x0202;
const WM_LBUTTONDBLCLK: usize = 0x0203;
const WM_RBUTTONUP: usize = 0x0205;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetCursorPos(lp_point: *mut POINT) -> i32;
    fn PostQuitMessage(n_exit_code: i32);
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

pub fn handle_menu_command(cmd: u32) {
    match cmd {
        IDM_TOGGLE_VIET => {
            let is_viet = {
                if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                    && let Some(ref mut engine) = *guard
                {
                    let res = engine.toggle_enabled();
                    config_store::save_config_and_macros_debounced(
                        engine.config(),
                        &engine.macro_table,
                    );
                    res
                } else {
                    false
                }
            };
            update_tray_icon(is_viet);
        }
        IDM_TELEX => {
            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                engine.config_mut().method = InputMethod::Telex;
                config_store::save_config_and_macros_debounced(
                    engine.config(),
                    &engine.macro_table,
                );
            }
        }
        IDM_VNI => {
            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                engine.config_mut().method = InputMethod::Vni;
                config_store::save_config_and_macros_debounced(
                    engine.config(),
                    &engine.macro_table,
                );
            }
        }
        IDM_SIMPLE_TELEX => {
            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                engine.config_mut().method = InputMethod::SimpleTelex1;
                config_store::save_config_and_macros_debounced(
                    engine.config(),
                    &engine.macro_table,
                );
            }
        }
        IDM_CONTROL_PANEL => {
            show_control_panel();
        }
        IDM_EXIT => unsafe {
            config_store::flush_config_debounced();
            PostQuitMessage(0);
        },
        _ => {}
    }
}

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
            let event = lparam as usize;
            if event == WM_LBUTTONUP {
                // Single click -> Toggle Vietnamese / English
                let is_viet = {
                    if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        let res = engine.toggle_enabled();
                        config_store::save_config_and_macros_debounced(
                            engine.config(),
                            &engine.macro_table,
                        );
                        res
                    } else {
                        false
                    }
                };
                update_tray_icon(is_viet);
            } else if event == WM_LBUTTONDBLCLK {
                // Double click -> Show Control Panel
                show_control_panel();
            } else if event == WM_RBUTTONUP {
                // Right click -> Context Menu
                let mut pt = POINT { x: 0, y: 0 };
                unsafe {
                    GetCursorPos(&mut pt);
                }

                let config = {
                    if let Ok(guard) = ENGINE_INSTANCE.lock()
                        && let Some(ref engine) = *guard
                    {
                        engine.config().clone()
                    } else {
                        EngineConfig::default()
                    }
                };

                let cmd = if let Ok(mut ui_guard) = UI_MANAGER.lock() {
                    if let Some(ref mut ui) = *ui_guard {
                        ui.tray.show_context_menu(hwnd, pt.x, pt.y, &config)
                    } else {
                        0
                    }
                } else {
                    0
                };

                handle_menu_command(cmd);
            }
            0
        }
        WM_SETTINGCHANGE => {
            // Windows Theme changed (Light <-> Dark mode) -> auto update tray icon
            if let Ok(mut ui_guard) = UI_MANAGER.lock()
                && let Some(ref mut ui) = *ui_guard
            {
                ui.tray.refresh_icon();
            }
            0
        }
        WM_DESTROY => {
            config_store::flush_config_debounced();
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
