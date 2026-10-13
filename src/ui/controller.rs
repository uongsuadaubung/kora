//! Thin controller between the views and the engine.
//!
//! Every UI edit goes through here so that locking, root/child-profile dispatch and
//! persistence live in one place instead of being copy-pasted into each tab/dialog.
//!
//! Rule: the closures run with `ENGINE_INSTANCE` locked. They must not call anything
//! that pumps messages (MessageBox, dialogs, TrackPopupMenu, SendMessage to another
//! thread): the keyboard hook runs on the same thread and would self-deadlock.

use super::GLOBAL_PROFILE_ID;
use crate::engine::profile::{
    AppProfile, ResolvedDisableItem, ResolvedKeyMacroItem, ResolvedMacroItem, ResolvedRemapItem,
};
use crate::engine::{EngineConfig, VietnameseEngine};
use crate::platform::win32::ENGINE_INSTANCE;

/// The settings object an edit applies to.
pub enum EditTarget<'a> {
    /// Root profile selected: edit the global config.
    Global(&'a mut EngineConfig),
    /// Child profile selected: edit its overrides.
    Child(&'a mut AppProfile),
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
    crate::platform::win32::sound::reconfigure_sound(eff.sound_enabled, &eff.sound_profile, eff.sound_volume);
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
    if let (Some(items), Some(c)) = (with_engine(|e| selected_disable_items(e)), super::controls()) {
        c.tab_keys.populate_disabled(&items);
        refresh_keys_overview();
    }
}

/// Re-populates the key macro list from the engine (call without holding the engine lock).
pub fn refresh_key_macro_list() {
    if let (Some(items), Some(c)) = (with_engine(|e| selected_key_macro_items(e)), super::controls()) {
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
