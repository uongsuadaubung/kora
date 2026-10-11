//! Pure Win32 API Low-Level Keyboard Hook & Key Injection in Rust
//! No external crates required — uses direct Win32 FFI bindings.

pub mod app_detect;
pub mod app_tracker;
pub mod injector;
pub mod sound;
pub mod types;

pub use app_detect::*;
pub use app_tracker::*;
pub use injector::*;
pub use sound::*;
pub use types::*;

use crate::engine::VietnameseEngine;
use crate::engine::action::EngineAction;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

pub type ModeChangeCallback = fn(bool);
static MODE_CHANGE_CALLBACK: Mutex<Option<ModeChangeCallback>> = Mutex::new(None);

/// Registers a callback to be invoked whenever the typing mode toggles (Vietnamese <-> English).
pub fn set_mode_change_callback(cb: ModeChangeCallback) {
    if let Ok(mut guard) = MODE_CHANGE_CALLBACK.lock() {
        *guard = Some(cb);
    }
}

pub fn notify_mode_change(is_vietnamese: bool) {
    if let Ok(guard) = MODE_CHANGE_CALLBACK.lock()
        && let Some(cb) = *guard
    {
        cb(is_vietnamese);
    }
}

fn is_ctrl_vk(vk: u32) -> bool {
    matches!(vk, 0x11 | 0xA2 | 0xA3) // VK_CONTROL, VK_LCONTROL, VK_RCONTROL
}

fn is_shift_vk(vk: u32) -> bool {
    matches!(vk, 0x10 | 0xA0 | 0xA1) // VK_SHIFT, VK_LSHIFT, VK_RSHIFT
}

fn is_alt_vk(vk: u32) -> bool {
    matches!(vk, 0x12 | 0xA4 | 0xA5) // VK_MENU, VK_LMENU, VK_RMENU
}

fn is_win_vk(vk: u32) -> bool {
    matches!(vk, 0x5B | 0x5C) // VK_LWIN, VK_RWIN
}

fn is_modifier_vk(vk: u32) -> bool {
    is_ctrl_vk(vk) || is_shift_vk(vk) || is_alt_vk(vk) || is_win_vk(vk)
}

pub const WM_HOTKEY_CAPTURED: u32 = 0x8000 + 101;
pub const WM_REMAP_KEY_CAPTURED: u32 = 0x8000 + 102;

static HOTKEY_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
static HOTKEY_DIALOG_HWND: AtomicIsize = AtomicIsize::new(0);
static CAPTURED_HOTKEY: Mutex<Option<crate::engine::config::Hotkey>> = Mutex::new(None);
static CAPTURE_DISPLAY_TEXT: Mutex<String> = Mutex::new(String::new());
static HOTKEY_MOD_ARMED: AtomicBool = AtomicBool::new(false);

static TRACKED_CTRL: AtomicBool = AtomicBool::new(false);
static TRACKED_SHIFT: AtomicBool = AtomicBool::new(false);
static TRACKED_ALT: AtomicBool = AtomicBool::new(false);
static TRACKED_WIN: AtomicBool = AtomicBool::new(false);

static REMAP_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
static REMAP_DIALOG_HWND: AtomicIsize = AtomicIsize::new(0);

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

pub fn clear_captured_hotkey() {
    if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
        guard.clear();
    }
    TRACKED_CTRL.store(false, Ordering::SeqCst);
    TRACKED_SHIFT.store(false, Ordering::SeqCst);
    TRACKED_ALT.store(false, Ordering::SeqCst);
    TRACKED_WIN.store(false, Ordering::SeqCst);
}

pub fn set_captured_hotkey(hotkey: Option<crate::engine::config::Hotkey>, display: &str) {
    if let Ok(mut guard) = CAPTURED_HOTKEY.lock() {
        *guard = hotkey;
    }
    if let Ok(mut guard) = CAPTURE_DISPLAY_TEXT.lock() {
        *guard = display.to_string();
    }
}

// Global thread-safe Engine instance accessed by the hook callback and UI
pub static ENGINE_INSTANCE: Mutex<Option<VietnameseEngine>> = Mutex::new(None);
static HOOK_HANDLE: Mutex<isize> = Mutex::new(0);
static MOUSE_HOOK_HANDLE: Mutex<isize> = Mutex::new(0);

