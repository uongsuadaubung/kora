//! Thin controller between the views and the engine.
//!
//! Every UI edit goes through here so that locking, root/child-profile dispatch and
//! persistence live in one place instead of being copy-pasted into each tab/dialog.
//!
//! Rule: the closures run with `ENGINE_INSTANCE` locked. They must not call anything
//! that pumps messages (MessageBox, dialogs, TrackPopupMenu, SendMessage to another
//! thread): the keyboard hook runs on the same thread and would self-deadlock.

use super::GLOBAL_PROFILE_ID;
use crate::engine::config::Hotkey;
use crate::engine::macro_table::{MacroEntry, MacroType};
use crate::engine::profile::{
    AppProfile, Child, KeyMacroRule, KeyRemapRule, MacroAction, ResolvedDisableItem,
    ResolvedKeyMacroItem, ResolvedMacroItem, ResolvedRemapItem, RuleOrigin,
};
use crate::engine::{EngineConfig, VietnameseEngine};
use crate::platform::win32::ENGINE_INSTANCE;
use crate::ui_framework::components::action_table::{ActionVariant, TableAction};

/// The settings object an edit applies to.
pub enum EditTarget<'a> {
    /// Root profile selected: edit the global config.
    Global(&'a mut EngineConfig),
    /// Child profile selected: edit its overrides (`AppProfile<Child>` guarantees `!is_root`).
    Child(&'a mut AppProfile<Child>),
}

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
    let eff = engine.effective_config();
    crate::platform::win32::sound::reconfigure_sound(
        eff.sound_enabled,
        &eff.sound_profile,
        eff.sound_volume,
    );
}

