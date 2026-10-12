//! Interactive Key Capture & Modifier State Tracking
//!
//! Quản lý trạng thái bắt phím trực tiếp từ Low-Level Keyboard Hook:
//! - Bắt đơn phím cho hộp thoại Đổi phím (Remap Key Dialog).
//! - Bắt tổ hợp phím cho hộp thoại Phím tắt (Hotkey Dialog).
//! - Theo dõi trạng thái phím bổ trợ (Ctrl, Shift, Alt, Windows) an toàn chống kẹt phím.

use super::sound;
use super::types::*;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;

pub const WM_HOTKEY_CAPTURED: u32 = 0x8000 + 101;
pub const WM_REMAP_KEY_CAPTURED: u32 = 0x8000 + 102;
pub const WM_HOTKEY_CANCELLED: u32 = 0x8000 + 103;

pub(crate) static HOTKEY_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
pub(crate) static HOTKEY_DIALOG_HWND: AtomicIsize = AtomicIsize::new(0);
pub(crate) static CAPTURED_HOTKEY: Mutex<Option<crate::engine::config::Hotkey>> = Mutex::new(None);
pub(crate) static CAPTURE_DISPLAY_TEXT: Mutex<String> = Mutex::new(String::new());
pub(crate) static HOTKEY_MOD_ARMED: AtomicBool = AtomicBool::new(false);

pub(crate) static TRACKED_CTRL: AtomicBool = AtomicBool::new(false);
pub(crate) static TRACKED_SHIFT: AtomicBool = AtomicBool::new(false);
pub(crate) static TRACKED_ALT: AtomicBool = AtomicBool::new(false);
pub(crate) static TRACKED_WIN: AtomicBool = AtomicBool::new(false);

pub(crate) static REMAP_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
pub(crate) static REMAP_DIALOG_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn is_ctrl_vk(vk: u32) -> bool {
    matches!(vk, 0x11 | 0xA2 | 0xA3) // VK_CONTROL, VK_LCONTROL, VK_RCONTROL
}

pub fn is_shift_vk(vk: u32) -> bool {
    matches!(vk, 0x10 | 0xA0 | 0xA1) // VK_SHIFT, VK_LSHIFT, VK_RSHIFT
}

pub fn is_alt_vk(vk: u32) -> bool {
    matches!(vk, 0x12 | 0xA4 | 0xA5) // VK_MENU, VK_LMENU, VK_RMENU
}

pub fn is_win_vk(vk: u32) -> bool {
    matches!(vk, 0x5B | 0x5C) // VK_LWIN, VK_RWIN
}

pub fn is_modifier_vk(vk: u32) -> bool {
    is_ctrl_vk(vk) || is_shift_vk(vk) || is_alt_vk(vk) || is_win_vk(vk)
}

pub fn start_single_key_capture(hwnd: isize) {
    REMAP_DIALOG_HWND.store(hwnd, Ordering::SeqCst);
    REMAP_CAPTURE_ACTIVE.store(true, Ordering::SeqCst);
}

pub fn stop_single_key_capture() {
    REMAP_CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
    REMAP_DIALOG_HWND.store(0, Ordering::SeqCst);
}

pub fn start_hotkey_capture(hwnd: isize) {
    HOTKEY_DIALOG_HWND.store(hwnd, Ordering::SeqCst);
    TRACKED_CTRL.store(false, Ordering::SeqCst);
    TRACKED_SHIFT.store(false, Ordering::SeqCst);
    TRACKED_ALT.store(false, Ordering::SeqCst);
    TRACKED_WIN.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
        guard.clear();
    }
    HOTKEY_CAPTURE_ACTIVE.store(true, Ordering::SeqCst);
}

pub fn stop_hotkey_capture() {
    HOTKEY_CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
    HOTKEY_DIALOG_HWND.store(0, Ordering::SeqCst);
    TRACKED_CTRL.store(false, Ordering::SeqCst);
    TRACKED_SHIFT.store(false, Ordering::SeqCst);
    TRACKED_ALT.store(false, Ordering::SeqCst);
    TRACKED_WIN.store(false, Ordering::SeqCst);
}

/// Resets all internal modifier and hotkey tracking flags to prevent stuck modifiers
/// (e.g. after Win+L workstation lock, sleep, session change, or focus switch).
pub fn reset_modifier_states() {
    TRACKED_CTRL.store(false, Ordering::SeqCst);
    TRACKED_SHIFT.store(false, Ordering::SeqCst);
    TRACKED_ALT.store(false, Ordering::SeqCst);
    TRACKED_WIN.store(false, Ordering::SeqCst);
    HOTKEY_MOD_ARMED.store(false, Ordering::SeqCst);
    sound::reset_key_sound_bits();
}

pub fn get_captured_hotkey() -> Option<crate::engine::config::Hotkey> {
    if let Ok(guard) = CAPTURED_HOTKEY.lock() {
        *guard
    } else {
        None
    }
}

pub fn get_captured_display_text() -> String {
    if let Ok(guard) = CAPTURE_DISPLAY_TEXT.lock() {
        guard.clone()
    } else {
        String::new()
    }
}

pub fn set_captured_hotkey(hotkey: Option<crate::engine::config::Hotkey>, display: &str) {
    if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
        *guard = hotkey;
    }
    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
        *guard = display.to_string();
    }
}

