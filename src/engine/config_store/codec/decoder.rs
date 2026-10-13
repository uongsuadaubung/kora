//! INI Parser / Decoder for EngineConfig, MacroTable, and ProfileManager

use crate::engine::config::EngineConfig;
use crate::engine::config::hotkey::name_to_vk;
use crate::engine::macro_table::{MacroEntry, MacroTable, MacroType};
use crate::engine::profile::{
    AppProfile, KeyMacroRule, KeyRemapRule, LanguageMode, MacroAction, MethodMode, ProfileManager,
};
use std::collections::HashMap;

/// Parse INI content into EngineConfig and MacroTable (legacy backward compatibility)
pub fn parse_config_and_macros(content: &str) -> (EngineConfig, MacroTable) {
    let (config, macros, _) = parse_config_all(content);
    (config, macros)
}

/// Parse INI content into EngineConfig, MacroTable, and ProfileManager
pub fn parse_config_all(content: &str) -> (EngineConfig, MacroTable, ProfileManager) {
    let mut config = EngineConfig::default();
    let mut macros = MacroTable::new();
    let mut profiles = ProfileManager::new();

    let mut current_section = String::new();
    let mut child_profiles_map: HashMap<String, AppProfile> = HashMap::new();
    let mut child_order: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_lowercase();
            if let Some(p_id) = current_section.strip_prefix("profile:") {
                let id = p_id.trim().to_string();
                if !child_profiles_map.contains_key(&id) {
                    child_order.push(id.clone());
                    child_profiles_map.insert(
                        id.clone(),
                        AppProfile::new_child(id.clone(), id, "", "🎮"),
                    );
                }
            }
            continue;
        }

        let (key, val) = if let Some((k, v)) = line.split_once('=') {
            (k.trim(), v.trim())
        } else if let Some((k, v)) = line.split_once(':') {
            (k.trim(), v.trim())
        } else {
            continue;
        };

        if current_section == "macro" {
            if !key.is_empty() && !val.is_empty() {
                let (real_val, mtype) = if let Some((v, t)) = val.split_once(':') {
                    (v.trim(), MacroType::from(t))
                } else {
                    (val, MacroType::Normal)
                };
                if key.eq_ignore_ascii_case("muted") || key.eq_ignore_ascii_case("muted_macros") {
                    for m in val.split(',') {
                        let m_trim = m.trim();
                        if !m_trim.is_empty() && !profiles.global_profile.muted_macros.iter().any(|k| k.eq_ignore_ascii_case(m_trim)) {
                            profiles.global_profile.muted_macros.push(m_trim.to_string());
                        }
                    }
                } else {
                    macros.insert_typed_no_cache(key, real_val, mtype);
                }
            }
        } else if current_section == "remap" {
            let key_l = key.to_lowercase();
            if key_l == "enabled" || key_l == "remap_enabled" {
                let val_l = val.to_lowercase();
                let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";
                config.remap_enabled = val_bool;
                profiles.global_profile.remap_enabled = Some(val_bool);
            } else if key_l == "muted" || key_l == "muted_remaps" {
                for k in val.split(',') {
                    if let Some(vk) = name_to_vk(k.trim()) {
                        if !profiles.global_profile.muted_remaps.contains(&vk) {
                            profiles.global_profile.muted_remaps.push(vk);
                        }
                    }
                }
            } else if let (Some(f_vk), Some(t_vk)) = (name_to_vk(key), name_to_vk(val)) {
                let rule = KeyRemapRule::new(f_vk, t_vk);
                if !profiles.global_profile.app_remaps.iter().any(|r| r.from_vk == f_vk) {
                    profiles.global_profile.app_remaps.push(rule);
                }
            }
        } else if current_section == "disable" {
            let key_l = key.to_lowercase();
            if key_l == "enabled" || key_l == "disable_enabled" {
                let val_l = val.to_lowercase();
                let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";
                config.disable_enabled = val_bool;
                profiles.global_profile.disable_enabled = Some(val_bool);
            } else if key_l == "keys" || key_l == "disabled" {
                for k in val.split(',') {
                    if let Some(vk) = name_to_vk(k.trim()) {
                        if !profiles.global_profile.app_disabled.contains(&vk) {
                            profiles.global_profile.app_disabled.push(vk);
                        }
                    }
                }
            } else if key_l == "allow_disabled" || key_l == "unmuted_disabled" || key_l == "muted" {
                for k in val.split(',') {
                    if let Some(vk) = name_to_vk(k.trim()) {
                        if !profiles.global_profile.unmuted_disabled.contains(&vk) {
                            profiles.global_profile.unmuted_disabled.push(vk);
                        }
                    }
                }
            } else if let Some(vk) = name_to_vk(key) {
                let val_l = val.to_lowercase();
                let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";
                if val_bool && !profiles.global_profile.app_disabled.contains(&vk) {
                    profiles.global_profile.app_disabled.push(vk);
                }
            }
        } else if current_section == "key_macro" || current_section == "key_macros" {
            let key_l = key.to_lowercase();
            if key_l == "enabled" || key_l == "key_macro_enabled" {
                let val_l = val.to_lowercase();
                let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";
                config.key_macro_enabled = val_bool;
                profiles.global_profile.key_macro_enabled = Some(val_bool);
            } else if key_l == "muted" || key_l == "muted_key_macros" {
                for k in val.split(',') {
                    if let Some(vk) = name_to_vk(k.trim()) {
                        if !profiles.global_profile.muted_key_macros.contains(&vk) {
                            profiles.global_profile.muted_key_macros.push(vk);
                        }
                    }
                }
            } else if let Some(trig_vk) = name_to_vk(key) {
                let (name, dsl) = if let Some((n, d)) = val.split_once('|') {
                    (n.trim().to_string(), d.trim())
                } else {
                    (format!("Macro {}", key), val.trim())
                };
                let actions = MacroAction::dsl_to_actions(dsl);
                if !profiles.global_profile.app_key_macros.iter().any(|m| m.trigger_vk == trig_vk) {
                    profiles.global_profile.app_key_macros.push(KeyMacroRule::new(trig_vk, name, actions));
                }
            }
        } else if let Some(p_id) = current_section.strip_prefix("profile:") {
            let p_id = p_id.trim();
            if let Some(child) = child_profiles_map.get_mut(p_id) {
                let key_l = key.to_lowercase();
                let val_l = val.to_lowercase();
                let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";

                match key_l.as_str() {
                    "name" => child.name = val.to_string(),
                    "process" | "processes" | "process_names" => {
                        child.process_names = val
                            .split(',')
                            .map(|s| s.trim().to_lowercase())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    "icon" => child.icon = val.trim().to_string(),
                    "mode" => child.mode = LanguageMode::from_str(val),
                    "method" => child.method = MethodMode::from_str(val),
                    "switch_key" => child.switch_key = Some(crate::engine::config::Hotkey::from_config_str(val)),
                    "switch_key_enabled" => child.switch_key_enabled = Some(val_bool),
                    "restore_on_wrong" | "restore_on_wrong_spelling" => child.restore_on_wrong_spelling = Some(val_bool),
                    "auto_uppercase_first" | "auto_uppercase_first_char" => child.auto_uppercase_first_char = Some(val_bool),
                    "bracket_w" => child.bracket_w = Some(val_bool),
                    "remember_history" | "remember_history_across_space" => child.remember_history_across_space = Some(val_bool),
                    "sound_enabled" => child.sound_enabled = Some(val_bool),
                    "switch" | "sound_profile" => child.sound_profile = Some(std::sync::Arc::from(val)),
                    "volume" | "sound_volume" => child.sound_volume = Some(val_l.parse().unwrap_or(50).clamp(0, 100)),
                    "use_macro" => child.use_macro = Some(val_bool),
                    "use_macro_in_english" | "macro_in_english" => child.use_macro_in_english_mode = Some(val_bool),
                    "remap_enabled" | "enable_remap" => child.remap_enabled = Some(val_bool),
                    "disable_enabled" | "enable_disable" => child.disable_enabled = Some(val_bool),
                    "muted_macros" => {
                        for m in val.split(',') {
                            let m = m.trim().to_lowercase();
                            if !m.is_empty() && !child.muted_macros.contains(&m) {
                                child.muted_macros.push(m);
                            }
                        }
                    }
                    "override_remap" | "override_remaps" => {
                        if let Some((f, t)) = val.split_once(['=', ':']) {
                            if let (Some(f_vk), Some(t_vk)) = (name_to_vk(f.trim()), name_to_vk(t.trim())) {
                                child.overridden_remaps.insert(f_vk, t_vk);
                            }
                        }
                    }
                    "app_remap" | "app_remaps" => {
                        if let Some((f, t)) = val.split_once(['=', ':']) {
                            if let (Some(f_vk), Some(t_vk)) = (name_to_vk(f.trim()), name_to_vk(t.trim())) {
                                child.app_remaps.push(KeyRemapRule::new(f_vk, t_vk));
                            }
                        }
                    }
                    "muted_remaps" => {
                        for k in val.split(',') {
                            if let Some(vk) = name_to_vk(k.trim()) {
                                if !child.muted_remaps.contains(&vk) {
                                    child.muted_remaps.push(vk);
                                }
                            }
                        }
                    }
                    "allow_disabled" | "unmuted_disabled" => {
                        for k in val.split(',') {
                            if let Some(vk) = name_to_vk(k.trim()) {
                                if !child.unmuted_disabled.contains(&vk) {
                                    child.unmuted_disabled.push(vk);
                                }
                            }
                        }
                    }
                    "disable_keys" | "app_disabled" => {
                        for k in val.split(',') {
                            if let Some(vk) = name_to_vk(k.trim()) {
                                if !child.app_disabled.contains(&vk) {
                                    child.app_disabled.push(vk);
                                }
                            }
                        }
                    }
                    "override_macro" | "override_macros" => {
                        if let Some((k, v)) = val.split_once(['=', ':']) {
                            let macro_key = k.trim().to_lowercase();
                            let (real_val, mtype) = if let Some((real_v, t)) = v.split_once(':') {
                                (real_v.trim().to_string(), MacroType::from(t.trim()))
                            } else {
                                (v.trim().to_string(), MacroType::Normal)
                            };
                            child.overridden_macros.insert(macro_key, (real_val, mtype));
                        }
                    }
                    "app_macro" | "app_macros" => {
                        if let Some((k, v)) = val.split_once(['=', ':']) {
                            let macro_key = k.trim().to_string();
                            let (real_val, mtype) = if let Some((real_v, t)) = v.split_once(':') {
                                (real_v.trim().to_string(), MacroType::from(t.trim()))
                            } else {
                                (v.trim().to_string(), MacroType::Normal)
                            };
                            child.app_macros.push(MacroEntry {
                                key: macro_key,
                                value: real_val,
                                macro_type: mtype,
                            });
                        }
                    }
                    "key_macro_enabled" | "enable_key_macro" => child.key_macro_enabled = Some(val_bool),
                    "muted_key_macros" => {
                        for k in val.split(',') {
                            if let Some(vk) = name_to_vk(k.trim()) {
                                if !child.muted_key_macros.contains(&vk) {
                                    child.muted_key_macros.push(vk);
                                }
                            }
                        }
                    }
                    "override_key_macro" | "override_key_macros" => {
                        if let Some((k, v)) = val.split_once(['=', ':']) {
                            if let Some(trig_vk) = name_to_vk(k.trim()) {
                                let (name, dsl) = if let Some((n, d)) = v.split_once('|') {
                                    (n.trim().to_string(), d.trim())
                                } else {
                                    (format!("Macro {}", k.trim()), v.trim())
                                };
                                let actions = MacroAction::dsl_to_actions(dsl);
                                child.overridden_key_macros.insert(trig_vk, (name, actions));
                            }
                        }
                    }
                    "app_key_macro" | "app_key_macros" => {
                        if let Some((k, v)) = val.split_once(['=', ':']) {
                            if let Some(trig_vk) = name_to_vk(k.trim()) {
                                let (name, dsl) = if let Some((n, d)) = v.split_once('|') {
                                    (n.trim().to_string(), d.trim())
                                } else {
                                    (format!("Macro {}", k.trim()), v.trim())
                                };
                                let actions = MacroAction::dsl_to_actions(dsl);
                                child.app_key_macros.push(KeyMacroRule::new(trig_vk, name, actions));
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            let key_l = key.to_lowercase();
            let val_l = val.to_lowercase();
            let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";

            match (current_section.as_str(), key_l.as_str()) {
                ("key_macro", "enabled") | (_, "key_macro_enabled") => {
                    config.key_macro_enabled = val_bool;
                }
                ("sound", "enabled") | (_, "sound_enabled") | (_, "sound") => {
                    config.sound_enabled = val_bool;
                }
                (_, "method") => {
                    config.method = val_l.parse().unwrap_or_default();
                }
                (_, "enabled") => config.enabled = val_bool,
                (_, "switch_key_enabled") | (_, "use_switch_key") | (_, "switch_enabled") => {
                    config.switch_key_enabled = val_bool;
                }
                (_, "switch_key") | (_, "switch_with_ctrl_shift") => {
                    config.switch_key = crate::engine::config::Hotkey::from_config_str(val);
                    config.switch_with_ctrl_shift = config.switch_key.is_ctrl_shift();
                }
                (_, "restore_on_wrong") | (_, "restore_on_wrong_spelling") => {
                    config.restore_on_wrong_spelling = val_bool;
                }
                (_, "auto_uppercase_first") | (_, "auto_uppercase_first_char") => {
                    config.auto_uppercase_first_char = val_bool;
                }
                (_, "use_macro") => config.use_macro = val_bool,
                (_, "bracket_w") => config.bracket_w = val_bool,
                (_, "remember_history") | (_, "remember_history_across_space") => {
                    config.remember_history_across_space = val_bool;
                }
                (_, "show_dialog_on_startup") | (_, "show_dialog") => {
                    config.show_dialog_on_startup = val_bool;
                }
                (_, "debug") => config.debug = val_bool,
                (_, "theme") => {
                    config.theme = val_l.parse().unwrap_or_default();
                }
                (_, "language") | (_, "lang") => {
                    config.language = val_l.parse().unwrap_or_default();
                }
                (_, "sound_profile") | (_, "switch") | (_, "switch_type") => {
                    config.sound_profile = std::sync::Arc::from(val.trim());
                }
                (_, "sound_volume") | (_, "volume") => {
                    config.sound_volume = val_l.parse().unwrap_or(50).clamp(0, 100);
                }
                (_, "remap_enabled") => {
                    config.remap_enabled = val_bool;
                    profiles.global_profile.remap_enabled = Some(val_bool);
                }
                (_, "disable_enabled") => {
                    config.disable_enabled = val_bool;
                    profiles.global_profile.disable_enabled = Some(val_bool);
                }
                _ => {}
            }
        }
    }

    // Attach child profiles in original order
    for id in child_order {
        if let Some(child) = child_profiles_map.remove(&id) {
            profiles.add_profile(child);
        }
    }

    macros.rebuild_cache();
    (config, macros, profiles)
}