/// Applies `f` to the profile selected in the panel (global config for the root
/// profile, the child profile otherwise), then saves (debounced).
pub fn edit_selected<R>(f: impl FnOnce(EditTarget<'_>) -> R) -> Option<R> {
    edit_selected_then(f, |_| {})
}

/// Like [`edit_selected`], then runs `after` on the engine (same lock) before saving.
pub fn edit_selected_then<R>(
    f: impl FnOnce(EditTarget<'_>) -> R,
    after: impl FnOnce(&mut VietnameseEngine),
) -> Option<R> {
    with_engine(|engine| {
        let sel = engine.profile_manager.selected_id.clone();
        let r = if sel == GLOBAL_PROFILE_ID {
            Some(f(EditTarget::Global(engine.config_mut())))
        } else {
            engine
                .profile_manager
                .get_profile_mut(&sel)
                .map(|p| f(EditTarget::Child(p)))
        }?;
        after(engine);
        engine.refresh_effective_config();
        save(engine);
        Some(r)
    })
    .flatten()
}

/// Id of the profile currently selected in the panel.
pub fn selected_id() -> Option<String> {
    with_engine(|e| e.profile_manager.selected_id.clone())
}

/// Macro list rows for the selected profile.
pub fn selected_macro_items(engine: &VietnameseEngine) -> Vec<ResolvedMacroItem> {
    let pm = &engine.profile_manager;
    pm.resolve_macro_items_for_ui(&pm.selected_id, &engine.macro_table.get_sorted_entries())
}

/// Remap list rows for the selected profile.
pub fn selected_remap_items(engine: &VietnameseEngine) -> Vec<ResolvedRemapItem> {
    let pm = &engine.profile_manager;
    pm.resolve_remap_items_for_ui(&pm.selected_id)
}

/// Disabled-key list rows for the selected profile.
pub fn selected_disable_items(engine: &VietnameseEngine) -> Vec<ResolvedDisableItem> {
    let pm = &engine.profile_manager;
    pm.resolve_disable_items_for_ui(&pm.selected_id)
}

/// Key Macro list rows for the selected profile.
pub fn selected_key_macro_items(engine: &VietnameseEngine) -> Vec<ResolvedKeyMacroItem> {
    let pm = &engine.profile_manager;
    pm.resolve_key_macro_items_for_ui(&pm.selected_id)
}

/// Re-populates the macro list from the engine (call without holding the engine lock).
pub fn refresh_macro_list() {
    if let (Some(items), Some(c)) = (with_engine(|e| selected_macro_items(e)), super::controls()) {
        c.tab_macro.populate_macro_items(&items);
    }
}

/// Re-populates the remap list from the engine (call without holding the engine lock).
pub fn refresh_remap_list() {
    if let (Some(items), Some(c)) = (with_engine(|e| selected_remap_items(e)), super::controls()) {
        c.tab_keys.populate_remaps(&items);
        refresh_keys_overview();
    }
}

/// Re-populates the disabled-key list from the engine (call without holding the engine lock).
pub fn refresh_disable_list() {
    if let (Some(items), Some(c)) = (
        with_engine(|e| selected_disable_items(e)),
        super::controls(),
    ) {
        c.tab_keys.populate_disabled(&items);
        refresh_keys_overview();
    }
}

/// Re-populates the key macro list from the engine (call without holding the engine lock).
pub fn refresh_key_macro_list() {
    if let (Some(items), Some(c)) = (
        with_engine(|e| selected_key_macro_items(e)),
        super::controls(),
    ) {
        c.tab_keys.populate_key_macros(&items);
        refresh_keys_overview();
    }
}

/// Re-populates the consolidated key overview table and diagnostics from the engine.
pub fn refresh_keys_overview() {
    if let (Some(items), Some(c)) = (
        with_engine(|e| {
            (
                selected_remap_items(e),
                selected_disable_items(e),
                selected_key_macro_items(e),
            )
        }),
        super::controls(),
    ) {
        c.tab_keys.refresh_overview(&items.0, &items.1, &items.2);
    }
}

/// Builds the standard Edit / Mute / Unmute / Delete action buttons for any rule table row.
pub fn build_rule_actions(
    origin: RuleOrigin,
    is_active: bool,
    on_edit: Option<impl Fn() + Send + Sync + 'static>,
    on_toggle_mute: impl Fn(bool) + Send + Sync + 'static,
    on_delete: impl Fn() + Send + Sync + 'static,
) -> Vec<TableAction> {
    let strings = crate::language::current();
    let mut actions = Vec::with_capacity(3);
    match origin {
        RuleOrigin::MutedGlobal => {
            actions.push(
                TableAction::new(strings.action_unmute, ActionVariant::Primary)
                    .with_callback(move || on_toggle_mute(false)),
            );
        }
        RuleOrigin::InheritedGlobal | RuleOrigin::OverriddenGlobal => {
            if let Some(edit_fn) = on_edit {
                actions.push(TableAction::edit(strings.action_edit, edit_fn));
            }
            actions.push(TableAction::secondary(strings.action_mute, move || {
                on_toggle_mute(true)
            }));
        }
        RuleOrigin::Root | RuleOrigin::AppSpecific => {
            if let Some(edit_fn) = on_edit {
                actions.push(TableAction::edit(strings.action_edit, edit_fn));
            }
            let (label, variant, mute_val) = if is_active {
                (strings.action_mute, ActionVariant::Secondary, true)
            } else {
                (strings.action_unmute, ActionVariant::Primary, false)
            };
            actions.push(
                TableAction::new(label, variant).with_callback(move || on_toggle_mute(mute_val)),
            );
            actions.push(TableAction::delete(strings.action_delete, on_delete));
        }
    }
    actions
}

// =============================================================================
// Centralized Rule CRUD & Mute/Unmute Operations (Zero Root/Child branching in Views)
// =============================================================================

/// Mutes or unmutes a text macro (`Gõ tắt`) on the selected profile and refreshes the UI list.
pub fn set_macro_muted(key: String, mute: bool) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        if sel_id == GLOBAL_PROFILE_ID {
            let target = &mut pm.global_profile;
            if mute {
                if !target
                    .muted_macros
                    .iter()
                    .any(|k| k.eq_ignore_ascii_case(&key))
                {
                    target.muted_macros.push(key);
                }
            } else {
                target
                    .muted_macros
                    .retain(|k| !k.eq_ignore_ascii_case(&key));
            }
        } else {
            let is_root_muted = pm
                .global_profile
                .muted_macros
                .iter()
                .any(|k| k.eq_ignore_ascii_case(&key));
            let global_macro = engine.macro_table.get(&key).cloned();
            let Some(child) = pm.get_profile_mut(&sel_id) else {
                return false;
            };
            if mute {
                child.overridden_macros.remove(&key.to_lowercase());
                child
                    .app_macros
                    .retain(|m| !m.key.eq_ignore_ascii_case(&key));
                if !child
                    .muted_macros
                    .iter()
                    .any(|k| k.eq_ignore_ascii_case(&key))
                {
                    child.muted_macros.push(key);
                }
            } else {
                child.muted_macros.retain(|k| !k.eq_ignore_ascii_case(&key));
                if is_root_muted
                    && !child
                        .app_macros
                        .iter()
                        .any(|m| m.key.eq_ignore_ascii_case(&key))
                    && let Some(m) = global_macro
                {
                    child.app_macros.push(m);
                }
            }
        }
        save(engine);
        true
    });
    if changed == Some(true) {
        refresh_macro_list();
    }
}

