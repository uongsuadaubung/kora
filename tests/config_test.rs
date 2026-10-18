use kora::{
    EngineConfig, InputMethod, MacroTable, UiTheme, get_config_path, parse_config_and_macros,
};

#[test]
fn test_default_config_has_macro_disabled() {
    let config = EngineConfig::default();
    assert!(!config.use_macro, "use_macro must be false by default");
    assert!(
        config.switch_with_ctrl_shift,
        "switch_with_ctrl_shift should default to true"
    );
    assert!(
        config.show_dialog_on_startup,
        "show_dialog_on_startup should default to true"
    );
    assert!(
        !config.remap_enabled,
        "remap_enabled should default to false"
    );
    assert!(
        !config.disable_enabled,
        "disable_enabled should default to false"
    );
}

#[test]
fn test_textbox_composite() {
    use kora::ui::components::textbox::TextBox;
    use kora::ui::win32::*;
    use std::ptr::null_mut;

    unsafe {
        let parent = CreateWindowExW(
            0,
            to_wide("STATIC").as_ptr(),
            null_mut(),
            0x80000000,
            0, 0, 300, 300,
            0, 0, 0, null_mut(),
        );
        assert!(parent != 0);

        let tb = TextBox::create(parent, 101, "initial", "placeholder", 10, 10, 120, 28, 0)
            .expect("TextBox should create successfully");

        assert_eq!(tb.id(), 101);
        assert_eq!(tb.get_text(), "initial");

        // Test SetText / GetText via method
        tb.set_text("updated text");
        assert_eq!(tb.get_text(), "updated text");
        assert!(!tb.is_empty());

        // Test Win32 standard API forwarding via container HWND
        let mut buf = [0u16; 64];
        let copied = GetWindowTextW(tb.hwnd(), buf.as_mut_ptr(), 64);
        assert!(copied > 0);
        let win_text = String::from_utf16_lossy(&buf[..copied as usize]);
        assert_eq!(win_text, "updated text");

        SetWindowTextW(tb.hwnd(), to_wide("via win32 set").as_ptr());
        assert_eq!(tb.get_text(), "via win32 set");

        tb.clear();
        assert!(tb.is_empty());

        DestroyWindow(tb.hwnd());
        DestroyWindow(parent);
    }
}

#[test]
fn test_parse_config_and_macros() {
    let ini = r#"
# Sample config file
[general]
method = vni
enabled = false
switch_key = ctrl_shift

[spelling]
check_spelling = true
restore_on_wrong = false
auto_uppercase_first = true

[features]
use_macro = true
bracket_w = false
quick_start_consonant = true
quick_end_consonant = true
remember_history = true

[sound]
enabled = true
switch = NovelKeys Cream
volume = 85

[system]
language = en
show_dialog_on_startup = false
debug = true
theme = dark

[macro]
ko = không
dc = được
vn = Việt Nam
"#;

    let (config, macros) = parse_config_and_macros(ini);

    assert_eq!(config.method, InputMethod::Vni);
    assert!(!config.enabled);
    assert!(config.switch_with_ctrl_shift);
    assert!(!config.restore_on_wrong_spelling);
    assert!(config.auto_uppercase_first_char);
    assert!(config.use_macro);
    assert!(!config.bracket_w);
    assert!(config.remember_history_across_space);
    assert!(config.sound_enabled);
    assert_eq!(&*config.sound_profile, "NovelKeys Cream");
    assert_eq!(config.sound_volume, 85);
    assert_eq!(config.language, kora::Language::English);
    assert!(!config.show_dialog_on_startup);
    assert!(config.debug);
    assert_eq!(config.theme, UiTheme::Dark);

    assert_eq!(macros.lookup("ko"), Some("không".to_string()));
    assert_eq!(macros.lookup("dc"), Some("được".to_string()));
    assert_eq!(macros.lookup("vn"), Some("Việt Nam".to_string()));
    assert_eq!(macros.len(), 3);
}

