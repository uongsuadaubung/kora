//! Unit tests for Kora Profile-First Keyboard Workspace Hub
//! Validates Live Overlay Inheritance (Formula 2) and INI round-trip persistence.

use kora::engine::config::{EngineConfig, InputMethod};
use kora::engine::macro_table::{MacroEntry, MacroTable, MacroType};
use kora::engine::profile::{
    AppProfile, KeyRemapRule, LanguageMode, ProfileManager, RuleOrigin,
};
use kora::{parse_config_all, serialize_config_all};

#[test]
fn test_live_inheritance_remap() {
    let mut manager = ProfileManager::new();

    // 1. Setup Global Remaps:
    // CapsLock (0x14) -> Ctrl (0x11)
    // ScrollLock (0x91) -> Mute (0xAD)
    manager.global_profile.app_remaps.push(KeyRemapRule::new(0x14, 0x11));
    manager.global_profile.app_remaps.push(KeyRemapRule::new(0x91, 0xAD));

    // 2. Setup Child Profile "cs2":
    // - Mutes ScrollLock (0x91)
    // - Overrides CapsLock (0x14) -> Escape (0x1B)
    // - Adds App-Specific: F1 (0x70) -> F5 (0x74)
    let mut cs2 = AppProfile::new_child("cs2", "Counter-Strike 2", "cs2.exe", "🎮");
    cs2.muted_remaps.push(0x91);
    cs2.overridden_remaps.insert(0x14, 0x1B);
    cs2.app_remaps.push(KeyRemapRule::new(0x70, 0x74));
    manager.add_profile(cs2);

    // 3. Resolve effective remaps for CS2:
    let effective = manager.resolve_effective_remaps("cs2");
    assert_eq!(effective.len(), 2);

    // CapsLock must be remapped to Escape (0x1B), NOT Ctrl (0x11)
    let caps_rule = effective.iter().find(|r| r.from_vk == 0x14).expect("CapsLock rule present");
    assert_eq!(caps_rule.to_vk, 0x1B);

    // ScrollLock was muted, so it must NOT be in effective rules
    assert!(effective.iter().all(|r| r.from_vk != 0x91));

    // App-specific rule F1 -> F5 must be present
    let f1_rule = effective.iter().find(|r| r.from_vk == 0x70).expect("F1 rule present");
    assert_eq!(f1_rule.to_vk, 0x74);

    // 4. UI resolution check
    let ui_items = manager.resolve_remap_items_for_ui("cs2");
    assert_eq!(ui_items.len(), 3); // 2 global items (one overridden, one muted) + 1 app-specific
    let caps_ui = ui_items.iter().find(|i| i.from_vk == 0x14).unwrap();
    assert_eq!(caps_ui.origin, RuleOrigin::OverriddenGlobal);
    assert!(caps_ui.is_active);

    let scroll_ui = ui_items.iter().find(|i| i.from_vk == 0x91).unwrap();
    assert_eq!(scroll_ui.origin, RuleOrigin::MutedGlobal);
    assert!(!scroll_ui.is_active);

    let f1_ui = ui_items.iter().find(|i| i.from_vk == 0x70).unwrap();
    assert_eq!(f1_ui.origin, RuleOrigin::AppSpecific);
    assert!(f1_ui.is_active);

    // 5. Global profile UI items should have RuleOrigin::Root and empty badge label
    let global_ui_items = manager.resolve_remap_items_for_ui("global");
    assert_eq!(global_ui_items[0].origin, RuleOrigin::Root);
    assert_eq!(global_ui_items[0].origin.badge_label(), "");

    // 6. Test remap_enabled toggle
    // When CS2 disables remap, effective remaps must be empty
    manager.get_profile_mut("cs2").unwrap().remap_enabled = Some(false);
    assert!(manager.resolve_effective_remaps("cs2").is_empty());

    // When CS2 inherits (None), it gets global enabled
    manager.get_profile_mut("cs2").unwrap().remap_enabled = None;
    assert_eq!(manager.resolve_effective_remaps("cs2").len(), 2);

    // When global remap is disabled, both global and child return empty
    manager.global_profile.remap_enabled = Some(false);
    assert!(manager.resolve_effective_remaps("global").is_empty());
    assert!(manager.resolve_effective_remaps("cs2").is_empty());
}