/// Deletes a text macro (`Gõ tắt`) from the selected profile and refreshes the UI list.
pub fn delete_macro(key_to_del: &str) {
    with_engine(|engine| {
        let sel_id = engine.profile_manager.selected_id.clone();
        if sel_id == GLOBAL_PROFILE_ID {
            engine.macro_table.remove(key_to_del);
        } else if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
            child
                .app_macros
                .retain(|m| !m.key.eq_ignore_ascii_case(key_to_del));
        }
        save(engine);
    });
    refresh_macro_list();
}

/// Inserts or updates a text macro (`Gõ tắt`) on the selected profile and refreshes the UI list.
pub fn save_macro(original_key: Option<&str>, key: &str, val: &str, mtype: MacroType) {
    with_engine(|engine| {
        let sel_id = engine.profile_manager.selected_id.clone();
        if sel_id == GLOBAL_PROFILE_ID {
            if let Some(orig) = original_key
                && orig != key
            {
                engine.macro_table.remove(orig);
            }
            engine.macro_table.insert_typed(key, val, mtype);
        } else if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
            let is_global = engine.macro_table.get(key).is_some();
            if let Some(orig) = original_key
                && orig != key
            {
                child.app_macros.retain(|m| m.key != orig);
                child.overridden_macros.remove(orig);
            }
            if is_global {
                child.muted_macros.retain(|k| k != key);
                child
                    .overridden_macros
                    .insert(key.to_string(), (val.to_string(), mtype));
            } else {
                child.app_macros.retain(|m| m.key != key);
                child.app_macros.push(MacroEntry {
                    key: key.to_string(),
                    value: val.to_string(),
                    macro_type: mtype,
                });
            }
        }
        save(engine);
    });
    refresh_macro_list();
}

/// Mutes or unmutes a key remap rule on the selected profile and refreshes the UI list.
pub fn set_remap_muted(from_vk: u32, mute: bool) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        if sel_id == GLOBAL_PROFILE_ID {
            let target = &mut pm.global_profile;
            if mute {
                if !target.muted_remaps.contains(&from_vk) {
                    target.muted_remaps.push(from_vk);
                }
            } else {
                target.muted_remaps.retain(|&v| v != from_vk);
            }
        } else {
            let is_root_muted = pm.global_profile.muted_remaps.contains(&from_vk);
            let global_rule = pm
                .global_profile
                .app_remaps
                .iter()
                .find(|r| r.from_vk == from_vk)
                .cloned();
            let Some(child) = pm.get_profile_mut(&sel_id) else {
                return false;
            };
            if mute {
                child.overridden_remaps.remove(&from_vk);
                child.app_remaps.retain(|r| r.from_vk != from_vk);
                if !child.muted_remaps.contains(&from_vk) {
                    child.muted_remaps.push(from_vk);
                }
            } else {
                child.muted_remaps.retain(|&v| v != from_vk);
                if is_root_muted
                    && !child.app_remaps.iter().any(|r| r.from_vk == from_vk)
                    && let Some(rule) = global_rule
                {
                    child.app_remaps.push(rule);
                }
            }
        }
        save(engine);
        true
    });
    if changed == Some(true) {
        refresh_remap_list();
    }
}