/// Xử lý chế độ bắt đơn phím từ Low-Level Keyboard Hook
pub unsafe fn handle_remap_capture(vk: u32, w_param: usize) -> Option<isize> {
    if REMAP_CAPTURE_ACTIVE.load(Ordering::SeqCst) {
        let dialog_hwnd = REMAP_DIALOG_HWND.load(Ordering::SeqCst);
        if dialog_hwnd != 0 && unsafe { GetForegroundWindow() } == dialog_hwnd {
            if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
                unsafe {
                    PostMessageW(dialog_hwnd, WM_REMAP_KEY_CAPTURED, vk as usize, 0);
                }
            }
            return Some(1);
        }
    }
    None
}

/// Xử lý chế độ bắt phím tắt Hotkey từ Low-Level Keyboard Hook
pub unsafe fn handle_hotkey_capture(vk: u32, w_param: usize) -> Option<isize> {
    if HOTKEY_CAPTURE_ACTIVE.load(Ordering::SeqCst)
        && HOTKEY_DIALOG_HWND.load(Ordering::SeqCst) == unsafe { GetForegroundWindow() }
    {
        if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
            if vk == 0x1B {
                stop_hotkey_capture();
                let dialog_hwnd = HOTKEY_DIALOG_HWND.load(Ordering::SeqCst);
                if dialog_hwnd != 0 {
                    unsafe {
                        PostMessageW(dialog_hwnd, WM_HOTKEY_CANCELLED, 0, 0);
                    }
                }
                return Some(1);
            }
            if is_ctrl_vk(vk) {
                TRACKED_CTRL.store(true, Ordering::SeqCst);
            }
            if is_shift_vk(vk) {
                TRACKED_SHIFT.store(true, Ordering::SeqCst);
            }
            if is_alt_vk(vk) {
                TRACKED_ALT.store(true, Ordering::SeqCst);
            }
            if is_win_vk(vk) {
                TRACKED_WIN.store(true, Ordering::SeqCst);
            }

            let async_shift = unsafe { (GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0 };
            let async_ctrl = unsafe { (GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0 };
            let async_alt = unsafe { (GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0 };
            let async_win = unsafe {
                ((GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) != 0)
                    || ((GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) != 0)
            };

            if !async_shift && !is_shift_vk(vk) {
                TRACKED_SHIFT.store(false, Ordering::SeqCst);
            }
            if !async_ctrl && !is_ctrl_vk(vk) {
                TRACKED_CTRL.store(false, Ordering::SeqCst);
            }
            if !async_alt && !is_alt_vk(vk) {
                TRACKED_ALT.store(false, Ordering::SeqCst);
            }
            if !async_win && !is_win_vk(vk) {
                TRACKED_WIN.store(false, Ordering::SeqCst);
            }

            let is_shift = is_shift_vk(vk) || async_shift;
            let is_ctrl = is_ctrl_vk(vk) || async_ctrl;
            let is_alt = is_alt_vk(vk) || async_alt;
            let is_win = is_win_vk(vk) || async_win;

            let is_mod = is_modifier_vk(vk);
            let captured = crate::engine::config::Hotkey {
                ctrl: is_ctrl,
                shift: is_shift,
                alt: is_alt,
                win: is_win,
                vk: if is_mod { 0 } else { vk },
            };

            let display = if captured.is_valid() {
                if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
                    *guard = Some(captured);
                }
                captured.display_text()
            } else if is_mod {
                let mut parts = Vec::new();
                if is_ctrl {
                    parts.push("Ctrl");
                }
                if is_alt {
                    parts.push("Alt");
                }
                if is_shift {
                    parts.push("Shift");
                }
                if is_win {
                    parts.push("Win");
                }
                format!("{} + ...", parts.join(" + "))
            } else {
                let key_name = crate::engine::config::vk_to_name(vk);
                format!("{} (Cần thêm Ctrl/Alt/Shift)", key_name)
            };

            if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
                *guard = display;
            }
            let dialog_hwnd = HOTKEY_DIALOG_HWND.load(Ordering::SeqCst);
            if dialog_hwnd != 0 {
                unsafe {
                    PostMessageW(dialog_hwnd, WM_HOTKEY_CAPTURED, 0, 0);
                }
            }
        } else if w_param == WM_KEYUP || w_param == WM_SYSKEYUP {
            unsafe {
                update_modifier_tracking_keyup(vk);
            }
        }
        return Some(1);
    }
    None
}

/// Cập nhật nhả phím cho các cờ theo dõi phím bổ trợ
pub unsafe fn update_modifier_tracking_keyup(vk: u32) {
    if is_ctrl_vk(vk) || unsafe { (GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) == 0 } {
        TRACKED_CTRL.store(false, Ordering::SeqCst);
    }
    if is_shift_vk(vk) || unsafe { (GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) == 0 } {
        TRACKED_SHIFT.store(false, Ordering::SeqCst);
    }
    if is_alt_vk(vk) || unsafe { (GetAsyncKeyState(VK_MENU) as u16 & 0x8000) == 0 } {
        TRACKED_ALT.store(false, Ordering::SeqCst);
    }
    if is_win_vk(vk)
        || unsafe {
            ((GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) == 0)
                && ((GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) == 0)
        }
    {
        TRACKED_WIN.store(false, Ordering::SeqCst);
    }
}
