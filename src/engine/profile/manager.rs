//! Profile Manager implementing Live Overlay Inheritance (Cách 2)
//!
//! Global Profile is the Single Source of Truth.
//! Child profiles only store deltas.
//! Resolution methods compute effective rules in real time.

use super::model::*;
use crate::engine::config::EngineConfig;
use crate::engine::macro_table::MacroEntry;
use std::collections::HashMap;

/// System-wide Profile Manager
#[derive(Debug, Clone)]
pub struct ProfileManager {
    pub global_profile: AppProfile,
    pub profiles: Vec<AppProfile>,
    pub active_id: String,
    pub selected_id: String,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileManager {
    pub fn new() -> Self {
        Self {
            global_profile: AppProfile::new_global(),
            profiles: Vec::new(),
            active_id: "global".to_string(),
            selected_id: "global".to_string(),
        }
    }

    /// Returns a list of all profiles starting with Global
    pub fn all_profiles(&self) -> Vec<&AppProfile> {
        let mut list = Vec::with_capacity(1 + self.profiles.len());
        list.push(&self.global_profile);
        for p in &self.profiles {
            list.push(p);
        }
        list
    }

    /// Gets a profile by ID
    pub fn get_profile(&self, id: &str) -> Option<&AppProfile> {
        if id == "global" {
            Some(&self.global_profile)
        } else {
            self.profiles.iter().find(|p| p.id == id)
        }
    }

    /// Gets a mutable profile by ID
    pub fn get_profile_mut(&mut self, id: &str) -> Option<&mut AppProfile> {
        if id == "global" {
            Some(&mut self.global_profile)
        } else {
            self.profiles.iter_mut().find(|p| p.id == id)
        }
    }

    /// Adds or updates a child profile
    pub fn add_profile(&mut self, profile: AppProfile) {
        if profile.is_root || profile.id == "global" {
            return;
        }
        if let Some(pos) = self.profiles.iter().position(|p| p.id == profile.id) {
            self.profiles[pos] = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    /// Removes a child profile by ID. Global cannot be removed.
    pub fn remove_profile(&mut self, id: &str) -> bool {
        if id == "global" {
            return false;
        }
        if let Some(pos) = self.profiles.iter().position(|p| p.id == id) {
            self.profiles.remove(pos);
            if self.active_id == id {
                self.active_id = "global".to_string();
            }
            if self.selected_id == id {
                self.selected_id = "global".to_string();
            }
            true
        } else {
            false
        }
    }

    /// Finds a profile matching the executable filename (e.g. "cs2.exe")
    pub fn find_profile_by_process(&self, exe_name: &str) -> Option<&AppProfile> {
        self.profiles.iter().find(|p| p.matches_process(exe_name))
    }

    /// Switches active profile based on the foreground application executable name.
    /// Returns (has_changed, active_profile_id).
    pub fn set_active_by_process(&mut self, exe_name: &str) -> (bool, String) {
        let new_id = if let Some(p) = self.find_profile_by_process(exe_name) {
            p.id.clone()
        } else {
            "global".to_string()
        };

        if self.active_id != new_id {
            self.active_id = new_id.clone();
            (true, new_id)
        } else {
            (false, new_id)
        }
    }

    /// Computes effective EngineConfig for a profile based on Live Inheritance
    pub fn resolve_effective_config(&self, profile_id: &str, base: &EngineConfig) -> EngineConfig {
        let mut cfg = base.clone();
        if profile_id == "global" {
            return cfg;
        }

        if let Some(child) = self.get_profile(profile_id) {
            match child.mode {
                LanguageMode::Vietnamese => cfg.enabled = true,
                LanguageMode::English => cfg.enabled = false,
                LanguageMode::Inherit => {}
            }

            if let MethodMode::Specific(m) = child.method {
                cfg.method = m;
            }

            if let Some(sw) = child.switch_key {
                cfg.switch_key = sw;
                cfg.switch_with_ctrl_shift = sw.is_ctrl_shift();
            }
            if let Some(en) = child.switch_key_enabled {
                cfg.switch_key_enabled = en;
            }
            if let Some(r) = child.restore_on_wrong_spelling {
                cfg.restore_on_wrong_spelling = r;
            }
            if let Some(u) = child.auto_uppercase_first_char {
                cfg.auto_uppercase_first_char = u;
            }
            if let Some(b) = child.bracket_w {
                cfg.bracket_w = b;
            }
            if let Some(h) = child.remember_history_across_space {
                cfg.remember_history_across_space = h;
            }
            if let Some(s) = child.sound_enabled {
                cfg.sound_enabled = s;
            }
            if let Some(ref sp) = child.sound_profile {
                cfg.sound_profile = sp.clone();
            }
            if let Some(v) = child.sound_volume {
                cfg.sound_volume = v;
            }
            if let Some(m) = child.use_macro {
                cfg.use_macro = m;
            }
            if let Some(me) = child.use_macro_in_english_mode {
                cfg.use_macro_in_english_mode = me;
            }
            if let Some(r) = child.remap_enabled {
                cfg.remap_enabled = r;
            }
            if let Some(d) = child.disable_enabled {
                cfg.disable_enabled = d;
            }
            if let Some(km) = child.key_macro_enabled {
                cfg.key_macro_enabled = km;
            }
        }

        cfg
    }

    /// Resolves effective active KeyRemapRules for the hook execution
    /// Formula: (Global \ Muted) U Overridden U AppSpecific
    pub fn resolve_effective_remaps(&self, profile_id: &str) -> Vec<KeyRemapRule> {
        let global_enabled = self.global_profile.remap_enabled.unwrap_or(false);
        let global_remaps = &self.global_profile.app_remaps;
        if profile_id == "global" {
            if !global_enabled {
                return Vec::new();
            }
            return global_remaps
                .iter()
                .filter(|r| !self.global_profile.muted_remaps.contains(&r.from_vk))
                .cloned()
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => {
                if !global_enabled {
                    return Vec::new();
                }
                return global_remaps
                    .iter()
                    .filter(|r| !self.global_profile.muted_remaps.contains(&r.from_vk))
                    .cloned()
                    .collect();
            }
        };

        let is_enabled = child.remap_enabled.unwrap_or(global_enabled);
        if !is_enabled {
            return Vec::new();
        }

        let mut result = Vec::new();

        // 1. Process Global rules with overrides and mutes
        for rule in global_remaps {
            if child.app_remaps.iter().any(|ar| ar.from_vk == rule.from_vk) {
                continue; // Handled as AppSpecific in step 2
            }
            if child.muted_remaps.contains(&rule.from_vk) {
                // Tombstone: muted in child
                continue;
            }
            if let Some(&overridden_to_vk) = child.overridden_remaps.get(&rule.from_vk) {
                result.push(KeyRemapRule::new(rule.from_vk, overridden_to_vk));
            } else if !self.global_profile.muted_remaps.contains(&rule.from_vk) {
                result.push(rule.clone());
            }
        }

        // 2. Add app-specific rules
        for rule in &child.app_remaps {
            if child.muted_remaps.contains(&rule.from_vk) {
                continue;
            }
            // Avoid duplicate from_vk
            if !result.iter().any(|r| r.from_vk == rule.from_vk) {
                result.push(rule.clone());
            }
        }

        result
    }

    /// Resolves effective disabled keys (VK codes) for hook execution
    /// Formula: (Global \ Unmuted) U AppDisabled
    pub fn resolve_effective_disabled(&self, profile_id: &str) -> Vec<u32> {
        let global_enabled = self.global_profile.disable_enabled.unwrap_or(false);
        let global_disabled = &self.global_profile.app_disabled;
        if profile_id == "global" {
            if !global_enabled {
                return Vec::new();
            }
            return global_disabled
                .iter()
                .filter(|&&vk| !self.global_profile.unmuted_disabled.contains(&vk))
                .copied()
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => {
                if !global_enabled {
                    return Vec::new();
                }
                return global_disabled
                    .iter()
                    .filter(|&&vk| !self.global_profile.unmuted_disabled.contains(&vk))
                    .copied()
                    .collect();
            }
        };

        let is_enabled = child.disable_enabled.unwrap_or(global_enabled);
        if !is_enabled {
            return Vec::new();
        }

        let mut result = Vec::new();

        // 1. Global disabled keys unless unmuted by root or unmuted by child
        for &vk in global_disabled {
            if !self.global_profile.unmuted_disabled.contains(&vk) && !child.unmuted_disabled.contains(&vk) {
                result.push(vk);
            }
        }

        // 2. App-specific disabled keys
        for &vk in &child.app_disabled {
            if child.unmuted_disabled.contains(&vk) {
                continue;
            }
            if !result.contains(&vk) {
                result.push(vk);
            }
        }

        result
    }

    /// Resolves effective macro entries for engine typing
    /// Formula: (Global \ Muted) U Overridden U AppSpecific
    pub fn resolve_effective_macros(&self, profile_id: &str, global_macros: &[MacroEntry]) -> Vec<MacroEntry> {
        if profile_id == "global" {
            return global_macros
                .iter()
                .filter(|entry| !self.global_profile.muted_macros.iter().any(|m| m.eq_ignore_ascii_case(&entry.key)))
                .cloned()
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => return global_macros
                .iter()
                .filter(|entry| !self.global_profile.muted_macros.iter().any(|m| m.eq_ignore_ascii_case(&entry.key)))
                .cloned()
                .collect(),
        };

        let mut map: HashMap<String, MacroEntry> = HashMap::new();

        // 1. Global macros
        for entry in global_macros {
            if child.muted_macros.iter().any(|m| m.eq_ignore_ascii_case(&entry.key)) {
                continue; // Muted in child
            }
            if let Some((over_val, over_type)) = child.overridden_macros.get(&entry.key.to_lowercase()) {
                map.insert(
                    entry.key.to_lowercase(),
                    MacroEntry {
                        key: entry.key.clone(),
                        value: over_val.clone(),
                        macro_type: *over_type,
                    },
                );
            } else if !self.global_profile.muted_macros.iter().any(|m| m.eq_ignore_ascii_case(&entry.key)) {
                map.insert(entry.key.to_lowercase(), entry.clone());
            }
        }

        // 2. Child app-specific macros
        for entry in &child.app_macros {
            if child.muted_macros.iter().any(|m| m.eq_ignore_ascii_case(&entry.key)) {
                continue;
            }
            map.insert(entry.key.to_lowercase(), entry.clone());
        }

        let mut res: Vec<MacroEntry> = map.into_values().collect();
        res.sort_by(|a, b| a.key.cmp(&b.key));
        res
    }

    /// Resolves effective active KeyMacroRules for the hook execution
    /// Formula: (Global \ Muted) U Overridden U AppSpecific
    pub fn resolve_effective_key_macros(&self, profile_id: &str) -> Vec<KeyMacroRule> {
        let global_enabled = self.global_profile.key_macro_enabled.unwrap_or(false);
        let global_macros = &self.global_profile.app_key_macros;
        if profile_id == "global" {
            if !global_enabled {
                return Vec::new();
            }
            return global_macros
                .iter()
                .filter(|m| !self.global_profile.muted_key_macros.contains(&m.trigger_vk))
                .cloned()
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => {
                if !global_enabled {
                    return Vec::new();
                }
                return global_macros
                    .iter()
                    .filter(|m| !self.global_profile.muted_key_macros.contains(&m.trigger_vk))
                    .cloned()
                    .collect();
            }
        };

        let is_enabled = child.key_macro_enabled.unwrap_or(global_enabled);
        if !is_enabled {
            return Vec::new();
        }

        let mut result = Vec::new();

        // 1. Process Global rules with overrides and mutes
        for rule in global_macros {
            if child.muted_key_macros.contains(&rule.trigger_vk) {
                continue; // Tombstone: muted in child
            }
            if let Some((overridden_name, overridden_actions)) = child.overridden_key_macros.get(&rule.trigger_vk) {
                result.push(KeyMacroRule::with_trigger_name(
                    rule.trigger_vk,
                    rule.trigger_name.clone(),
                    overridden_name.clone(),
                    overridden_actions.clone(),
                ));
            } else if !self.global_profile.muted_key_macros.contains(&rule.trigger_vk) {
                result.push(rule.clone());
            }
        }

        // 2. Add app-specific rules
        for rule in &child.app_key_macros {
            if child.muted_key_macros.contains(&rule.trigger_vk) {
                continue;
            }
            if !result.iter().any(|r| r.trigger_vk == rule.trigger_vk) {
                result.push(rule.clone());
            }
        }

        result
    }

    // =========================================================================
    // UI Display Resolvers (Providing exact 4-column ListView items with badges)
    // =========================================================================

    /// Builds UI list items for Key Remapping table
    pub fn resolve_remap_items_for_ui(&self, profile_id: &str) -> Vec<ResolvedRemapItem> {
        let global_remaps = &self.global_profile.app_remaps;
        if profile_id == "global" || self.get_profile(profile_id).is_some_and(|p| p.is_root) {
            return global_remaps
                .iter()
                .map(|r| {
                    let is_muted = self.global_profile.muted_remaps.contains(&r.from_vk);
                    ResolvedRemapItem {
                        from_vk: r.from_vk,
                        to_vk: r.to_vk,
                        from_name: r.from_name.clone(),
                        to_name: r.to_name.clone(),
                        origin: RuleOrigin::Root,
                        is_active: !is_muted,
                    }
                })
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut items = Vec::new();

        // 1. All global items with their child state
        for r in global_remaps {
            if child.app_remaps.iter().any(|ar| ar.from_vk == r.from_vk) {
                continue; // Handled as AppSpecific in step 2
            }
            if child.muted_remaps.contains(&r.from_vk) {
                items.push(ResolvedRemapItem {
                    from_vk: r.from_vk,
                    to_vk: r.to_vk,
                    from_name: r.from_name.clone(),
                    to_name: r.to_name.clone(),
                    origin: RuleOrigin::MutedGlobal,
                    is_active: false,
                });
            } else if let Some(&over_to_vk) = child.overridden_remaps.get(&r.from_vk) {
                items.push(ResolvedRemapItem {
                    from_vk: r.from_vk,
                    to_vk: over_to_vk,
                    from_name: r.from_name.clone(),
                    to_name: crate::engine::config::hotkey::vk_to_name(over_to_vk).to_string(),
                    origin: RuleOrigin::OverriddenGlobal,
                    is_active: true,
                });
            } else if self.global_profile.muted_remaps.contains(&r.from_vk) {
                items.push(ResolvedRemapItem {
                    from_vk: r.from_vk,
                    to_vk: r.to_vk,
                    from_name: r.from_name.clone(),
                    to_name: r.to_name.clone(),
                    origin: RuleOrigin::MutedGlobal,
                    is_active: false,
                });
            } else {
                items.push(ResolvedRemapItem {
                    from_vk: r.from_vk,
                    to_vk: r.to_vk,
                    from_name: r.from_name.clone(),
                    to_name: r.to_name.clone(),
                    origin: RuleOrigin::InheritedGlobal,
                    is_active: true,
                });
            }
        }

        // 2. Child app-specific items
        for r in &child.app_remaps {
            let is_muted = child.muted_remaps.contains(&r.from_vk);
            items.push(ResolvedRemapItem {
                from_vk: r.from_vk,
                to_vk: r.to_vk,
                from_name: r.from_name.clone(),
                to_name: r.to_name.clone(),
                origin: RuleOrigin::AppSpecific,
                is_active: !is_muted,
            });
        }

        items
    }

    /// Builds UI list items for Key Disabling table
    pub fn resolve_disable_items_for_ui(&self, profile_id: &str) -> Vec<ResolvedDisableItem> {
        let global_disabled = &self.global_profile.app_disabled;
        if profile_id == "global" || self.get_profile(profile_id).is_some_and(|p| p.is_root) {
            return global_disabled
                .iter()
                .map(|&vk| {
                    let is_muted = self.global_profile.unmuted_disabled.contains(&vk);
                    ResolvedDisableItem {
                        vk,
                        key_name: crate::engine::config::hotkey::vk_to_name(vk).to_string(),
                        origin: RuleOrigin::Root,
                        is_active: !is_muted,
                    }
                })
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut items = Vec::new();

        // 1. All global disabled items with child status
        for &vk in global_disabled {
            if child.app_disabled.contains(&vk) {
                continue; // Handled as AppSpecific in step 2
            }
            if child.unmuted_disabled.contains(&vk) || self.global_profile.unmuted_disabled.contains(&vk) {
                items.push(ResolvedDisableItem {
                    vk,
                    key_name: crate::engine::config::hotkey::vk_to_name(vk).to_string(),
                    origin: RuleOrigin::MutedGlobal,
                    is_active: false,
                });
            } else {
                items.push(ResolvedDisableItem {
                    vk,
                    key_name: crate::engine::config::hotkey::vk_to_name(vk).to_string(),
                    origin: RuleOrigin::InheritedGlobal,
                    is_active: true,
                });
            }
        }

        // 2. Child app-specific disabled items
        for &vk in &child.app_disabled {
            let is_muted = child.unmuted_disabled.contains(&vk);
            items.push(ResolvedDisableItem {
                vk,
                key_name: crate::engine::config::hotkey::vk_to_name(vk).to_string(),
                origin: RuleOrigin::AppSpecific,
                is_active: !is_muted,
            });
        }

        items
    }

    /// Builds UI list items for Macro table
    pub fn resolve_macro_items_for_ui(&self, profile_id: &str, global_macros: &[MacroEntry]) -> Vec<ResolvedMacroItem> {
        if profile_id == "global" || self.get_profile(profile_id).is_some_and(|p| p.is_root) {
            return global_macros
                .iter()
                .map(|m| {
                    let is_muted = self.global_profile.muted_macros.iter().any(|k| k.eq_ignore_ascii_case(&m.key));
                    ResolvedMacroItem {
                        key: m.key.clone(),
                        value: m.value.clone(),
                        macro_type: m.macro_type,
                        origin: RuleOrigin::Root,
                        is_active: !is_muted,
                    }
                })
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut items = Vec::new();

        // 1. Global macros with child status
        for m in global_macros {
            if child.app_macros.iter().any(|am| am.key.eq_ignore_ascii_case(&m.key)) {
                continue; // Handled as AppSpecific in step 2
            }
            if child.muted_macros.iter().any(|k| k.eq_ignore_ascii_case(&m.key)) {
                items.push(ResolvedMacroItem {
                    key: m.key.clone(),
                    value: m.value.clone(),
                    macro_type: m.macro_type,
                    origin: RuleOrigin::MutedGlobal,
                    is_active: false,
                });
            } else if let Some((over_val, over_type)) = child.overridden_macros.get(&m.key.to_lowercase()) {
                items.push(ResolvedMacroItem {
                    key: m.key.clone(),
                    value: over_val.clone(),
                    macro_type: *over_type,
                    origin: RuleOrigin::OverriddenGlobal,
                    is_active: true,
                });
            } else if self.global_profile.muted_macros.iter().any(|k| k.eq_ignore_ascii_case(&m.key)) {
                items.push(ResolvedMacroItem {
                    key: m.key.clone(),
                    value: m.value.clone(),
                    macro_type: m.macro_type,
                    origin: RuleOrigin::MutedGlobal,
                    is_active: false,
                });
            } else {
                items.push(ResolvedMacroItem {
                    key: m.key.clone(),
                    value: m.value.clone(),
                    macro_type: m.macro_type,
                    origin: RuleOrigin::InheritedGlobal,
                    is_active: true,
                });
            }
        }

        // 2. Child app-specific macros
        for m in &child.app_macros {
            let is_muted = child.muted_macros.iter().any(|k| k.eq_ignore_ascii_case(&m.key));
            items.push(ResolvedMacroItem {
                key: m.key.clone(),
                value: m.value.clone(),
                macro_type: m.macro_type,
                origin: RuleOrigin::AppSpecific,
                is_active: !is_muted,
            });
        }

        items.sort_by(|a, b| a.key.cmp(&b.key));
        items
    }

    /// Builds UI list items for Key Macro table
    pub fn resolve_key_macro_items_for_ui(&self, profile_id: &str) -> Vec<ResolvedKeyMacroItem> {
        let global_macros = &self.global_profile.app_key_macros;
        if profile_id == "global" || self.get_profile(profile_id).is_some_and(|p| p.is_root) {
            return global_macros
                .iter()
                .map(|m| {
                    let is_muted = self.global_profile.muted_key_macros.contains(&m.trigger_vk);
                    ResolvedKeyMacroItem {
                        trigger_vk: m.trigger_vk,
                        trigger_name: m.trigger_name.clone(),
                        name: m.name.clone(),
                        actions: m.actions.clone(),
                        origin: RuleOrigin::Root,
                        is_active: !is_muted,
                    }
                })
                .collect();
        }

        let child = match self.get_profile(profile_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut items = Vec::new();

        // 1. All global items with their child state
        for m in global_macros {
            if child.app_key_macros.iter().any(|akm| akm.trigger_vk == m.trigger_vk) {
                continue; // Handled as AppSpecific in step 2
            }
            if child.muted_key_macros.contains(&m.trigger_vk) {
                items.push(ResolvedKeyMacroItem {
                    trigger_vk: m.trigger_vk,
                    trigger_name: m.trigger_name.clone(),
                    name: m.name.clone(),
                    actions: m.actions.clone(),
                    origin: RuleOrigin::MutedGlobal,
                    is_active: false,
                });
            } else if let Some((over_name, over_actions)) = child.overridden_key_macros.get(&m.trigger_vk) {
                items.push(ResolvedKeyMacroItem {
                    trigger_vk: m.trigger_vk,
                    trigger_name: m.trigger_name.clone(),
                    name: over_name.clone(),
                    actions: over_actions.clone(),
                    origin: RuleOrigin::OverriddenGlobal,
                    is_active: true,
                });
            } else if self.global_profile.muted_key_macros.contains(&m.trigger_vk) {
                items.push(ResolvedKeyMacroItem {
                    trigger_vk: m.trigger_vk,
                    trigger_name: m.trigger_name.clone(),
                    name: m.name.clone(),
                    actions: m.actions.clone(),
                    origin: RuleOrigin::MutedGlobal,
                    is_active: false,
                });
            } else {
                items.push(ResolvedKeyMacroItem {
                    trigger_vk: m.trigger_vk,
                    trigger_name: m.trigger_name.clone(),
                    name: m.name.clone(),
                    actions: m.actions.clone(),
                    origin: RuleOrigin::InheritedGlobal,
                    is_active: true,
                });
            }
        }

        // 2. Child app-specific macros
        for m in &child.app_key_macros {
            let is_muted = child.muted_key_macros.contains(&m.trigger_vk);
            items.push(ResolvedKeyMacroItem {
                trigger_vk: m.trigger_vk,
                trigger_name: m.trigger_name.clone(),
                name: m.name.clone(),
                actions: m.actions.clone(),
                origin: RuleOrigin::AppSpecific,
                is_active: !is_muted,
            });
        }

        items
    }
}