/// Deletes a key remap rule from the selected profile and refreshes the UI list.
pub fn delete_remap(from_vk: u32) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        let remaps = if sel_id == GLOBAL_PROFILE_ID {
            Some(&mut pm.global_profile.app_remaps)
        } else {
            pm.get_profile_mut(&sel_id).map(|p| &mut p.app_remaps)
        }?;
        remaps.retain(|r| r.from_vk != from_vk);
        save(engine);
        Some(true)
    })
    .flatten();
    if changed == Some(true) {
        refresh_remap_list();
    }
}

/// Inserts or updates a key remap rule on the selected profile and refreshes the UI list.
pub fn save_remap(orig_from: Option<u32>, from: u32, to: u32) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        if sel_id == GLOBAL_PROFILE_ID {
            if let Some(orig) = orig_from {
                pm.global_profile.app_remaps.retain(|r| r.from_vk != orig);
            }
            pm.global_profile.app_remaps.retain(|r| r.from_vk != from);
            pm.global_profile
                .app_remaps
                .push(KeyRemapRule::new(from, to));
            save(engine);
            true
        } else {
            let is_global = pm
                .global_profile
                .app_remaps
                .iter()
                .any(|r| r.from_vk == from);
            let Some(child) = pm.get_profile_mut(&sel_id) else {
                return false;
            };
            if let Some(orig) = orig_from {
                child.overridden_remaps.remove(&orig);
                child.app_remaps.retain(|r| r.from_vk != orig);
            }
            if is_global {
                child.muted_remaps.retain(|&v| v != from);
                child.overridden_remaps.insert(from, to);
            } else {
                child.app_remaps.retain(|r| r.from_vk != from);
                child.app_remaps.push(KeyRemapRule::new(from, to));
            }
            save(engine);
            true
        }
    });
    if changed == Some(true) {
        refresh_remap_list();
    }
}

/// Mutes or unmutes a disabled-key rule on the selected profile and refreshes the UI list.
pub fn set_disable_muted(vk: u32, mute: bool) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        if sel_id == GLOBAL_PROFILE_ID {
            let target = &mut pm.global_profile;
            if mute {
                if !target.unmuted_disabled.contains(&vk) {
                    target.unmuted_disabled.push(vk);
                }
            } else {
                target.unmuted_disabled.retain(|&v| v != vk);
            }
        } else {
            let is_root_unmuted = pm.global_profile.unmuted_disabled.contains(&vk);
            let Some(child) = pm.get_profile_mut(&sel_id) else {
                return false;
            };
            if mute {
                if !child.unmuted_disabled.contains(&vk) {
                    child.unmuted_disabled.push(vk);
                }
                child.app_disabled.retain(|&v| v != vk);
            } else {
                child.unmuted_disabled.retain(|&v| v != vk);
                if is_root_unmuted && !child.app_disabled.contains(&vk) {
                    child.app_disabled.push(vk);
                }
            }
        }
        save(engine);
        true
    });
    if changed == Some(true) {
        refresh_disable_list();
    }
}

/// Deletes a disabled-key rule from the selected profile and refreshes the UI list.
pub fn delete_disable(vk: u32) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        let disabled = if sel_id == GLOBAL_PROFILE_ID {
            Some(&mut pm.global_profile.app_disabled)
        } else {
            pm.get_profile_mut(&sel_id).map(|p| &mut p.app_disabled)
        }?;
        disabled.retain(|&v| v != vk);
        save(engine);
        Some(true)
    })
    .flatten();
    if changed == Some(true) {
        refresh_disable_list();
    }
}

