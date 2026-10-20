//! INI Serializer / Encoder for EngineConfig, MacroTable, and ProfileManager

use crate::engine::config::EngineConfig;
use crate::engine::config::hotkey::vk_to_name;
use crate::engine::macro_table::{MacroTable, MacroType};
use crate::engine::profile::{CharsetMode, LanguageMode, MacroAction, MethodMode, ProfileManager};

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
    out.push_str(&format!("charset = {}\n", config.charset.as_str()));
    out.push_str(&format!("enabled = {}\n", config.enabled));
    out.push_str(&format!(
        "switch_key_enabled = {}\n",
        config.switch_key_enabled
    ));
    out.push_str(&format!(
        "switch_key = {}\n",
        config.switch_key.to_config_str()
    ));
    out.push_str(&format!(
        "target_capture_enabled = {}\n",
        config.target_capture_enabled
    ));
    out.push_str(&format!(
        "target_capture_hotkey = {}\n\n",
        config.target_capture_hotkey.to_config_str()
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
    out.push_str(&format!(
        "use_macro_in_english = {}\n",
        config.use_macro_in_english_mode
    ));
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
    if !profiles.global_profile.muted_macros.is_empty() {
        out.push_str(&format!(
            "muted = {}\n",
            profiles.global_profile.muted_macros.join(", ")
        ));
    }
    out.push('\n');

    // [remap]
    out.push_str("[remap]\n");
    out.push_str(&format!("enabled = {}\n", config.remap_enabled));
    if !profiles.global_profile.app_remaps.is_empty() {
        out.push_str("# Đổi phím toàn hệ thống: <Phím gốc> = <Phím đích>\n");
        for r in &profiles.global_profile.app_remaps {
            if !r.from_name.is_empty() && !r.to_name.is_empty() {
                out.push_str(&format!("{} = {}\n", r.from_name, r.to_name));
            }
        }
    }
    if !profiles.global_profile.muted_remaps.is_empty() {
        let names: Vec<String> = profiles
            .global_profile
            .muted_remaps
            .iter()
            .map(|&vk| vk_to_name(vk).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        out.push_str(&format!("muted = {}\n", names.join(", ")));
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
            .map(|&vk| vk_to_name(vk).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        out.push_str(&format!("keys = {}\n", names.join(", ")));
    }
    if !profiles.global_profile.unmuted_disabled.is_empty() {
        let names: Vec<String> = profiles
            .global_profile
            .unmuted_disabled
            .iter()
            .map(|&vk| vk_to_name(vk).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        out.push_str(&format!("allow_disabled = {}\n", names.join(", ")));
    }
    out.push('\n');

    // [key_macro]
    out.push_str("[key_macro]\n");
    out.push_str(&format!("enabled = {}\n", config.key_macro_enabled));
    if !profiles.global_profile.app_key_macros.is_empty() {
        out.push_str("# Phím macro toàn hệ thống: <Phím kích hoạt> = <Tên> | <DSL>\n");
        for m in &profiles.global_profile.app_key_macros {
            if !m.trigger_name.is_empty() {
                let dsl = MacroAction::actions_to_dsl(&m.actions);
                out.push_str(&format!("{} = {} | {}\n", m.trigger_name, m.name, dsl));
            }
        }
    }
    if !profiles.global_profile.muted_key_macros.is_empty() {
        let names: Vec<String> = profiles
            .global_profile
            .muted_key_macros
            .iter()
            .map(|&vk| vk_to_name(vk).to_string())
            .filter(|s| !s.is_empty())
            .collect();
        out.push_str(&format!("muted = {}\n", names.join(", ")));
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
        if child.charset != CharsetMode::Inherit {
            out.push_str(&format!("charset = {}\n", child.charset.as_str()));
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
                .map(|&vk| vk_to_name(vk).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push_str(&format!("muted_remaps = {}\n", names.join(", ")));
        }
        for (&f_vk, &t_vk) in &child.overridden_remaps {
            let fnm = vk_to_name(f_vk);
            let tnm = vk_to_name(t_vk);
            out.push_str(&format!("override_remap = {} = {}\n", fnm, tnm));
        }
        for r in &child.app_remaps {
            if !r.from_name.is_empty() && !r.to_name.is_empty() {
                out.push_str(&format!("app_remap = {} = {}\n", r.from_name, r.to_name));
            }
        }

        if !child.unmuted_disabled.is_empty() {
            let names: Vec<String> = child
                .unmuted_disabled
                .iter()
                .map(|&vk| vk_to_name(vk).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push_str(&format!("allow_disabled = {}\n", names.join(", ")));
        }
        if !child.app_disabled.is_empty() {
            let names: Vec<String> = child
                .app_disabled
                .iter()
                .map(|&vk| vk_to_name(vk).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push_str(&format!("disable_keys = {}\n", names.join(", ")));
        }

        if !child.muted_macros.is_empty() {
            out.push_str(&format!(
                "muted_macros = {}\n",
                child.muted_macros.join(", ")
            ));
        }
        for (k, (v, t)) in &child.overridden_macros {
            if *t == MacroType::Normal {
                out.push_str(&format!("override_macro = {} = {}\n", k, v));
            } else {
                out.push_str(&format!("override_macro = {} = {}:{}\n", k, v, t.as_str()));
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

        if let Some(km_en) = child.key_macro_enabled {
            out.push_str(&format!("key_macro_enabled = {}\n", km_en));
        }
        if !child.muted_key_macros.is_empty() {
            let names: Vec<String> = child
                .muted_key_macros
                .iter()
                .map(|&vk| vk_to_name(vk).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            out.push_str(&format!("muted_key_macros = {}\n", names.join(", ")));
        }
        for (&trig_vk, (nm, actions)) in &child.overridden_key_macros {
            let trig_nm = vk_to_name(trig_vk);
            let dsl = MacroAction::actions_to_dsl(actions);
            out.push_str(&format!(
                "override_key_macro = {} = {} | {}\n",
                trig_nm, nm, dsl
            ));
        }
        for m in &child.app_key_macros {
            if !m.trigger_name.is_empty() {
                let dsl = MacroAction::actions_to_dsl(&m.actions);
                out.push_str(&format!(
                    "app_key_macro = {} = {} | {}\n",
                    m.trigger_name, m.name, dsl
                ));
            }
        }
        if let Some(strat) = child.app_strategy {
            out.push_str(&format!("app_strategy = {}\n", strat.as_str()));
        }
        for tr in &child.target_rules {
            out.push_str(&format!(
                "target = {} = {}\n",
                tr.target_id,
                tr.strategy.as_str()
            ));
        }
        out.push('\n');
    }

    out
}