#[test]
fn test_config_store_roundtrip() {
    let config = EngineConfig {
        method: InputMethod::SimpleTelex1,
        language: kora::Language::English,
        use_macro: false,
        show_dialog_on_startup: false,
        theme: UiTheme::Dark,
        sound_enabled: true,
        sound_profile: std::sync::Arc::from("Topre"),
        sound_volume: 80,
        ..Default::default()
    };

    let mut macros = MacroTable::new();
    macros.insert("test", "thử nghiệm");
    macros.insert("rust", "ngôn ngữ Rust");

    // Serialize to INI string and parse back
    let ini_str = kora::serialize_config_and_macros(&config, &macros);
    let (loaded_config, loaded_macros) = parse_config_and_macros(&ini_str);

    assert_eq!(loaded_config.method, InputMethod::SimpleTelex1);
    assert_eq!(loaded_config.language, kora::Language::English);
    assert!(!loaded_config.use_macro);
    assert!(!loaded_config.show_dialog_on_startup);
    assert_eq!(loaded_config.theme, UiTheme::Dark);
    assert!(loaded_config.sound_enabled);
    assert_eq!(&*loaded_config.sound_profile, "Topre");
    assert_eq!(loaded_config.sound_volume, 80);
    assert_eq!(loaded_macros.lookup("test"), Some("thử nghiệm".to_string()));
    assert_eq!(
        loaded_macros.lookup("rust"),
        Some("ngôn ngữ Rust".to_string())
    );

    // Verify config path helper is valid
    let path = get_config_path();
    assert!(
        path.to_string_lossy().contains(".config"),
        "Config path should be in .config"
    );
}

#[test]
fn test_inspect_sound_banks() {
    if let Some(switches_dir) = kora::engine::config_store::get_switches_dir()
        && let Ok(entries) = std::fs::read_dir(&switches_dir)
    {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type()
                && ft.is_dir()
            {
                let switch_name = entry.file_name().to_string_lossy().to_string();
                let mut space_files = Vec::new();
                let mut backspace_files = Vec::new();
                let mut normal_files = Vec::new();
                if let Ok(sub) = std::fs::read_dir(entry.path()) {
                    for f in sub.flatten() {
                        let fname = f.file_name().to_string_lossy().to_string();
                        let stem = f
                            .path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_ascii_lowercase())
                            .unwrap_or_default();
                        match kora::platform::win32::match_file_tag(&stem) {
                            Some(kora::platform::win32::KeyTag::Space) => space_files.push(fname),
                            Some(kora::platform::win32::KeyTag::Backspace) => {
                                backspace_files.push(fname)
                            }
                            None => normal_files.push(fname),
                            _ => {}
                        }
                    }
                }
                eprintln!(
                    "Switch [{}] -> Space: {:?} | Backspace: {:?} | Normal: {:?}",
                    switch_name, space_files, backspace_files, normal_files
                );
                assert_eq!(
                    space_files,
                    vec!["space.wav"],
                    "Space should only contain space.wav for {}",
                    switch_name
                );
                assert_eq!(
                    backspace_files,
                    vec!["backspace.wav"],
                    "Backspace should only contain backspace.wav for {}",
                    switch_name
                );
            }
        }
    }
}