/// Adds or re-enables a disabled-key rule on the selected profile and refreshes the UI list.
pub fn save_disable(vk: u32) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        let is_global = pm.global_profile.app_disabled.contains(&vk);
        let modified = if sel_id == GLOBAL_PROFILE_ID {
            if !is_global {
                pm.global_profile.app_disabled.push(vk);
                true
            } else {
                false
            }
        } else if let Some(child) = pm.get_profile_mut(&sel_id) {
            if is_global {
                let had = child.unmuted_disabled.contains(&vk);
                child.unmuted_disabled.retain(|&v| v != vk);
                had
            } else if !child.app_disabled.contains(&vk) {
                child.app_disabled.push(vk);
                true
            } else {
                false
            }
        } else {
            false
        };
        if modified {
            save(engine);
        }
        modified
    });
    if changed == Some(true) {
        refresh_disable_list();
    }
}

/// Mutes or unmutes a key macro rule on the selected profile and refreshes the UI list.
pub fn set_key_macro_muted(trigger_vk: u32, mute: bool) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        if sel_id == GLOBAL_PROFILE_ID {
            let target = &mut pm.global_profile;
            if mute {
                if !target.muted_key_macros.contains(&trigger_vk) {
                    target.muted_key_macros.push(trigger_vk);
                }
            } else {
                target.muted_key_macros.retain(|&v| v != trigger_vk);
            }
        } else {
            let is_root_muted = pm.global_profile.muted_key_macros.contains(&trigger_vk);
            let global_macro = pm
                .global_profile
                .app_key_macros
                .iter()
                .find(|m| m.trigger_vk == trigger_vk)
                .cloned();
            let Some(child) = pm.get_profile_mut(&sel_id) else {
                return false;
            };
            if mute {
                child.overridden_key_macros.remove(&trigger_vk);
                child.app_key_macros.retain(|m| m.trigger_vk != trigger_vk);
                if !child.muted_key_macros.contains(&trigger_vk) {
                    child.muted_key_macros.push(trigger_vk);
                }
            } else {
                child.muted_key_macros.retain(|&v| v != trigger_vk);
                if is_root_muted
                    && !child
                        .app_key_macros
                        .iter()
                        .any(|m| m.trigger_vk == trigger_vk)
                    && let Some(m) = global_macro
                {
                    child.app_key_macros.push(m);
                }
            }
        }
        save(engine);
        true
    });
    if changed == Some(true) {
        refresh_key_macro_list();
    }
}

/// Deletes a key macro rule from the selected profile and refreshes the UI list.
pub fn delete_key_macro(trigger_vk: u32) {
    let changed = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        let sel_id = pm.selected_id.clone();
        let macros = if sel_id == GLOBAL_PROFILE_ID {
            Some(&mut pm.global_profile.app_key_macros)
        } else {
            pm.get_profile_mut(&sel_id).map(|p| &mut p.app_key_macros)
        }?;
        macros.retain(|r| r.trigger_vk != trigger_vk);
        save(engine);
        Some(true)
    })
    .flatten();
    if changed == Some(true) {
        refresh_key_macro_list();
    }
}

/// Inserts or updates a key macro rule on the selected profile and refreshes the UI list.
pub fn save_key_macro(orig_vk: Option<u32>, trig_vk: u32, name: String, actions: Vec<MacroAction>) {
    let rule = KeyMacroRule::new(trig_vk, name.clone(), actions.clone());
    edit_selected_then(
        |_| {},
        |engine| {
            let pm = &mut engine.profile_manager;
            let sel_id = pm.selected_id.clone();
            if sel_id == GLOBAL_PROFILE_ID {
                if let Some(old_vk) = orig_vk {
                    pm.global_profile
                        .app_key_macros
                        .retain(|m| m.trigger_vk != old_vk);
                }
                pm.global_profile
                    .app_key_macros
                    .retain(|m| m.trigger_vk != trig_vk);
                pm.global_profile.app_key_macros.push(rule);
            } else {
                let is_global = pm
                    .global_profile
                    .app_key_macros
                    .iter()
                    .any(|m| m.trigger_vk == trig_vk);
                if let Some(child) = pm.get_profile_mut(&sel_id) {
                    if is_global {
                        child.muted_key_macros.retain(|&v| v != trig_vk);
                        child.overridden_key_macros.insert(trig_vk, (name, actions));
                    } else {
                        if let Some(old_vk) = orig_vk {
                            child.app_key_macros.retain(|m| m.trigger_vk != old_vk);
                        }
                        child.app_key_macros.retain(|m| m.trigger_vk != trig_vk);
                        child.app_key_macros.push(rule);
                    }
                }
            }
        },
    );
    refresh_key_macro_list();
}

