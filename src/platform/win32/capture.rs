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

pub const WM_COMBO_CAPTURED: u32 = 0x8000 + 101;
pub const WM_SINGLE_KEY_CAPTURED: u32 = 0x8000 + 102;
pub const WM_COMBO_CANCELLED: u32 = 0x8000 + 103;

pub(crate) static COMBO_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
pub(crate) static COMBO_DIALOG_HWND: AtomicIsize = AtomicIsize::new(0);
pub(crate) static CAPTURED_HOTKEY: Mutex<Option<crate::engine::config::Hotkey>> = Mutex::new(None);
pub(crate) static CAPTURE_DISPLAY_TEXT: Mutex<String> = Mutex::new(String::new());
pub(crate) static HOTKEY_MOD_ARMED: AtomicBool = AtomicBool::new(false);
pub(crate) static HOTKEY_ALLOW_SINGLE_KEY: AtomicBool = AtomicBool::new(false);

// Global runtime modifier tracking flags
pub(crate) static TRACKED_CTRL: AtomicBool = AtomicBool::new(false);
pub(crate) static TRACKED_SHIFT: AtomicBool = AtomicBool::new(false);
pub(crate) static TRACKED_ALT: AtomicBool = AtomicBool::new(false);
pub(crate) static TRACKED_WIN: AtomicBool = AtomicBool::new(false);

// Dedicated capture tracking flags (independent from runtime hooks, prevents losing held modifiers)
pub(crate) static CAPTURE_CTRL: AtomicBool = AtomicBool::new(false);
pub(crate) static CAPTURE_SHIFT: AtomicBool = AtomicBool::new(false);
pub(crate) static CAPTURE_ALT: AtomicBool = AtomicBool::new(false);
pub(crate) static CAPTURE_WIN: AtomicBool = AtomicBool::new(false);
pub(crate) static PENDING_MODIFIER_HOTKEY: Mutex<Option<crate::engine::config::Hotkey>> = Mutex::new(None);

pub(crate) static SINGLE_KEY_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
pub(crate) static SINGLE_KEY_DIALOG_HWND: AtomicIsize = AtomicIsize::new(0);

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
    SINGLE_KEY_DIALOG_HWND.store(hwnd, Ordering::SeqCst);
    SINGLE_KEY_CAPTURE_ACTIVE.store(true, Ordering::SeqCst);
}

pub fn stop_single_key_capture() {
    SINGLE_KEY_CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
    SINGLE_KEY_DIALOG_HWND.store(0, Ordering::SeqCst);
}

pub fn start_combo_capture(hwnd: isize, allow_single: bool) {
    COMBO_DIALOG_HWND.store(hwnd, Ordering::SeqCst);
    HOTKEY_ALLOW_SINGLE_KEY.store(allow_single, Ordering::SeqCst);
    CAPTURE_CTRL.store(false, Ordering::SeqCst);
    CAPTURE_SHIFT.store(false, Ordering::SeqCst);
    CAPTURE_ALT.store(false, Ordering::SeqCst);
    CAPTURE_WIN.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = PENDING_MODIFIER_HOTKEY.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
        guard.clear();
    }
    COMBO_CAPTURE_ACTIVE.store(true, Ordering::SeqCst);
}

pub fn stop_combo_capture() {
    COMBO_CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
    HOTKEY_ALLOW_SINGLE_KEY.store(false, Ordering::SeqCst);
    COMBO_DIALOG_HWND.store(0, Ordering::SeqCst);
    CAPTURE_CTRL.store(false, Ordering::SeqCst);
    CAPTURE_SHIFT.store(false, Ordering::SeqCst);
    CAPTURE_ALT.store(false, Ordering::SeqCst);
    CAPTURE_WIN.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = PENDING_MODIFIER_HOTKEY.lock() {
        *guard = None;
    }
}

