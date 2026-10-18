//! Foreground Window & Active Application Tracker for Profile Switching
//! Uses Win32 Event Hook (EVENT_SYSTEM_FOREGROUND) for zero-CPU, real-time profile switching.

use super::app_detect::get_foreground_process_name;
use super::types::{
    EVENT_SYSTEM_FOREGROUND, SetWinEventHook, UnhookWinEvent, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS,
};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

static WIN_EVENT_HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);
static HAS_ACTIVE_TARGET_RULES: AtomicBool = AtomicBool::new(false);
static TARGET_UPDATE_TRIGGER: OnceLock<std::sync::mpsc::SyncSender<()>> = OnceLock::new();

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

fn ensure_worker_started() -> std::sync::mpsc::SyncSender<()> {
    TARGET_UPDATE_TRIGGER
        .get_or_init(|| {
            let (tx, rx) = std::sync::mpsc::sync_channel::<()>(1);
            std::thread::Builder::new()
                .name("kora-uia-worker".to_string())
                .spawn(move || {
                    while rx.recv().is_ok() {
                        // Debounce rapid focus events (e.g. 40ms)
                        std::thread::sleep(std::time::Duration::from_millis(40));
                        while rx.try_recv().is_ok() {}

                        let (has_rules, app_strat) = if let Ok(guard) = super::ENGINE_INSTANCE.lock() {
                            if let Some(ref engine) = *guard {
                                let pid = &engine.profile_manager.active_id;
                                engine
                                    .profile_manager
                                    .get_profile(pid)
                                    .map(|p| {
                                        (
                                            !p.target_rules.is_empty(),
                                            p.app_strategy.map(|s| s.to_raw_u8()).unwrap_or(0),
                                        )
                                    })
                                    .unwrap_or((false, 0))
                            } else {
                                (false, 0)
                            }
                        } else {
                            (false, 0)
                        };

                        if !has_rules {
                            super::app_detect::set_current_target_strategy_raw(app_strat);
                            continue;
                        }

                        if let Some(target_id) = super::uia_detect::get_focused_element_id() {
                            let strategy = if let Ok(guard) = super::ENGINE_INSTANCE.lock() {
                                if let Some(ref engine) = *guard {
                                    let pid = &engine.profile_manager.active_id;
                                    engine
                                        .profile_manager
                                        .get_profile(pid)
                                        .and_then(|p| {
                                            p.target_rules
                                                .iter()
                                                .find(|r| r.target_id.eq_ignore_ascii_case(&target_id))
                                                .map(|r| r.strategy.to_raw_u8())
                                        })
                                        .unwrap_or(app_strat)
                                } else {
                                    app_strat
                                }
                            } else {
                                app_strat
                            };
                            super::app_detect::set_current_target_strategy_raw(strategy);
                        } else {
                            super::app_detect::set_current_target_strategy_raw(app_strat);
                        }
                    }
                })
                .expect("Failed to spawn UIA worker thread");
            tx
        })
        .clone()
}

/// Schedules an asynchronous check of the focused element on the worker thread.
/// Completely non-blocking: execution completes in microseconds with ZERO lag on hooks.
pub fn schedule_target_strategy_update() {
    if HAS_ACTIVE_TARGET_RULES.load(Ordering::Relaxed) {
        let tx = ensure_worker_started();
        let _ = tx.try_send(());
    }
}

/// Synchronizes the atomic flag indicating whether the active profile has any target rules.
pub fn sync_active_target_rules_flag() {
    let (has_rules, app_strat) = if let Ok(guard) = super::ENGINE_INSTANCE.lock() {
        if let Some(ref engine) = *guard {
            let pid = &engine.profile_manager.active_id;
            engine
                .profile_manager
                .get_profile(pid)
                .map(|p| {
                    (
                        !p.target_rules.is_empty(),
                        p.app_strategy.map(|s| s.to_raw_u8()).unwrap_or(0),
                    )
                })
                .unwrap_or((false, 0))
        } else {
            (false, 0)
        }
    } else {
        (false, 0)
    };

    HAS_ACTIVE_TARGET_RULES.store(has_rules, Ordering::Relaxed);
    if has_rules {
        schedule_target_strategy_update();
    } else {
        super::app_detect::set_current_target_strategy_raw(app_strat);
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
        (changed, new_id, changed.then(|| engine.effective_config().clone()))
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
    sync_active_target_rules_flag();
    (changed, new_id)
}

static FOCUS_EVENT_HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

/// Updates the current cached key injection strategy asynchronously
pub fn update_target_strategy_for_current_focus() {
    schedule_target_strategy_update();
}

/// Callback for WinEventHook on EVENT_OBJECT_FOCUS.
/// If active profile has no target rules, returns in ~1 nanosecond.
/// If target rules exist, non-blockingly signals background worker.
unsafe extern "system" fn focus_event_proc(
    _h_hook: isize,
    _event: u32,
    _hwnd: isize,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if HAS_ACTIVE_TARGET_RULES.load(Ordering::Relaxed) {
        schedule_target_strategy_update();
    }
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

        let f_hook = SetWinEventHook(
            super::types::EVENT_OBJECT_FOCUS,
            super::types::EVENT_OBJECT_FOCUS,
            0,
            Some(focus_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if f_hook != 0 {
            FOCUS_EVENT_HOOK_HANDLE.store(f_hook, Ordering::SeqCst);
        }
    }
    // The hook only fires on future switches: resolve the app already in front now
    let (changed, _) = check_and_switch_profile();
    if !changed
        && let Ok(guard) = super::ENGINE_INSTANCE.lock()
        && let Some(ref engine) = *guard
    {
        let eff = engine.effective_config();
        super::notify_mode_change(eff.enabled);
    }
    sync_active_target_rules_flag();
}

/// Unhooks the foreground window event hook
pub fn shutdown_app_tracker() {
    let hook = WIN_EVENT_HOOK_HANDLE.swap(0, Ordering::SeqCst);
    if hook != 0 {
        unsafe {
            UnhookWinEvent(hook);
        }
    }
    let f_hook = FOCUS_EVENT_HOOK_HANDLE.swap(0, Ordering::SeqCst);
    if f_hook != 0 {
        unsafe {
            UnhookWinEvent(f_hook);
        }
    }
}

/// Reinitializes the foreground app tracker after sleep / session change
pub fn reinit_app_tracker() {
    shutdown_app_tracker();
    init_app_tracker();
}