/// Saves the switch hotkey (`None` = inherit global) for `target_profile_id` and refreshes UI.
pub fn save_switch_hotkey(target_profile_id: &str, is_root: bool, hotkey: Option<Hotkey>) {
    with_engine(|engine| {
        if let Some(hk) = hotkey {
            if is_root {
                engine.config_mut().switch_key = hk;
                engine.config_mut().switch_with_ctrl_shift = hk.is_ctrl_shift();
            } else if let Some(child) = engine.profile_manager.get_profile_mut(target_profile_id) {
                child.switch_key = Some(hk);
            }
        } else if !is_root
            && let Some(child) = engine.profile_manager.get_profile_mut(target_profile_id)
        {
            child.switch_key = None;
        }
        save(engine);
        if let Some(c) = super::controls() {
            let child = engine.profile_manager.get_profile(target_profile_id);
            c.tab_typing.load_profile(child, engine.config());
        }
    });
    refresh_keys_overview();
}

/// Saves the global target capture hotkey and refreshes UI.
pub fn save_target_capture_hotkey(hotkey: Hotkey) {
    with_engine(|engine| {
        engine.config_mut().target_capture_hotkey = hotkey;
        save(engine);
        if let Some(c) = super::controls() {
            let sel_id = engine.profile_manager.selected_id.clone();
            let child = engine.profile_manager.get_profile(&sel_id);
            c.tab_typing.load_profile(child, engine.config());
        }
    });
}

/// Saves the global target capture enabled state and persists config.
pub fn save_target_capture_enabled(enabled: bool) {
    with_engine(|engine| {
        engine.config_mut().target_capture_enabled = enabled;
        engine.refresh_effective_config();
        save(engine);
    });
    #[cfg(target_os = "windows")]
    crate::platform::win32::capture::set_target_capture_enabled(enabled);
}

