//! Foreground Window & Active Application Tracker for Profile Switching
//! Uses Win32 Event Hook (EVENT_SYSTEM_FOREGROUND) for zero-CPU, real-time profile switching.

use super::app_detect::get_foreground_process_name;
use super::types::{
    EVENT_SYSTEM_FOREGROUND, SetWinEventHook, UnhookWinEvent, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicIsize, Ordering};

static WIN_EVENT_HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

pub type ProfileChangeCallback = fn(&str);
static PROFILE_CHANGE_CALLBACK: Mutex<Option<ProfileChangeCallback>> = Mutex::new(None);

pub fn set_profile_change_callback(cb: ProfileChangeCallback) {
    if let Ok(mut guard) = PROFILE_CHANGE_CALLBACK.lock() {
        *guard = Some(cb);
    }
}

pub fn notify_profile_change(profile_id: &str) {
    if let Ok(guard) = PROFILE_CHANGE_CALLBACK.lock()
        && let Some(cb) = *guard
    {
        cb(profile_id);
    }
}

/// Checks the foreground window executable and switches active profile if needed
pub fn check_and_switch_profile() -> (bool, String) {
    let exe = get_foreground_process_name();
    if exe.is_empty() {
        return (false, "global".to_string());
    }

    let switched = if let Ok(mut guard) = super::ENGINE_INSTANCE.lock()
        && let Some(ref mut engine) = *guard
    {
        let (changed, new_id) = engine.switch_profile_by_process(&exe);
        (changed, new_id, changed.then(|| engine.effective_config()))
    } else {
        return (false, "global".to_string());
    };

    // Notify outside the engine lock so callbacks may lock it themselves
    let (changed, new_id, effective) = switched;
    if let Some(eff) = effective {
        super::sound::reconfigure_sound(eff.sound_enabled, &eff.sound_profile, eff.sound_volume);
        super::notify_mode_change(eff.enabled);
        notify_profile_change(&new_id);
    }
    (changed, new_id)
}

/// Callback for WinEventHook on EVENT_SYSTEM_FOREGROUND
unsafe extern "system" fn foreground_event_proc(
    _h_hook: isize,
    _event: u32,
    _hwnd: isize,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    check_and_switch_profile();
}

/// Initializes the foreground window event hook
pub fn init_app_tracker() {
    if WIN_EVENT_HOOK_HANDLE.load(Ordering::SeqCst) != 0 {
        return;
    }

    unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            0,
            Some(foreground_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if hook != 0 {
            WIN_EVENT_HOOK_HANDLE.store(hook, Ordering::SeqCst);
        }
    }
    // The hook only fires on future switches: resolve the app already in front now
    check_and_switch_profile();
}

/// Unhooks the foreground window event hook
pub fn shutdown_app_tracker() {
    let hook = WIN_EVENT_HOOK_HANDLE.swap(0, Ordering::SeqCst);
    if hook != 0 {
        unsafe {
            UnhookWinEvent(hook);
        }
    }
}