/// Mouse hook callback: resets the engine word session whenever user clicks somewhere
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: usize,
    l_param: isize,
) -> isize {
    unsafe {
        if n_code >= 0
            && (w_param == WM_LBUTTONDOWN || w_param == WM_RBUTTONDOWN || w_param == WM_MBUTTONDOWN)
            && let Ok(mut guard) = ENGINE_INSTANCE.lock()
            && let Some(ref mut engine) = *guard
        {
            if engine.config().debug && !engine.buffer.is_empty() {
                engine.log_debug(format!(
                    "[DBG][WIN32_MOUSE] Mouse button {:#X} clicked -> Reset engine buffer",
                    w_param
                ));
            }
            engine.reset();
        }
        CallNextHookEx(0, n_code, w_param, l_param)
    }
}

/// The Low-Level Keyboard Hook procedure
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: usize,
    l_param: isize,
) -> isize {
    unsafe {
        if n_code >= 0 {
            let hook_struct = *(l_param as *const KBDLLHOOKSTRUCT);

            // 1. Ignore our own synthetic events to avoid infinite recursion
            if hook_struct.dw_extra_info == MAGIC_EXTRA_INFO {
                return CallNextHookEx(0, n_code, w_param, l_param);
            }

            let vk = hook_struct.vk_code;

            // Handle single key capture mode for Remap / Key Dialogs
            if REMAP_CAPTURE_ACTIVE.load(Ordering::SeqCst) {
                if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
                    let dialog_hwnd = REMAP_DIALOG_HWND.load(Ordering::SeqCst);
                    if dialog_hwnd != 0 {
                        PostMessageW(dialog_hwnd, WM_REMAP_KEY_CAPTURED, vk as usize, 0);
                    }
                }
                return 1;
            }

            // Handle hotkey capture mode for HotkeyCaptureDialog
            if HOTKEY_CAPTURE_ACTIVE.load(Ordering::SeqCst) {
                if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
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

                    let is_shift = TRACKED_SHIFT.load(Ordering::SeqCst)
                        || ((GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0)
                        || is_shift_vk(vk);
                    let is_ctrl = TRACKED_CTRL.load(Ordering::SeqCst)
                        || ((GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0)
                        || is_ctrl_vk(vk);
                    let is_alt = TRACKED_ALT.load(Ordering::SeqCst)
                        || ((GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0)
                        || is_alt_vk(vk);
                    let is_win = TRACKED_WIN.load(Ordering::SeqCst)
                        || ((GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) != 0)
                        || ((GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) != 0)
                        || is_win_vk(vk);

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
                        PostMessageW(dialog_hwnd, WM_HOTKEY_CAPTURED, 0, 0);
                    }
                } else if w_param == WM_KEYUP || w_param == WM_SYSKEYUP {
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
                return 1;
            }

            let (is_disabled, remap_target, switch_key_enabled, switch_key) =
                if let Ok(guard) = ENGINE_INSTANCE.lock() {
                    if let Some(ref engine) = *guard {
                        let eff_cfg = engine.effective_config();
                        let disabled = engine.effective_disabled().contains(&vk);
                        let remap = engine
                            .effective_remaps()
                            .into_iter()
                            .find(|r| r.from_vk == vk)
                            .map(|r| r.to_vk);
                        (disabled, remap, eff_cfg.switch_key_enabled, eff_cfg.switch_key)
                    } else {
                        (false, None, true, crate::engine::config::Hotkey::default())
                    }
                } else {
                    (false, None, true, crate::engine::config::Hotkey::default())
                };

            // 1. Intercept Key Disabling: swallow event completely
            if is_disabled {
                return 1;
            }

            // 2. Intercept Key Remapping: swallow original event and dispatch target key
            if let Some(target_vk) = remap_target {
                let is_up = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
                send_vk_event(target_vk, is_up);
                return 1;
            }

            // Handle key-up events for modifier-only hotkey triggers (e.g. Ctrl + Shift toggle)
            if w_param == WM_KEYUP || w_param == WM_SYSKEYUP {
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

                release_key_sound(vk);
                if switch_key_enabled
                    && switch_key.vk == 0
                    && is_modifier_vk(vk)
                    && HOTKEY_MOD_ARMED.swap(false, Ordering::SeqCst)
                    && let Ok(mut guard) = ENGINE_INSTANCE.lock()
                    && let Some(ref mut engine) = *guard
                {
                    let new_state = engine.toggle_enabled();
                    crate::engine::config_store::save_config_all_debounced(
                        engine.config(),
                        &engine.macro_table,
                        &engine.profile_manager,
                    );
                    if new_state {
                        engine.log_debug("[Kora] >> Chế độ gõ: [V] TIẾNG VIỆT");
                    } else {
                        engine.log_debug("[Kora] >> Chế độ gõ: [E] TIẾNG ANH");
                    }
                    notify_mode_change(new_state);
                }
                return CallNextHookEx(0, n_code, w_param, l_param);
            }

            // Only process key-down events
            if w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN {
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

                trigger_key_sound(vk);

                // 2. Query hardware modifier states cleanly and reliably
                let is_caps = (GetKeyState(VK_CAPITAL) & 1) != 0;
                let is_shift = TRACKED_SHIFT.load(Ordering::SeqCst)
                    || ((GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0)
                    || is_shift_vk(vk);
                let is_ctrl = TRACKED_CTRL.load(Ordering::SeqCst)
                    || ((GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0)
                    || is_ctrl_vk(vk);
                let is_alt = TRACKED_ALT.load(Ordering::SeqCst)
                    || ((GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0)
                    || is_alt_vk(vk);
                let is_win = TRACKED_WIN.load(Ordering::SeqCst)
                    || ((GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) != 0)
                    || ((GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) != 0)
                    || is_win_vk(vk);

                // Check modifier-only hotkey arming (e.g. Ctrl + Shift)
                if switch_key_enabled {
                    if switch_key.vk == 0 {
                        if is_modifier_vk(vk) {
                            let match_ctrl = is_ctrl == switch_key.ctrl;
                            let match_shift = is_shift == switch_key.shift;
                            let match_alt = is_alt == switch_key.alt;
                            let match_win = is_win == switch_key.win;
                            if match_ctrl && match_shift && match_alt && match_win {
                                HOTKEY_MOD_ARMED.store(true, Ordering::SeqCst);
                            }
                            return CallNextHookEx(0, n_code, w_param, l_param);
                        } else {
                            HOTKEY_MOD_ARMED.store(false, Ordering::SeqCst);
                        }
                    } else {
                        // Check modifier + key hotkey (e.g. Ctrl + Space, Alt + Z, Shift + Space)
                        if vk == switch_key.vk
                            && is_ctrl == switch_key.ctrl
                            && is_shift == switch_key.shift
                            && is_alt == switch_key.alt
                            && is_win == switch_key.win
                        {
                            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                                && let Some(ref mut engine) = *guard
                            {
                                let new_state = engine.toggle_enabled();
                                crate::engine::config_store::save_config_all_debounced(
                                    engine.config(),
                                    &engine.macro_table,
                                    &engine.profile_manager,
                                );
                                if new_state {
                                    engine.log_debug("[Kora] >> Chế độ gõ: [V] TIẾNG VIỆT");
                                } else {
                                    engine.log_debug("[Kora] >> Chế độ gõ: [E] TIẾNG ANH");
                                }
                                notify_mode_change(new_state);
                                return 1;
                            }
                            return CallNextHookEx(0, n_code, w_param, l_param);
                        }
                    }
                }

                // If Ctrl, Alt, or Win are held down, let OS handle and reset buffer
                if is_ctrl || is_alt || is_win {
                    if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        if engine.config().debug && !engine.buffer.is_empty() {
                            engine.log_debug(format!(
                                "[DBG][WIN32_MOD] Modifier active (Ctrl: {is_ctrl}, Alt: {is_alt}, Win: {is_win}) -> Reset engine buffer"
                            ));
                        }
                        engine.reset();
                    }
                    return CallNextHookEx(0, n_code, w_param, l_param);
                }

                // Navigation / Termination keys reset the current word session
                let is_nav = matches!(
                    vk,
                    VK_RETURN
                        | VK_TAB
                        | VK_ESCAPE
                        | VK_LEFT
                        | VK_RIGHT
                        | VK_UP
                        | VK_DOWN
                        | VK_HOME
                        | VK_END
                        | VK_PRIOR
                        | VK_NEXT
                        | VK_DELETE
                        | VK_INSERT
                );
                if is_nav {
                    if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        if engine.config().debug && !engine.buffer.is_empty() {
                            engine.log_debug(format!(
                                "[DBG][WIN32_NAV] VK 0x{:02X} pressed -> Reset engine buffer",
                                vk
                            ));
                        }
                        engine.reset();
                    }
                    return CallNextHookEx(0, n_code, w_param, l_param);
                }

                // Handle Backspace
                if vk == VK_BACK {
                    if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                        && let Some(ref mut engine) = *guard
                    {
                        engine.on_backspace();
                    }
                    return CallNextHookEx(0, n_code, w_param, l_param);
                }

                // 3. Convert VK code to unicode character
                let mut key_state = [0u8; 256];
                GetKeyboardState(key_state.as_mut_ptr());

                // Synchronize Shift / CapsLock in the keyboard state array for ToUnicode
                if is_shift {
                    key_state[VK_SHIFT as usize] = 0x80;
                } else {
                    key_state[VK_SHIFT as usize] = 0;
                }
                if is_caps {
                    key_state[VK_CAPITAL as usize] = 0x01;
                } else {
                    key_state[VK_CAPITAL as usize] = 0;
                }

                let mut buff = [0u16; 4];
                let count = ToUnicode(
                    vk,
                    hook_struct.scan_code,
                    key_state.as_ptr(),
                    buff.as_mut_ptr(),
                    4,
                    0,
                );

                if count > 0
                    && let Some(ch) = char::decode_utf16(buff[0..count as usize].iter().cloned())
                        .next()
                        .and_then(|r| r.ok())
                    && let Ok(mut guard) = ENGINE_INSTANCE.lock()
                    && let Some(ref mut engine) = *guard
                {
                    let action = engine.on_key(ch, is_shift, is_caps);
                    match action {
                        EngineAction::Passthrough => {
                            // Let OS handle original keystroke
                            return CallNextHookEx(0, n_code, w_param, l_param);
                        }
                        EngineAction::Replace { backspaces, output } => {
                            // Consume this key, send backspaces and replacement string in a single atomic SendInput call
                            send_replace(backspaces, &output);
                            return 1; // Intercept: do not pass to target window
                        }
                        EngineAction::Consume => {
                            return 1; // Drop key entirely
                        }
                    }
                }
            }
        }

        CallNextHookEx(0, n_code, w_param, l_param)
    }
}

/// Installs the Windows keyboard & mouse hooks and starts the Win32 message pump
pub fn run_hook_loop(engine: VietnameseEngine, on_ready: impl FnOnce()) {
    let sound_enabled = engine.config().sound_enabled;
    let sound_profile = engine.config().sound_profile.clone();
    let sound_volume = engine.config().sound_volume;
    {
        let mut guard = ENGINE_INSTANCE.lock().unwrap();
        *guard = Some(engine);
    }

    // Initialize mechanical keyboard sound engine
    sound::init_sound(sound_enabled, &sound_profile, sound_volume);

    unsafe {
        let kbd_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            low_level_keyboard_proc,
            GetModuleHandleW(null_mut()),
            0,
        );

        if kbd_hook == 0 {
            eprintln!("[Error] Không thể cài đặt Windows Keyboard Hook!");
            return;
        }

        let mouse_hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            low_level_mouse_proc,
            GetModuleHandleW(null_mut()),
            0,
        );

        {
            let mut h_guard = HOOK_HANDLE.lock().unwrap();
            *h_guard = kbd_hook;
        }
        if mouse_hook != 0 {
            let mut m_guard = MOUSE_HOOK_HANDLE.lock().unwrap();
            *m_guard = mouse_hook;
        }

        println!(">> Windows Hook đã kích hoạt! Hãy mở Notepad/Browser và gõ tiếng Việt để test.");
        println!(">> Nhấn Ctrl + C trong cửa sổ này để tắt.");

        // Invoke caller's initialization callback (e.g. initialize UI)
        on_ready();

        // Initialize foreground application tracker for automatic profile switching
        app_tracker::init_app_tracker();

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Flush any pending debounced config saves before exiting
        crate::engine::config_store::flush_config_debounced();

        // Cleanup
        app_tracker::shutdown_app_tracker();
        let h = *HOOK_HANDLE.lock().unwrap();
        if h != 0 {
            UnhookWindowsHookEx(h);
        }
        let mh = *MOUSE_HOOK_HANDLE.lock().unwrap();
        if mh != 0 {
            UnhookWindowsHookEx(mh);
        }
    }
}

/// Trims the process working set, returning unused RAM and cached pages back to Windows.
/// Reduces resident physical RAM footprint down to ~1-2 MB.
pub fn trim_working_set() {
    unsafe {
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}
