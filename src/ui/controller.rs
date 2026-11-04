//! Lightweight controller for thread-safe access to the engine and configuration.

use crate::engine::VietnameseEngine;
use crate::platform::win32::ENGINE_INSTANCE;

/// Runs `f` with the engine locked. Returns `None` if the engine is unavailable.
pub fn with_engine<R>(f: impl FnOnce(&mut VietnameseEngine) -> R) -> Option<R> {
    let mut guard = ENGINE_INSTANCE.lock().ok()?;
    guard.as_mut().map(f)
}

/// Persists config, macros and profiles (debounced, off the hook thread).
pub fn save(engine: &VietnameseEngine) {
    crate::engine::config_store::save_config_all_debounced(
        engine.config(),
        &engine.macro_table,
        &engine.profile_manager,
    );
}

/// Re-applies the effective sound settings (after a sound-related edit).
pub fn reconfigure_sound(engine: &VietnameseEngine) {
    crate::platform::win32::sound::reconfigure_sound(
        engine.config().sound_enabled,
        &engine.config().sound_profile,
        engine.config().sound_volume,
    );
}

/// Handles target capture event for an application process.
pub fn capture_target_for_app(proc: &str, tid: &str) {
    let outcome = with_engine(|engine| {
        if let Some(child) = engine.profile_manager.find_profile_by_process_mut(proc) {
            child.target_rules.push(crate::engine::profile::TargetRule::new(
                tid.to_string(),
                crate::engine::profile::KeyInjectionStrategy::AutocompleteGuard,
            ));
            let ret = (child.id.clone(), child.name.clone(), false);
            save(engine);
            Some(ret)
        } else {
            let pm = &mut engine.profile_manager;
            let base_id = proc
                .trim_end_matches(".exe")
                .replace([' ', '-', '.'], "_")
                .to_lowercase();
            let base_id = if base_id.is_empty() {
                "app".to_string()
            } else {
                base_id
            };
            let mut id = base_id.clone();
            let mut counter = 1;
            while id == "global" || pm.get_profile(&id).is_some() {
                id = format!("{}_{}", base_id, counter);
                counter += 1;
            }

            let raw_name = proc.trim_end_matches(".exe");
            let display_name = if let Some(first) = raw_name.chars().next() {
                format!("{}{}", first.to_uppercase(), &raw_name[first.len_utf8()..])
            } else {
                "App".to_string()
            };

            let mut new_profile = crate::engine::profile::AppProfile::new_child(
                id.clone(),
                display_name.clone(),
                proc.to_string(),
                "💻",
            );
            new_profile.target_rules.push(crate::engine::profile::TargetRule::new(
                tid.to_string(),
                crate::engine::profile::KeyInjectionStrategy::AutocompleteGuard,
            ));

            pm.add_profile(new_profile);
            pm.active_id = id.clone();
            pm.selected_id = id.clone();
            save(engine);
            Some((id, display_name, true))
        }
    })
    .flatten();

    if let Some((_prof_id, app_name, is_new)) = outcome {
        crate::platform::win32::app_tracker::sync_active_target_rules_flag();
        let strings = crate::language::current();
        let message = if is_new {
            strings
                .notif_target_created_profile
                .replace("{app}", &app_name)
                .replace("{target}", tid)
        } else {
            strings
                .notif_target_added_rule
                .replace("{app}", &app_name)
                .replace("{target}", tid)
        };
        crate::ui::notification::notify_info("Kora", message);
    }
}
