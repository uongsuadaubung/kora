//! Kora Win32 Window Procedures & Message Handlers

pub use crate::ui_framework::win32::ctlcolor as colors;
pub mod commands;
pub mod tray;

pub use tray::{handle_menu_command, tray_helper_wnd_proc};

use crate::platform::win32::ENGINE_INSTANCE;
use crate::ui::win32::*;
use crate::ui::views::control_panel::paint_control_panel;
use crate::ui::theme::{apply_ui_theme, is_current_dark, resolve_is_dark};

const WM_PAINT: u32 = 0x000F;
const WM_CLOSE: u32 = 0x0010;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SHOWWINDOW: u32 = 0x0018;
const WM_SETTINGCHANGE: u32 = 0x001A;
const WM_SETCURSOR: u32 = 0x0020;
const WM_DRAWITEM: u32 = 0x002B;
const WM_MEASUREITEM: u32 = 0x002C;
const WM_NOTIFY: u32 = 0x004E;
const WM_COMMAND: u32 = 0x0111;
const WM_HSCROLL: u32 = 0x0114;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORLISTBOX: u32 = 0x0134;
const WM_CTLCOLORSTATIC: u32 = 0x0138;

/// Control Panel Dialog window procedure
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn control_panel_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    // 1. Direct Lock-Free Window State Access via thread-local UI with GWLP_USERDATA fallback
    let controls = crate::ui::controls().or_else(|| {
        let controls_ptr = unsafe {
            GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const crate::ui::views::ControlPanelControls
        };
        if !controls_ptr.is_null() {
            unsafe { Some(&*controls_ptr) }
        } else {
            None
        }
    });

    match msg {
        WM_ERASEBKGND => 1,
        WM_SHOWWINDOW => 0,
        WM_PAINT => {
            unsafe {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let mut rc = RECT::default();
                    GetClientRect(hwnd, &mut rc);
                    let width = rc.right - rc.left;
                    let height = rc.bottom - rc.top;
                    let is_dark = is_current_dark();

                    if width > 0 && height > 0 {
                        if let Some(bb) = BackBuffer::new(hdc, width, height) {
                            paint_control_panel(hwnd, bb.hdc(), is_dark);
                        } else {
                            paint_control_panel(hwnd, hdc, is_dark);
                        }
                    } else {
                        paint_control_panel(hwnd, hdc, is_dark);
                    }
                    EndPaint(hwnd, &ps);
                }
            }
            0
        }
        WM_MEASUREITEM => {
            let mis = unsafe { &mut *(lparam as *mut MEASUREITEMSTRUCT) };
            if let Some(c) = controls
                && c.on_measure_item(mis)
            {
                return 1;
            }
            0
        }
        WM_DRAWITEM => {
            let dis = unsafe { &*(lparam as *const DRAWITEMSTRUCT) };
            // Message Reflection: Forward OCM_DRAWITEM directly to child control
            if dis.hwnd_item != 0 {
                let res = unsafe { SendMessageW(dis.hwnd_item, OCM_DRAWITEM, wparam, lparam) };
                if res != 0 {
                    return 1;
                }
            }
            0
        }
        WM_SETTINGCHANGE => {
            // Broadcast for many unrelated reasons (env vars, clock, ...): only react to
            // "ImmersiveColorSet" and only when the resolved dark/light state really changes.
            if lparam != 0 && unsafe { wide_ptr_eq(lparam as *const u16, "ImmersiveColorSet") } {
                let cfg_theme = ENGINE_INSTANCE
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|e| e.config().theme))
                    .unwrap_or(crate::engine::config::UiTheme::Auto);
                if cfg_theme == crate::engine::config::UiTheme::Auto
                    && let Some(c) = controls
                {
                    let is_dark = resolve_is_dark(cfg_theme);
                    if is_dark != is_current_dark() {
                        apply_ui_theme(hwnd, c, is_dark);
                    }
                }
            }
            0
        }
        WM_HSCROLL => {
            if let Some(c) = controls {
                c.on_hscroll(wparam, lparam);
            }
            0
        }
        crate::ui::WM_APP_UPDATE_CHECKED => {
            // lparam owns a Box<UpdateStatus> handed over by the update-check worker thread
            let status = unsafe { Box::from_raw(lparam as *mut crate::updater::UpdateStatus) };
            if let Some(c) = controls {
                c.tab_about.on_update_checked();
            }
            crate::updater::show_update_result_dialog(hwnd, &status);
            0
        }
        crate::ui::WM_APP_SOUNDPACK_INSTALLED => {
            // lparam owns a Box<SoundpackInstallStatus> handed over by the installer worker thread
            let status = unsafe {
                Box::from_raw(
                    lparam
                        as *mut crate::platform::win32::sound::installer::SoundpackInstallStatus,
                )
            };
            if let Some(c) = controls {
                c.tab_sound.on_soundpack_installed(&status);
            }
            crate::platform::win32::sound::installer::show_soundpack_install_dialog(hwnd, &status);
            0
        }
        WM_COMMAND => commands::handle_command(hwnd, wparam, lparam, controls),
        WM_NOTIFY => {
            let nmhdr = unsafe { &*(lparam as *const NMHDR) };
            // Message Reflection: Forward OCM_NOTIFY directly to child component
            if nmhdr.hwnd_from != 0 {
                let res = unsafe { SendMessageW(nmhdr.hwnd_from, OCM_NOTIFY, wparam, lparam) };
                if res != 0 {
                    return res;
                }
            }
            0
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
            colors::handle_ctlcolor_edit_listbox(hwnd, msg, wparam, lparam)
        }
        WM_CTLCOLORSTATIC => colors::handle_ctlcolor_static(hwnd, msg, wparam, lparam),
        WM_SETCURSOR => colors::handle_setcursor(hwnd, msg, wparam, lparam),
        WM_CLOSE => {
            crate::ui::hide_control_panel();
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Compares a NUL-terminated UTF-16 string with an ASCII literal.
unsafe fn wide_ptr_eq(p: *const u16, s: &str) -> bool {
    let mut i = 0usize;
    for b in s.bytes() {
        if unsafe { *p.add(i) } != b as u16 {
            return false;
        }
        i += 1;
    }
    unsafe { *p.add(i) == 0 }
}