#[test]
fn test_live_inheritance_disable() {
    let mut manager = ProfileManager::new();

    // Global disables: Insert (0x2D), Pause (0x13)
    manager.global_profile.app_disabled.push(0x2D);
    manager.global_profile.app_disabled.push(0x13);

    // Child profile "word" unblocks Insert (allows it), but blocks CapsLock (0x14)
    let mut word = AppProfile::new_child("word", "Microsoft Word", "winword.exe", "📝");
    word.unmuted_disabled.push(0x2D); // Unblocks Insert
    word.app_disabled.push(0x14); // Blocks CapsLock
    manager.add_profile(word);

    let effective = manager.resolve_effective_disabled("word");
    assert_eq!(effective.len(), 2);
    assert!(!effective.contains(&0x2D)); // Insert is unblocked!
    assert!(effective.contains(&0x13)); // Pause is still blocked (inherited from global)
    assert!(effective.contains(&0x14)); // CapsLock is blocked (app-specific)

    let global_ui_items = manager.resolve_disable_items_for_ui("global");
    assert_eq!(global_ui_items[0].origin, RuleOrigin::Root);
    assert_eq!(global_ui_items[0].origin.badge_label(), "");

    // Test disable_enabled toggle
    // When "word" disables key disabling, effective disabled keys must be empty
    manager.get_profile_mut("word").unwrap().disable_enabled = Some(false);
    assert!(manager.resolve_effective_disabled("word").is_empty());

    // When "word" inherits (None), it gets global enabled
    manager.get_profile_mut("word").unwrap().disable_enabled = None;
    assert_eq!(manager.resolve_effective_disabled("word").len(), 2);

    // When global disable is disabled, both global and child return empty
    manager.global_profile.disable_enabled = Some(false);
    assert!(manager.resolve_effective_disabled("global").is_empty());
    assert!(manager.resolve_effective_disabled("word").is_empty());
}

#[test]
fn test_live_inheritance_macro() {
    let mut manager = ProfileManager::new();

    let global_macros = vec![
        MacroEntry {
            key: "vn".to_string(),
            value: "Việt Nam".to_string(),
            macro_type: MacroType::Normal,
        },
        MacroEntry {
            key: "dc".to_string(),
            value: "được".to_string(),
            macro_type: MacroType::Normal,
        },
        MacroEntry {
            key: "email".to_string(),
            value: "global@kora.vn".to_string(),
            macro_type: MacroType::Normal,
        },
    ];

    // Child profile "cs2":
    // - mutes "vn"
    // - overrides "email" -> "cs2@valve.com"
    // - adds app_macro "gg" -> "Good Game!"
    let mut cs2 = AppProfile::new_child("cs2", "CS2", "cs2.exe", "🎮");
    cs2.muted_macros.push("vn".to_string());
    cs2.overridden_macros.insert(
        "email".to_string(),
        ("cs2@valve.com".to_string(), MacroType::Normal),
    );
    cs2.app_macros.push(MacroEntry {
        key: "gg".to_string(),
        value: "Good Game!".to_string(),
        macro_type: MacroType::Normal,
    });
    manager.add_profile(cs2);

    let effective = manager.resolve_effective_macros("cs2", &global_macros);
    assert_eq!(effective.len(), 3);

    // "vn" must be muted
    assert!(effective.iter().all(|m| m.key != "vn"));

    // "dc" is inherited
    let dc_entry = effective.iter().find(|m| m.key == "dc").unwrap();
    assert_eq!(dc_entry.value, "được");

    // "email" is overridden
    let email_entry = effective.iter().find(|m| m.key == "email").unwrap();
    assert_eq!(email_entry.value, "cs2@valve.com");

    // "gg" is app specific
    let gg_entry = effective.iter().find(|m| m.key == "gg").unwrap();
    assert_eq!(gg_entry.value, "Good Game!");

    let global_ui_items = manager.resolve_macro_items_for_ui("global", &global_macros);
    assert_eq!(global_ui_items[0].origin, RuleOrigin::Root);
    assert_eq!(global_ui_items[0].origin.badge_label(), "");
}