/// Captures a target input field for the specified process:
/// - If a profile for this executable exists, adds the target rule to that profile.
/// - If no profile exists, automatically creates a new default child profile (inheriting from Global)
///   for this process and adds the target rule to it.
/// - Synchronizes active target rules flag, refreshes UI if open, and dispatches a Windows desktop notification.
pub fn capture_target_for_app(process_name: &str, target_id: &str) {
    let tid = target_id.trim().to_string();
    if tid.is_empty() {
        return;
    }

    let mut proc = process_name.trim().to_lowercase();
    if proc.is_empty() {
        proc = crate::platform::win32::app_detect::get_foreground_process_name();
    }
    if proc.is_empty() {
        return;
    }

    // Never auto-create profile for Kora / OpenKey itself
    let current_exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    if proc == current_exe || proc == "kora.exe" || proc == "openkey.exe" {
        return;
    }

    let outcome = with_engine(|engine| {
        let pm = &mut engine.profile_manager;
        if let Some(child) = pm.find_profile_by_process_mut(&proc) {
            let prof_id = child.id.clone();
            let app_name = child.name.clone();
            let already = child
                .target_rules
                .iter()
                .any(|r| r.target_id.eq_ignore_ascii_case(&tid));
            if !already {
                child
                    .target_rules
                    .push(crate::engine::profile::TargetRule::new(
                        tid.clone(),
                        crate::engine::profile::KeyInjectionStrategy::AutocompleteGuard,
                    ));
            }
            pm.active_id = prof_id.clone();
            pm.selected_id = prof_id.clone();
            save(engine);
            Some((prof_id, app_name, false))
        } else {
            // Auto-create new child profile inheriting from Global
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
            while id == GLOBAL_PROFILE_ID || pm.get_profile(&id).is_some() {
                id = format!("{}_{}", base_id, counter);
                counter += 1;
            }

            let raw_name = proc.trim_end_matches(".exe");
            let display_name = if let Some(first) = raw_name.chars().next() {
                format!("{}{}", first.to_uppercase(), &raw_name[first.len_utf8()..])
            } else {
                "App".to_string()
            };

            let mut new_profile =
                AppProfile::new_child(id.clone(), display_name.clone(), proc.clone(), "💻");
            new_profile
                .target_rules
                .push(crate::engine::profile::TargetRule::new(
                    tid.clone(),
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

    if let Some((prof_id, app_name, is_new)) = outcome {
        crate::platform::win32::app_tracker::sync_active_target_rules_flag();

        if let Some(c) = super::controls() {
            if let Some(ui_st) = super::ui() {
                crate::ui::views::control_panel::SidebarControls::switch_to_profile(
                    c,
                    ui_st.h_panel,
                    &prof_id,
                );
            } else {
                c.refresh_from_engine();
            }
        }

        // Dispatch Windows native notification
        let strings = crate::language::current();
        let message = if is_new {
            strings
                .notif_target_created_profile
                .replace("{app}", &app_name)
                .replace("{target}", &tid)
        } else {
            strings
                .notif_target_added_rule
                .replace("{app}", &app_name)
                .replace("{target}", &tid)
        };
        crate::ui::notification::notify_info("Kora", message);
    }
}

/// Adds a target rule to the active (or selected) child profile and refreshes the panel.
pub fn add_target_rule_to_selected(target_id: &str) {
    let proc = crate::platform::win32::app_detect::get_foreground_process_name();
    capture_target_for_app(&proc, target_id);
}

/// Updates the key injection strategy for a target rule in the selected child profile.
pub fn update_target_rule_strategy(
    target_id: &str,
    strategy: crate::engine::profile::KeyInjectionStrategy,
) {
    let tid = target_id.to_string();
    let updated = with_engine(|engine| {
        let sel_id = engine.profile_manager.selected_id.clone();
        if sel_id != GLOBAL_PROFILE_ID {
            if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                if let Some(r) = child
                    .target_rules
                    .iter_mut()
                    .find(|r| r.target_id.eq_ignore_ascii_case(&tid))
                {
                    r.strategy = strategy;
                }
            }
            save(engine);
            if let Some(c) = super::controls() {
                let child = engine.profile_manager.get_profile(&sel_id);
                c.tab_typing.load_profile(child, engine.config());
            }
            true
        } else {
            false
        }
    })
    .unwrap_or(false);

    if updated {
        crate::platform::win32::app_tracker::sync_active_target_rules_flag();
    }
}

/// Deletes a target rule from the selected child profile.
pub fn delete_target_rule(target_id: &str) {
    let tid = target_id.to_string();
    let updated = with_engine(|engine| {
        let sel_id = engine.profile_manager.selected_id.clone();
        if sel_id != GLOBAL_PROFILE_ID {
            if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                child
                    .target_rules
                    .retain(|r| !r.target_id.eq_ignore_ascii_case(&tid));
            }
            save(engine);
            if let Some(c) = super::controls() {
                let child = engine.profile_manager.get_profile(&sel_id);
                c.tab_typing.load_profile(child, engine.config());
            }
            true
        } else {
            false
        }
    })
    .unwrap_or(false);

    if updated {
        crate::platform::win32::app_tracker::sync_active_target_rules_flag();
    }
}

/// Updates the app-level default key injection strategy for the selected child profile.
pub fn update_app_key_injection_strategy(
    strategy: Option<crate::engine::profile::KeyInjectionStrategy>,
) {
    let updated = with_engine(|engine| {
        let sel_id = engine.profile_manager.selected_id.clone();
        if sel_id != GLOBAL_PROFILE_ID {
            if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                child.app_strategy = strategy;
            }
            save(engine);
            if let Some(c) = super::controls() {
                let child = engine.profile_manager.get_profile(&sel_id);
                c.tab_typing.load_profile(child, engine.config());
            }
            true
        } else {
            false
        }
    })
    .unwrap_or(false);

    if updated {
        crate::platform::win32::app_tracker::sync_active_target_rules_flag();
    }
}