#[test]
fn test_exact_match_file_tag() {
    use kora::platform::win32::{KeyTag, match_file_tag};

    // Strict 1:1 match for all 18 canonical keys
    assert_eq!(match_file_tag("space"), Some(KeyTag::Space));
    assert_eq!(match_file_tag("backspace"), Some(KeyTag::Backspace));
    assert_eq!(match_file_tag("enter"), Some(KeyTag::Enter));
    assert_eq!(match_file_tag("tab"), Some(KeyTag::Tab));
    assert_eq!(match_file_tag("delete"), Some(KeyTag::Delete));
    assert_eq!(match_file_tag("shift"), Some(KeyTag::Shift));
    assert_eq!(match_file_tag("ctrl"), Some(KeyTag::Ctrl));
    assert_eq!(match_file_tag("alt"), Some(KeyTag::Alt));
    assert_eq!(match_file_tag("win"), Some(KeyTag::Win));
    assert_eq!(match_file_tag("capslock"), Some(KeyTag::CapsLock));
    assert_eq!(match_file_tag("esc"), Some(KeyTag::Esc));
    assert_eq!(match_file_tag("fn"), Some(KeyTag::Fn));
    assert_eq!(match_file_tag("arrow"), Some(KeyTag::Arrow));
    assert_eq!(match_file_tag("home"), Some(KeyTag::Home));
    assert_eq!(match_file_tag("end"), Some(KeyTag::End));
    assert_eq!(match_file_tag("pageup"), Some(KeyTag::PageUp));
    assert_eq!(match_file_tag("pagedown"), Some(KeyTag::PageDown));
    assert_eq!(match_file_tag("insert"), Some(KeyTag::Insert));

    // Ambiguous and non-canonical names must NOT match special keys, must fallback to None (Normal)
    assert_eq!(match_file_tag("winter"), None);
    assert_eq!(match_file_tag("alter"), None);
    assert_eq!(match_file_tag("space_enter"), None);
    assert_eq!(match_file_tag("del"), None);
    assert_eq!(match_file_tag("back"), None);
    assert_eq!(match_file_tag("escape"), None);
    assert_eq!(match_file_tag("1"), None);
    assert_eq!(match_file_tag("click"), None);
}

#[test]
fn test_custom_switch_hotkey() {
    use kora::Hotkey;

    // Default: Ctrl + Shift
    let def_hk = Hotkey::default();
    assert!(def_hk.is_ctrl_shift());
    assert_eq!(def_hk.display_text(), "Ctrl + Shift");
    assert_eq!(def_hk.to_config_str(), "ctrl_shift");

    // Custom: Ctrl + Space
    let space_hk = Hotkey::from_config_str("ctrl_space");
    assert_eq!(space_hk.display_text(), "Ctrl + Space");
    assert_eq!(space_hk.vk, 0x20);
    assert!(space_hk.ctrl && !space_hk.shift);

    // Custom: Alt + Z
    let alt_z = Hotkey::from_config_str("alt_z");
    assert_eq!(alt_z.display_text(), "Alt + Z");
    assert_eq!(alt_z.vk, 0x5A);
    assert!(alt_z.alt && !alt_z.ctrl);

    // INI parsing with custom switch_key and switch_key_enabled
    let ini = r#"
[general]
method = telex
enabled = true
switch_key_enabled = false
switch_key = ctrl_space
"#;
    let (config, _) = parse_config_and_macros(ini);
    assert_eq!(config.switch_key.display_text(), "Ctrl + Space");
    assert!(!config.switch_with_ctrl_shift);
    assert!(!config.switch_key_enabled);

    // Serialization roundtrip
    let mut config2 = EngineConfig::default();
    assert!(config2.switch_key_enabled);
    config2.switch_key_enabled = false;
    config2.switch_key = Hotkey::new(true, false, false, false, 0x20); // Ctrl + Space
    let macros2 = MacroTable::new();
    let serialized = kora::serialize_config_and_macros(&config2, &macros2);
    assert!(serialized.contains("switch_key_enabled = false"));
    assert!(serialized.contains("switch_key = ctrl_space"));

    let (loaded, _) = parse_config_and_macros(&serialized);
    assert_eq!(loaded.switch_key, config2.switch_key);
    assert_eq!(loaded.switch_key_enabled, config2.switch_key_enabled);
}

#[test]
fn test_target_capture_hotkey_and_state() {
    #[cfg(target_os = "windows")]
    {
        use kora::platform::win32::capture::*;
        assert!(is_target_capture_enabled());
        let hk = get_target_capture_hotkey();
        assert_eq!(hk.display_text(), "Ctrl + ~");
        assert!(hk.ctrl && !hk.alt && !hk.shift && !hk.win);
        assert_eq!(hk.vk, 0xC0);

        set_target_capture_enabled(false);
        assert!(!is_target_capture_enabled());
        set_target_capture_enabled(true);
        assert!(is_target_capture_enabled());

        let custom_hk = kora::engine::config::Hotkey::new(true, true, false, false, 0x4B);
        set_target_capture_hotkey(custom_hk);
        assert_eq!(get_target_capture_hotkey().display_text(), "Ctrl + Shift + K");

        set_target_capture_hotkey(hk);
        assert_eq!(get_target_capture_hotkey().display_text(), "Ctrl + ~");
    }
}

