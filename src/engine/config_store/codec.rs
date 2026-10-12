//! INI Parser and Serializer for EngineConfig, MacroTable, and ProfileManager
//! Supports Live Inheritance format with [remap], [disable], and [profile:<id>] sections.

use crate::engine::config::EngineConfig;
use crate::engine::config::hotkey::name_to_vk;
use crate::engine::macro_table::{MacroEntry, MacroTable, MacroType};
use crate::engine::profile::{
    AppProfile, KeyRemapRule, LanguageMode, MethodMode, ProfileManager,
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
                macros.insert_typed_no_cache(key, real_val, mtype);
            }
        } else if current_section == "remap" {
            let key_l = key.to_lowercase();
            if key_l == "enabled" || key_l == "remap_enabled" {
                let val_l = val.to_lowercase();
                let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";
                config.remap_enabled = val_bool;
                profiles.global_profile.remap_enabled = Some(val_bool);
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
            } else if let Some(vk) = name_to_vk(key) {
                let val_l = val.to_lowercase();
                let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";
                if val_bool && !profiles.global_profile.app_disabled.contains(&vk) {
                    profiles.global_profile.app_disabled.push(vk);
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
                    _ => {}
                }
            }
        } else {
            let key_l = key.to_lowercase();
            let val_l = val.to_lowercase();
            let val_bool = val_l == "true" || val_l == "1" || val_l == "yes";

            match (current_section.as_str(), key_l.as_str()) {
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

/// Serializes EngineConfig and MacroTable into unified INI string format
pub fn serialize_config_and_macros(config: &EngineConfig, macros: &MacroTable) -> String {
    let dummy_profiles = ProfileManager::new();
    serialize_config_all(config, macros, &dummy_profiles)
}

/// Serializes EngineConfig, MacroTable, and ProfileManager into complete INI string format
pub fn serialize_config_all(
    config: &EngineConfig,
    macros: &MacroTable,
    profiles: &ProfileManager,
) -> String {
    let mut out = String::from(
        "# Cấu hình Kora - Modern Input & Keyboard Suite\n# Đường dẫn: ~/.config/kora/config.ini\n\n",
    );

    out.push_str("[general]\n");
    out.push_str(&format!("method = {}\n", config.method));
    out.push_str(&format!("enabled = {}\n", config.enabled));
    out.push_str(&format!(
        "switch_key_enabled = {}\n",
        config.switch_key_enabled
    ));
    out.push_str(&format!(
        "switch_key = {}\n\n",
        config.switch_key.to_config_str()
    ));

    out.push_str("[spelling]\n");
    out.push_str(&format!(
        "restore_on_wrong = {}\n",
        config.restore_on_wrong_spelling
    ));
    out.push_str(&format!(
        "auto_uppercase_first = {}\n\n",
        config.auto_uppercase_first_char
    ));

    out.push_str("[features]\n");
    out.push_str(&format!("use_macro = {}\n", config.use_macro));
    out.push_str(&format!("bracket_w = {}\n", config.bracket_w));
    out.push_str(&format!(
        "remember_history = {}\n\n",
        config.remember_history_across_space
    ));

    out.push_str("[sound]\n");
    out.push_str(&format!("enabled = {}\n", config.sound_enabled));
    out.push_str(&format!("switch = {}\n", config.sound_profile));
    out.push_str(&format!("volume = {}\n\n", config.sound_volume));

    out.push_str("[system]\n");
    out.push_str(&format!("language = {}\n", config.language));
    out.push_str(&format!(
        "show_dialog_on_startup = {}\n",
        config.show_dialog_on_startup
    ));
    out.push_str(&format!("debug = {}\n", config.debug));
    out.push_str(&format!("theme = {}\n\n", config.theme));

    out.push_str("[macro]\n");
    out.push_str(
        "# Danh sách từ gõ tắt: <từ viết tắt> = <cụm từ thay thế>[:loại (normal/start/end)]\n",
    );
    for entry in macros.get_sorted_entries() {
        if entry.macro_type == MacroType::Normal {
            out.push_str(&format!("{} = {}\n", entry.key, entry.value));
        } else {
            out.push_str(&format!(
                "{} = {}:{}\n",
                entry.key,
                entry.value,
                entry.macro_type.as_str()
            ));
        }
    }
    out.push('\n');

    // [remap]
    out.push_str("[remap]\n");
    out.push_str(&format!("enabled = {}\n", config.remap_enabled));
    if !profiles.global_profile.app_remaps.is_empty() {
        out.push_str("# Đổi phím toàn hệ thống: <Phím gốc> = <Phím đích>\n");
        for r in &profiles.global_profile.app_remaps {
            let from_str = if !r.from_name.is_empty() {
                &r.from_name
            } else {
                "Key"
            };
            let to_str = if !r.to_name.is_empty() {
                &r.to_name
            } else {
                "Key"
            };
            out.push_str(&format!("{} = {}\n", from_str, to_str));
        }
    }
    out.push('\n');

    // [disable]
    out.push_str("[disable]\n");
    out.push_str(&format!("enabled = {}\n", config.disable_enabled));
    if !profiles.global_profile.app_disabled.is_empty() {
        out.push_str("# Phím vô hiệu hóa toàn hệ thống (cách nhau bởi dấu phẩy)\n");
        let names: Vec<String> = profiles
            .global_profile
            .app_disabled
            .iter()
            .map(|&vk| crate::engine::config::hotkey::vk_to_name(vk).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        out.push_str(&format!("keys = {}\n", names.join(", ")));
    }
    out.push('\n');

    // Child Profiles
    for child in &profiles.profiles {
        out.push_str(&format!("[profile:{}]\n", child.id));
        out.push_str(&format!("name = {}\n", child.name));
        if !child.process_names.is_empty() {
            out.push_str(&format!("process = {}\n", child.process_names.join(", ")));
        }
        out.push_str(&format!("icon = {}\n", child.icon));
        if child.mode != LanguageMode::Inherit {
            out.push_str(&format!("mode = {}\n", child.mode.as_str()));
        }
        if child.method != MethodMode::Inherit {
            out.push_str(&format!("method = {}\n", child.method.as_str()));
        }
        if let Some(sw) = child.switch_key {
            out.push_str(&format!("switch_key = {}\n", sw.to_config_str()));
        }
        if let Some(en) = child.switch_key_enabled {
            out.push_str(&format!("switch_key_enabled = {}\n", en));
        }
        if let Some(r) = child.restore_on_wrong_spelling {
            out.push_str(&format!("restore_on_wrong = {}\n", r));
        }
        if let Some(u) = child.auto_uppercase_first_char {
            out.push_str(&format!("auto_uppercase_first = {}\n", u));
        }
        if let Some(b) = child.bracket_w {
            out.push_str(&format!("bracket_w = {}\n", b));
        }
        if let Some(h) = child.remember_history_across_space {
            out.push_str(&format!("remember_history = {}\n", h));
        }
        if let Some(snd) = child.sound_enabled {
            out.push_str(&format!("sound_enabled = {}\n", snd));
        }
        if let Some(ref sp) = child.sound_profile {
            out.push_str(&format!("sound_profile = {}\n", sp));
        }
        if let Some(vol) = child.sound_volume {
            out.push_str(&format!("sound_volume = {}\n", vol));
        }
        if let Some(u_m) = child.use_macro {
            out.push_str(&format!("use_macro = {}\n", u_m));
        }
        if let Some(u_me) = child.use_macro_in_english_mode {
            out.push_str(&format!("use_macro_in_english = {}\n", u_me));
        }
        if let Some(r_en) = child.remap_enabled {
            out.push_str(&format!("remap_enabled = {}\n", r_en));
        }
        if let Some(d_en) = child.disable_enabled {
            out.push_str(&format!("disable_enabled = {}\n", d_en));
        }

        if !child.muted_remaps.is_empty() {
            let names: Vec<String> = child
                .muted_remaps
                .iter()
                .map(|&vk| crate::engine::config::hotkey::vk_to_name(vk).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push_str(&format!("muted_remaps = {}\n", names.join(", ")));
        }
        for (&f_vk, &t_vk) in &child.overridden_remaps {
            let fnm = crate::engine::config::hotkey::vk_to_name(f_vk);
            let tnm = crate::engine::config::hotkey::vk_to_name(t_vk);
            out.push_str(&format!("override_remap = {} = {}\n", fnm, tnm));
        }
        for r in &child.app_remaps {
            let fnm = if !r.from_name.is_empty() {
                &r.from_name
            } else {
                "Key"
            };
            let tnm = if !r.to_name.is_empty() {
                &r.to_name
            } else {
                "Key"
            };
            out.push_str(&format!("app_remap = {} = {}\n", fnm, tnm));
        }

        if !child.unmuted_disabled.is_empty() {
            let names: Vec<String> = child
                .unmuted_disabled
                .iter()
                .map(|&vk| crate::engine::config::hotkey::vk_to_name(vk).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push_str(&format!("allow_disabled = {}\n", names.join(", ")));
        }
        if !child.app_disabled.is_empty() {
            let names: Vec<String> = child
                .app_disabled
                .iter()
                .map(|&vk| crate::engine::config::hotkey::vk_to_name(vk).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push_str(&format!("disable_keys = {}\n", names.join(", ")));
        }

        if !child.muted_macros.is_empty() {
            out.push_str(&format!("muted_macros = {}\n", child.muted_macros.join(", ")));
        }
        for (k, (v, t)) in &child.overridden_macros {
            if *t == MacroType::Normal {
                out.push_str(&format!("override_macro = {} = {}\n", k, v));
            } else {
                out.push_str(&format!(
                    "override_macro = {} = {}:{}\n",
                    k,
                    v,
                    t.as_str()
                ));
            }
        }
        for m in &child.app_macros {
            if m.macro_type == MacroType::Normal {
                out.push_str(&format!("app_macro = {} = {}\n", m.key, m.value));
            } else {
                out.push_str(&format!(
                    "app_macro = {} = {}:{}\n",
                    m.key,
                    m.value,
                    m.macro_type.as_str()
                ));
            }
        }
        out.push('\n');
    }

    out
}