#[test]
fn test_profile_ini_roundtrip() {
    let mut config = EngineConfig::default();
    config.method = InputMethod::Telex;

    let mut macros = MacroTable::new();
    macros.insert_typed_no_cache("vn", "Việt Nam", MacroType::Normal);

    let mut manager = ProfileManager::new();
    manager.global_profile.app_remaps.push(KeyRemapRule::new(0x14, 0x11)); // Caps Lock = Ctrl
    manager.global_profile.app_disabled.push(0x2D); // Insert

    let mut cs2 = AppProfile::new_child("cs2", "Counter-Strike 2", "cs2.exe", "🎮");
    cs2.mode = LanguageMode::English;
    cs2.app_disabled.push(0x5B); // Left Windows
    cs2.remap_enabled = Some(false);
    cs2.disable_enabled = Some(true);
    cs2.use_macro = Some(true);
    cs2.use_macro_in_english_mode = Some(false);
    cs2.muted_macros.push("vn".to_string());
    cs2.app_macros.push(MacroEntry {
        key: "gg".to_string(),
        value: "Good game!".to_string(),
        macro_type: MacroType::Normal,
    });
    manager.add_profile(cs2);

    let ini_str = serialize_config_all(&config, &macros, &manager);

    let (parsed_cfg, parsed_macros, parsed_manager) = parse_config_all(&ini_str);

    assert_eq!(parsed_cfg.method, InputMethod::Telex);
    assert!(parsed_cfg.remap_enabled);
    assert!(parsed_cfg.disable_enabled);
    assert_eq!(parsed_manager.global_profile.remap_enabled, Some(true));
    assert_eq!(parsed_manager.global_profile.disable_enabled, Some(true));
    assert_eq!(parsed_macros.get("vn").unwrap().value, "Việt Nam");

    assert_eq!(parsed_manager.global_profile.app_remaps.len(), 1);
    assert_eq!(parsed_manager.global_profile.app_remaps[0].from_vk, 0x14);
    assert_eq!(parsed_manager.global_profile.app_remaps[0].to_vk, 0x11);

    assert_eq!(parsed_manager.global_profile.app_disabled, vec![0x2D]);

    let parsed_cs2 = parsed_manager.get_profile("cs2").expect("cs2 profile exists");
    assert_eq!(parsed_cs2.mode, LanguageMode::English);
    assert_eq!(parsed_cs2.app_disabled, vec![0x5B]);
    assert_eq!(parsed_cs2.remap_enabled, Some(false));
    assert_eq!(parsed_cs2.disable_enabled, Some(true));
    assert_eq!(parsed_cs2.use_macro, Some(true));
    assert_eq!(parsed_cs2.use_macro_in_english_mode, Some(false));
    assert_eq!(parsed_cs2.muted_macros, vec!["vn".to_string()]);
    assert_eq!(parsed_cs2.app_macros.len(), 1);
    assert_eq!(parsed_cs2.app_macros[0].key, "gg");
    assert_eq!(parsed_cs2.app_macros[0].value, "Good game!");
}

/// Typing must use the foreground app's profile, not the global config
#[test]
fn test_engine_types_with_active_child_profile() {
    use kora::{EngineAction, VietnameseEngine};

    let mut engine = VietnameseEngine::new(EngineConfig::default());
    let mut notepad = AppProfile::new_child("notepad", "Notepad", "notepad.exe", "");
    notepad.mode = LanguageMode::English;
    engine.profile_manager.add_profile(notepad);

    let type_word = |engine: &mut VietnameseEngine| {
        ['a', 'a']
            .iter()
            .map(|&c| engine.on_key(c, false, false))
            .any(|a| matches!(a, EngineAction::Replace { .. }))
    };

    // Global (Vietnamese): "aa" -> "â"
    assert!(type_word(&mut engine));
    engine.reset();

    // Child pins English: passthrough
    let (changed, id) = engine.switch_profile_by_process("NOTEPAD.EXE");
    assert!(changed);
    assert_eq!(id, "notepad");
    assert!(!type_word(&mut engine));

    // Toggle inside a child with a pinned mode flips the child, not global
    assert!(engine.toggle_enabled());
    assert!(engine.config().enabled);
    assert_eq!(engine.profile_manager.get_profile("notepad").unwrap().mode, LanguageMode::Vietnamese);
    engine.reset();
    assert!(type_word(&mut engine));
}
