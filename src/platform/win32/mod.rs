//! Pure Win32 API Low-Level Keyboard Hook & Key Injection in Rust
//! No external crates required — uses direct Win32 FFI bindings.

pub mod app_detect;
pub mod app_tracker;
pub mod capture;
pub mod injector;
pub mod macro_executor;
pub mod sound;
pub mod types;
pub mod uia_detect;

pub use app_detect::*;
pub use app_tracker::*;
pub use capture::*;
pub use injector::*;
pub use macro_executor::*;
pub use sound::*;
pub use types::*;
pub use uia_detect::get_focused_element_id;

use crate::engine::VietnameseEngine;
use crate::engine::action::EngineAction;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

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

/// Pre-translation filter for the main message loop; receives a `*const MSG`
/// and returns `true` if the message was fully handled (e.g. `IsDialogMessageW`).
pub type MessageFilter = fn(*const std::ffi::c_void) -> bool;
static MESSAGE_FILTER: std::sync::OnceLock<MessageFilter> = std::sync::OnceLock::new();

/// Registers the message-loop pre-translation filter (first registration wins).
pub fn set_message_filter(filter: MessageFilter) {
    let _ = MESSAGE_FILTER.set(filter);
}

/// Reinstalls both low-level keyboard and mouse hooks, clears modifier states and engine buffer.
/// Called when Windows resumes from sleep / Modern Standby or when the workstation is unlocked.
pub fn reinstall_hooks() {
    reset_modifier_states();

    if let Ok(mut guard) = ENGINE_INSTANCE.lock()
        && let Some(ref mut engine) = *guard
    {
        engine.reset();
    }

    unsafe {
        if let Ok(mut h_guard) = HOOK_HANDLE.lock() {
            if *h_guard != 0 {
                UnhookWindowsHookEx(*h_guard);
                *h_guard = 0;
            }
            let kbd_hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                low_level_keyboard_proc,
                GetModuleHandleW(null_mut()),
                0,
            );
            if kbd_hook != 0 {
                *h_guard = kbd_hook;
            } else {
                eprintln!("[Kora] Không thể cài đặt lại Keyboard Hook!");
            }
        }

        if let Ok(mut m_guard) = MOUSE_HOOK_HANDLE.lock() {
            if *m_guard != 0 {
                UnhookWindowsHookEx(*m_guard);
                *m_guard = 0;
            }
            let mouse_hook = SetWindowsHookExW(
                WH_MOUSE_LL,
                low_level_mouse_proc,
                GetModuleHandleW(null_mut()),
                0,
            );
            if mouse_hook != 0 {
                *m_guard = mouse_hook;
            }
        }
    }

    app_tracker::reinit_app_tracker();
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
        {
            if (GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) == 0 {
                capture::TRACKED_CTRL.store(false, Ordering::SeqCst);
            }
            if (GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) == 0 {
                capture::TRACKED_SHIFT.store(false, Ordering::SeqCst);
            }
            if (GetAsyncKeyState(VK_MENU) as u16 & 0x8000) == 0 {
                capture::TRACKED_ALT.store(false, Ordering::SeqCst);
            }
            if ((GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) == 0)
                && ((GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) == 0)
            {
                capture::TRACKED_WIN.store(false, Ordering::SeqCst);
            }

            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
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

            // 1. Ignore synthetic events to avoid infinite recursion and prevent conflict
            // when running alongside another Vietnamese input engine (UniKey, EVKey, OpenKey, etc.).
            // Real physical keystrokes always have dw_extra_info == 0.
            if hook_struct.dw_extra_info != 0 {
                return CallNextHookEx(0, n_code, w_param, l_param);
            }

            // 2. Ignore keystrokes if an IME pad (Japanese, Chinese, etc.) is currently active
            let fg_hwnd = GetForegroundWindow();
            if fg_hwnd != 0 {
                let ime_hwnd = ImmGetDefaultIMEWnd(fg_hwnd);
                if ime_hwnd != 0 {
                    let mut is_ime_on: usize = 0;
                    let ok = SendMessageTimeoutW(
                        ime_hwnd,
                        WM_IME_CONTROL,
                        IMC_GETOPENSTATUS,
                        0,
                        SMTO_ABORTIFHUNG,
                        5,
                        &mut is_ime_on,
                    );
                    if ok != 0 && is_ime_on != 0 {
                        return CallNextHookEx(0, n_code, w_param, l_param);
                    }
                }
            }

            let vk = hook_struct.vk_code;

            // ========================================================================
            // EXCLUSIVE CAPTURE SANDBOX:
            // When any capture mode is active and the capture dialog is active,
            // 100% of keystrokes are intercepted and swallowed.
            // NO internal hotkey (switch mode), NO key macro, NO remap, NO disable,
            // NO Vietnamese typing engine, and NO Windows system shortcut can trigger!
            // ========================================================================
            if capture::is_any_capture_active() && capture::is_capture_dialog_active() {
                if let Some(res) = capture::handle_single_key_capture(vk, w_param) {
                    return res;
                }
                if let Some(res) = capture::handle_combo_capture(vk, w_param) {
                    return res;
                }
                return 1;
            }

            let (
                is_disabled,
                remap_target,
                macro_actions,
                switch_key_enabled,
                switch_key,
                target_capture_enabled,
                target_capture_hotkey,
            ) = if let Ok(guard) = ENGINE_INSTANCE.lock() {
                if let Some(ref engine) = *guard {
                    let eff_cfg = engine.effective_config();
                    let disabled = engine.is_key_disabled(vk);
                    let remap = engine.lookup_key_remap(vk);
                    let key_macro = if eff_cfg.key_macro_enabled {
                        engine.lookup_key_macro(vk).map(|s| s.to_vec())
                    } else {
                        None
                    };
                    (
                        disabled,
                        remap,
                        key_macro,
                        eff_cfg.switch_key_enabled,
                        eff_cfg.switch_key,
                        eff_cfg.target_capture_enabled,
                        eff_cfg.target_capture_hotkey,
                    )
                } else {
                    (
                        false,
                        None,
                        None,
                        true,
                        crate::engine::config::Hotkey::default(),
                        false,
                        crate::engine::config::Hotkey::new(true, false, false, false, 0xC0),
                    )
                }
            } else {
                (
                    false,
                    None,
                    None,
                    true,
                    crate::engine::config::Hotkey::default(),
                    false,
                    crate::engine::config::Hotkey::new(true, false, false, false, 0xC0),
                )
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

            // 3. Intercept Key Macro: swallow original event and trigger macro asynchronously
            if let Some(actions) = macro_actions {
                let is_down = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
                if is_down {
                    macro_executor::execute_actions(actions);
                }
                return 1;
            }

            // Handle key-up events for modifier-only hotkey triggers (e.g. Ctrl + Shift toggle)
            if w_param == WM_KEYUP || w_param == WM_SYSKEYUP {
                capture::update_modifier_tracking_keyup(vk);

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

                let is_shift = is_shift_vk(vk)
                    || ((GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0)
                    || ((GetAsyncKeyState(0xA0) as u16 & 0x8000) != 0) // VK_LSHIFT
                    || ((GetAsyncKeyState(0xA1) as u16 & 0x8000) != 0) // VK_RSHIFT
                    || TRACKED_SHIFT.load(Ordering::SeqCst);
                let is_ctrl = is_ctrl_vk(vk)
                    || ((GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0)
                    || ((GetAsyncKeyState(0xA2) as u16 & 0x8000) != 0) // VK_LCONTROL
                    || ((GetAsyncKeyState(0xA3) as u16 & 0x8000) != 0) // VK_RCONTROL
                    || TRACKED_CTRL.load(Ordering::SeqCst);
                let is_alt = is_alt_vk(vk)
                    || ((GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0)
                    || ((GetAsyncKeyState(0xA4) as u16 & 0x8000) != 0) // VK_LMENU
                    || ((GetAsyncKeyState(0xA5) as u16 & 0x8000) != 0) // VK_RMENU
                    || TRACKED_ALT.load(Ordering::SeqCst);
                let is_win = is_win_vk(vk)
                    || ((GetAsyncKeyState(VK_LWIN) as u16 & 0x8000) != 0)
                    || ((GetAsyncKeyState(VK_RWIN) as u16 & 0x8000) != 0)
                    || TRACKED_WIN.load(Ordering::SeqCst);

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
                        // Check modifier + key hotkey (e.g. Ctrl + Space, Alt + Z, Shift + Space, Win + Z)
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
                                if is_win {
                                    send_vk_event(0xFF, false);
                                    send_vk_event(0xFF, true);
                                }
                                return 1;
                            }
                            return CallNextHookEx(0, n_code, w_param, l_param);
                        }
                    }
                }

                // Check target input field capture hotkey (e.g. Ctrl + `)
                if target_capture_enabled
                    && target_capture_hotkey.is_valid()
                    && vk == target_capture_hotkey.vk
                    && is_ctrl == target_capture_hotkey.ctrl
                    && is_shift == target_capture_hotkey.shift
                    && is_alt == target_capture_hotkey.alt
                    && is_win == target_capture_hotkey.win
                {
                    capture::trigger_target_capture_async();
                    return 1;
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
                {
                    if let Ok(mut guard) = ENGINE_INSTANCE.lock()
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
        let filter = MESSAGE_FILTER.get().copied();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            if let Some(f) = filter
                && f(&msg as *const MSG as *const std::ffi::c_void)
            {
                continue;
            }
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