/// Trả về true nếu bất kỳ chế độ bắt phím nào đang hoạt động
pub fn is_any_capture_active() -> bool {
    SINGLE_KEY_CAPTURE_ACTIVE.load(Ordering::SeqCst) || COMBO_CAPTURE_ACTIVE.load(Ordering::SeqCst)
}

/// Trả về HWND của cửa sổ dialog đang thực hiện capture
pub fn active_capture_dialog() -> isize {
    let s = SINGLE_KEY_DIALOG_HWND.load(Ordering::SeqCst);
    if s != 0 {
        return s;
    }
    COMBO_DIALOG_HWND.load(Ordering::SeqCst)
}

/// Kiểm tra xem cửa sổ hiện tại (foreground) có phải là dialog capture hoặc con của nó không
pub fn is_capture_dialog_active() -> bool {
    let dlg = active_capture_dialog();
    if dlg == 0 {
        return false;
    }
    let fg = unsafe { GetForegroundWindow() };
    if fg == 0 {
        return false;
    }
    fg == dlg || unsafe { GetAncestor(fg, GA_ROOT) } == dlg || unsafe { GetAncestor(fg, GA_ROOTOWNER) } == dlg
}


/// Resets all internal modifier and hotkey tracking flags to prevent stuck modifiers
/// (e.g. after Win+L workstation lock, sleep, session change, or focus switch).
pub fn reset_modifier_states() {
    TRACKED_CTRL.store(false, Ordering::SeqCst);
    TRACKED_SHIFT.store(false, Ordering::SeqCst);
    TRACKED_ALT.store(false, Ordering::SeqCst);
    TRACKED_WIN.store(false, Ordering::SeqCst);
    CAPTURE_CTRL.store(false, Ordering::SeqCst);
    CAPTURE_SHIFT.store(false, Ordering::SeqCst);
    CAPTURE_ALT.store(false, Ordering::SeqCst);
    CAPTURE_WIN.store(false, Ordering::SeqCst);
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

pub fn get_pending_modifier_hotkey() -> Option<crate::engine::config::Hotkey> {
    if let Ok(guard) = PENDING_MODIFIER_HOTKEY.lock() {
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
/// Dùng cho: Đổi phím (Remap), Khóa phím (Disable), Phím kích hoạt Macro, Giữ/Nhả phím Macro
pub unsafe fn handle_single_key_capture(vk: u32, w_param: usize) -> Option<isize> {
    if SINGLE_KEY_CAPTURE_ACTIVE.load(Ordering::SeqCst) && is_capture_dialog_active() {
        let dialog_hwnd = SINGLE_KEY_DIALOG_HWND.load(Ordering::SeqCst);

        // Chặn phím Win bật Start Menu của Windows
        if is_win_vk(vk) {
            super::injector::send_vk_event(0xFF, false);
            super::injector::send_vk_event(0xFF, true);
        }

        if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
            if dialog_hwnd != 0 {
                unsafe {
                    PostMessageW(dialog_hwnd, WM_SINGLE_KEY_CAPTURED, vk as usize, 0);
                }
            }
        }
        return Some(1);
    }
    None
}

/// Xử lý chế độ bắt tổ hợp phím (Combo / Hotkey) từ Low-Level Keyboard Hook
/// Hỗ trợ bấm tổ hợp phím hoàn toàn tự do và thoải mái:
/// - Mọi kết hợp từ 1 đến 4 phím bổ trợ (Ctrl, Shift, Alt, Win) + 1 phím thường (ví dụ: Ctrl + Shift + Alt + Z)
/// - Mọi tổ hợp từ 2 đến 4 phím bổ trợ thuần (ví dụ: Ctrl + Shift, Ctrl + Alt, Ctrl + Shift + Alt)
/// - Hỗ trợ cả phím đơn lẻ khi bật cờ HOTKEY_ALLOW_SINGLE_KEY (dùng cho Macro Press)
pub unsafe fn handle_combo_capture(vk: u32, w_param: usize) -> Option<isize> {
    if COMBO_CAPTURE_ACTIVE.load(Ordering::SeqCst) && is_capture_dialog_active() {
        let dialog_hwnd = COMBO_DIALOG_HWND.load(Ordering::SeqCst);

        // Chặn phím Win bật Start Menu của Windows
        if is_win_vk(vk) {
            super::injector::send_vk_event(0xFF, false);
            super::injector::send_vk_event(0xFF, true);
        }

        if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
            if vk == 0x1B && !HOTKEY_ALLOW_SINGLE_KEY.load(Ordering::SeqCst) {
                stop_combo_capture();
                if dialog_hwnd != 0 {
                    unsafe {
                        PostMessageW(dialog_hwnd, WM_COMBO_CANCELLED, 0, 0);
                    }
                }
                return Some(1);
            }

            if is_ctrl_vk(vk) {
                CAPTURE_CTRL.store(true, Ordering::SeqCst);
            }
            if is_shift_vk(vk) {
                CAPTURE_SHIFT.store(true, Ordering::SeqCst);
            }
            if is_alt_vk(vk) {
                CAPTURE_ALT.store(true, Ordering::SeqCst);
            }
            if is_win_vk(vk) {
                CAPTURE_WIN.store(true, Ordering::SeqCst);
            }

            // Sync with physical hardware key states to guarantee no held modifier is ever missed
            let async_ctrl = unsafe { (GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0 };
            let async_shift = unsafe { (GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0 };
            let async_alt = unsafe { (GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0 };
            let async_win = unsafe {
                ((GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) != 0)
                    || ((GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) != 0)
            };

            let is_ctrl = CAPTURE_CTRL.load(Ordering::SeqCst) || is_ctrl_vk(vk) || async_ctrl;
            let is_shift = CAPTURE_SHIFT.load(Ordering::SeqCst) || is_shift_vk(vk) || async_shift;
            let is_alt = CAPTURE_ALT.load(Ordering::SeqCst) || is_alt_vk(vk) || async_alt;
            let is_win = CAPTURE_WIN.load(Ordering::SeqCst) || is_win_vk(vk) || async_win;

            let is_mod = is_modifier_vk(vk);

            if is_mod {
                let mod_candidate = crate::engine::config::Hotkey {
                    ctrl: is_ctrl,
                    shift: is_shift,
                    alt: is_alt,
                    win: is_win,
                    vk: 0,
                };
                let has_changed = if let Ok(mut guard) = PENDING_MODIFIER_HOTKEY.lock() {
                    let changed = *guard != Some(mod_candidate);
                    if changed {
                        *guard = Some(mod_candidate);
                    }
                    changed
                } else {
                    false
                };

                // Tránh bàn phím auto-repeat gửi WM_KEYDOWN liên tục khi người dùng giữ lâu phím bổ trợ,
                // gây post message và repaint hàng chục lần/giây làm chớp giật giao diện!
                if !has_changed {
                    return Some(1);
                }

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

                let display = if parts.len() <= 1 {
                    format!("{} + ...", parts.join(" + "))
                } else {
                    parts.join(" + ")
                };

                if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
                    *guard = display;
                }
                if dialog_hwnd != 0 {
                    unsafe {
                        PostMessageW(dialog_hwnd, WM_COMBO_CAPTURED, 0, 0);
                    }
                }
            } else {
                // Non-modifier key pressed (e.g. Z, Space, F1..F12, etc.)
                if let Ok(mut guard) = PENDING_MODIFIER_HOTKEY.lock() {
                    *guard = None;
                }

                let captured = crate::engine::config::Hotkey {
                    ctrl: is_ctrl,
                    shift: is_shift,
                    alt: is_alt,
                    win: is_win,
                    vk,
                };

                let allow_single = HOTKEY_ALLOW_SINGLE_KEY.load(Ordering::SeqCst);
                if captured.is_valid() || allow_single {
                    if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
                        *guard = Some(captured);
                    }
                    let display_text = if !is_ctrl && !is_shift && !is_alt && !is_win {
                        crate::engine::config::vk_to_name(vk).to_string()
                    } else {
                        captured.display_text()
                    };
                    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
                        *guard = display_text;
                    }
                    if dialog_hwnd != 0 {
                        unsafe {
                            PostMessageW(dialog_hwnd, WM_COMBO_CAPTURED, 0, 0);
                        }
                    }
                } else {
                    let key_name = crate::engine::config::vk_to_name(vk);
                    let display = format!("{} (Cần thêm Ctrl/Alt/Shift)", key_name);
                    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
                        *guard = display;
                    }
                    if dialog_hwnd != 0 {
                        unsafe {
                            PostMessageW(dialog_hwnd, WM_COMBO_CAPTURED, 0, 0);
                        }
                    }
                }
            }
        } else if w_param == WM_KEYUP || w_param == WM_SYSKEYUP {
            let was_ctrl = CAPTURE_CTRL.load(Ordering::SeqCst);
            let was_shift = CAPTURE_SHIFT.load(Ordering::SeqCst);
            let was_alt = CAPTURE_ALT.load(Ordering::SeqCst);
            let was_win = CAPTURE_WIN.load(Ordering::SeqCst);

            if is_ctrl_vk(vk) {
                CAPTURE_CTRL.store(false, Ordering::SeqCst);
            }
            if is_shift_vk(vk) {
                CAPTURE_SHIFT.store(false, Ordering::SeqCst);
            }
            if is_alt_vk(vk) {
                CAPTURE_ALT.store(false, Ordering::SeqCst);
            }
            if is_win_vk(vk) || was_win {
                CAPTURE_WIN.store(false, Ordering::SeqCst);
                super::injector::send_vk_event(0xFF, false);
                super::injector::send_vk_event(0xFF, true);
            }

            // Khi người dùng nhả phím bổ trợ (Ctrl/Shift/Alt/Win) mà trước đó đang giữ >= 2 phím
            // và chưa bấm phím thường nào, ta chốt tổ hợp thuần phím bổ trợ (ví dụ: Ctrl + Shift, Ctrl + Alt, Ctrl + Shift + Alt)!
            if is_modifier_vk(vk) {
                let pending = if let Ok(mut guard) = PENDING_MODIFIER_HOTKEY.lock() {
                    guard.take()
                } else {
                    None
                };

                let mod_count = was_ctrl as u8 + was_shift as u8 + was_alt as u8 + was_win as u8;
                if mod_count >= 2 && pending.is_some() {
                    let mod_hk = crate::engine::config::Hotkey {
                        ctrl: was_ctrl,
                        shift: was_shift,
                        alt: was_alt,
                        win: was_win,
                        vk: 0,
                    };
                    if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
                        *guard = Some(mod_hk);
                    }
                    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
                        *guard = mod_hk.display_text();
                    }
                    if dialog_hwnd != 0 {
                        unsafe {
                            PostMessageW(dialog_hwnd, WM_COMBO_CAPTURED, 0, 0);
                        }
                    }
                }
            }
        }
        return Some(1);
    }
    None
}

/// Cập nhật nhả phím cho các cờ theo dõi phím bổ trợ
pub unsafe fn update_modifier_tracking_keyup(vk: u32) {
    if is_ctrl_vk(vk) {
        TRACKED_CTRL.store(false, Ordering::SeqCst);
    }
    if is_shift_vk(vk) {
        TRACKED_SHIFT.store(false, Ordering::SeqCst);
    }
    if is_alt_vk(vk) {
        TRACKED_ALT.store(false, Ordering::SeqCst);
    }
    if is_win_vk(vk) {
        TRACKED_WIN.store(false, Ordering::SeqCst);
    }
}