#[test]
fn test_target_capture_inheritance_and_codec() {
    use kora::engine::profile::{AppProfile, ProfileManager};
    use kora::Hotkey;

    // 1. Root default configuration
    let root = EngineConfig::default();
    assert!(root.target_capture_enabled, "root default must have target_capture_enabled = true");
    assert_eq!(root.target_capture_hotkey.display_text(), "Ctrl + ~");
    assert_eq!(root.target_capture_hotkey.vk, 0xC0);

    // 2. Child profile inheritance
    let mut pm = ProfileManager::new();
    let child = AppProfile::new_child("figma", "Figma", "figma.exe", "🎨");
    pm.add_profile(child);
    let child_ref = pm.get_profile("figma").unwrap();
    assert_eq!(child_ref.target_capture_enabled, None);
    assert_eq!(child_ref.target_capture_hotkey, None);

    // Resolve effective config when child inherits
    let eff = pm.resolve_effective_config("figma", &root);
    assert!(eff.target_capture_enabled);
    assert_eq!(eff.target_capture_hotkey.display_text(), "Ctrl + ~");

    // 3. Child profile overrides target capture
    if let Some(c) = pm.get_profile_mut("figma") {
        c.target_capture_enabled = Some(false);
        c.target_capture_hotkey = Some(Hotkey::new(true, false, true, false, 0x4B)); // Ctrl + Alt + K
    }
    let eff_override = pm.resolve_effective_config("figma", &root);
    assert!(!eff_override.target_capture_enabled);
    assert_eq!(eff_override.target_capture_hotkey.display_text(), "Ctrl + Alt + K");

    // 4. Codec serialization and deserialization
    let macros = MacroTable::new();
    let serialized = kora::engine::config_store::codec::encoder::serialize_config_all(&root, &macros, &pm);
    assert!(serialized.contains("target_capture_enabled = true"));
    assert!(serialized.contains("target_capture_hotkey = ctrl_~"));
    assert!(serialized.contains("target_capture_enabled = false"));
    assert!(serialized.contains("target_capture_hotkey = ctrl_alt_k"));

    let (dec_cfg, _, dec_pm) = kora::parse_config_all(&serialized);
    assert!(dec_cfg.target_capture_enabled);
    assert_eq!(dec_cfg.target_capture_hotkey.display_text(), "Ctrl + ~");
    let dec_child = dec_pm.get_profile("figma").unwrap();
    assert_eq!(dec_child.target_capture_enabled, Some(false));
    assert_eq!(dec_child.target_capture_hotkey.as_ref().map(|k| k.display_text()), Some("Ctrl + Alt + K".to_string()));

    // 5. Backward compatibility: verify decoding legacy ctrl_` still parses to 0xC0
    let (dec_legacy, _, _) = kora::parse_config_all("[general]\ntarget_capture_hotkey = ctrl_`\n");
    assert_eq!(dec_legacy.target_capture_hotkey.vk, 0xC0);
    assert_eq!(dec_legacy.target_capture_hotkey.display_text(), "Ctrl + ~");
}

#[test]
fn test_modifier_reset_and_reinstall_hooks() {
    #[cfg(target_os = "windows")]
    {
        kora::platform::reset_modifier_states();
        kora::platform::reinstall_hooks();
        kora::platform::reset_modifier_states();
    }
}

#[test]
fn test_uia_focused() {
    #[cfg(target_os = "windows")]
    {
        // Verifies that COM initialization, UIAutomation runtime resolution, and query succeed without error or panic
        let _ = kora::platform::win32::get_focused_element_id();
    }
}


