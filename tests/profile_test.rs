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
    manager.global_profile.remap_enabled = true;

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
    manager.global_profile.remap_enabled = false;
    assert!(manager.resolve_effective_remaps("global").is_empty());
    assert!(manager.resolve_effective_remaps("cs2").is_empty());
}

#[test]
fn test_live_inheritance_disable() {
    let mut manager = ProfileManager::new();
    manager.global_profile.disable_enabled = true;

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
    manager.global_profile.disable_enabled = false;
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
    let config = EngineConfig {
        method: InputMethod::Telex,
        ..Default::default()
    };

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
    assert!(!parsed_cfg.remap_enabled);
    assert!(!parsed_cfg.disable_enabled);
    assert!(!parsed_manager.global_profile.remap_enabled);
    assert!(!parsed_manager.global_profile.disable_enabled);
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

#[test]
fn test_macro_action_dsl() {
    use kora::engine::profile::MacroAction;

    let actions = vec![
        MacroAction::KeyDown("W".into()),
        MacroAction::Delay(150),
        MacroAction::Press("Ctrl+Shift+L".into()),
        MacroAction::Delay(50),
        MacroAction::KeyUp("W".into()),
        MacroAction::Text("glhf".into()),
    ];

    let dsl = MacroAction::actions_to_dsl(&actions);
    assert_eq!(
        dsl,
        "down(W) + delay(150) + press(Ctrl+Shift+L) + delay(50) + up(W) + text(glhf)"
    );

    let parsed = MacroAction::dsl_to_actions(&dsl);
    assert_eq!(parsed, actions);
}

#[test]
fn test_live_inheritance_key_macro() {
    use kora::engine::profile::{KeyMacroRule, MacroAction};

    let mut manager = ProfileManager::new();
    manager.global_profile.key_macro_enabled = true;

    // Global Key Macros:
    // F1 -> "Bunny Hop" | down(Space) + delay(15) + up(Space)
    // F2 -> "Buy AK" | press(B) + delay(50) + press(4)
    manager.global_profile.app_key_macros.push(KeyMacroRule::new(
        0x70,
        "Bunny Hop",
        vec![
            MacroAction::KeyDown("Space".into()),
            MacroAction::Delay(15),
            MacroAction::KeyUp("Space".into()),
        ],
    ));
    manager.global_profile.app_key_macros.push(KeyMacroRule::new(
        0x71,
        "Buy AK",
        vec![
            MacroAction::Press("B".into()),
            MacroAction::Delay(50),
            MacroAction::Press("4".into()),
        ],
    ));

    // Child Profile "cs2":
    // - Mutes F1 (0x70)
    // - Overrides F2 (0x71) -> "Custom Buy"
    // - Adds App-Specific: F3 (0x72) -> "Quick Drop"
    let mut cs2 = AppProfile::new_child("cs2", "Counter-Strike 2", "cs2.exe", "🎮");
    cs2.muted_key_macros.push(0x70);
    cs2.overridden_key_macros.insert(
        0x71,
        (
            "Custom Buy".into(),
            vec![
                MacroAction::Press("B".into()),
                MacroAction::Delay(100),
                MacroAction::Press("5".into()),
            ],
        ),
    );
    cs2.app_key_macros.push(KeyMacroRule::new(
        0x72,
        "Quick Drop",
        vec![
            MacroAction::Press("G".into()),
            MacroAction::Delay(20),
            MacroAction::Press("1".into()),
        ],
    ));
    manager.add_profile(cs2);

    // 1. Resolve effective key macros for CS2
    let effective = manager.resolve_effective_key_macros("cs2");
    assert_eq!(effective.len(), 2);

    // F1 is muted
    assert!(effective.iter().all(|m| m.trigger_vk != 0x70));

    // F2 is overridden
    let f2 = effective.iter().find(|m| m.trigger_vk == 0x71).unwrap();
    assert_eq!(f2.name, "Custom Buy");
    assert_eq!(f2.actions.len(), 3);

    // F3 is app-specific
    let f3 = effective.iter().find(|m| m.trigger_vk == 0x72).unwrap();
    assert_eq!(f3.name, "Quick Drop");

    // 2. UI items for CS2
    let ui_items = manager.resolve_key_macro_items_for_ui("cs2");
    assert_eq!(ui_items.len(), 3);

    let f1_ui = ui_items.iter().find(|i| i.trigger_vk == 0x70).unwrap();
    assert_eq!(f1_ui.origin, RuleOrigin::MutedGlobal);
    assert!(!f1_ui.is_active);

    let f2_ui = ui_items.iter().find(|i| i.trigger_vk == 0x71).unwrap();
    assert_eq!(f2_ui.origin, RuleOrigin::OverriddenGlobal);
    assert!(f2_ui.is_active);

    let f3_ui = ui_items.iter().find(|i| i.trigger_vk == 0x72).unwrap();
    assert_eq!(f3_ui.origin, RuleOrigin::AppSpecific);
    assert!(f3_ui.is_active);

    // 3. UI items for Global (Root) must have RuleOrigin::Root and empty badge label
    let global_ui = manager.resolve_key_macro_items_for_ui("global");
    assert_eq!(global_ui.len(), 2);
    assert_eq!(global_ui[0].origin, RuleOrigin::Root);
    assert_eq!(global_ui[0].origin.badge_label(), "");
    assert_eq!(global_ui[1].origin, RuleOrigin::Root);
    assert_eq!(global_ui[1].origin.badge_label(), "");
}

#[test]
fn test_key_macro_ini_roundtrip() {
    use kora::engine::profile::{KeyMacroRule, MacroAction};

    let mut config = EngineConfig::default();
    config.key_macro_enabled = true;
    let macros = MacroTable::new();
    let mut profiles = ProfileManager::new();
    profiles.global_profile.key_macro_enabled = true;

    profiles.global_profile.app_key_macros.push(KeyMacroRule::new(
        0x70, // F1
        "BHop",
        vec![
            MacroAction::KeyDown("Space".into()),
            MacroAction::Delay(20),
            MacroAction::KeyUp("Space".into()),
        ],
    ));

    let mut child = AppProfile::new_child("game", "Game", "game.exe", "🎮");
    child.key_macro_enabled = Some(true);
    child.app_key_macros.push(KeyMacroRule::new(
        0x71, // F2
        "Drop Weapon",
        vec![
            MacroAction::Press("G".into()),
            MacroAction::Delay(10),
            MacroAction::Press("1".into()),
        ],
    ));
    profiles.add_profile(child);

    let ini_str = serialize_config_all(&config, &macros, &profiles);
    assert!(ini_str.contains("[key_macro]"));
    assert!(ini_str.contains("enabled = true"));
    assert!(ini_str.contains("F1 = BHop | down(Space) + delay(20) + up(Space)"));
    assert!(ini_str.contains("app_key_macro = F2 = Drop Weapon | press(G) + delay(10) + press(1)"));

    let (parsed_cfg, _, parsed_profiles) = parse_config_all(&ini_str);
    assert!(parsed_cfg.key_macro_enabled);
    assert_eq!(parsed_profiles.global_profile.app_key_macros.len(), 1);
    assert_eq!(parsed_profiles.global_profile.app_key_macros[0].trigger_vk, 0x70);
    assert_eq!(parsed_profiles.global_profile.app_key_macros[0].name, "BHop");
    assert_eq!(parsed_profiles.global_profile.app_key_macros[0].actions.len(), 3);

    let parsed_child = parsed_profiles.get_profile("game").unwrap();
    assert_eq!(parsed_child.app_key_macros.len(), 1);
    assert_eq!(parsed_child.app_key_macros[0].trigger_vk, 0x71);
    assert_eq!(parsed_child.app_key_macros[0].name, "Drop Weapon");
    assert_eq!(parsed_child.app_key_macros[0].actions.len(), 3);
}

#[test]
fn test_child_inherits_root_muted_state() {
    use kora::engine::profile::{KeyMacroRule, MacroAction};

    let mut manager = ProfileManager::new();
    manager.global_profile.remap_enabled = true;
    manager.global_profile.disable_enabled = true;
    manager.global_profile.key_macro_enabled = true;

    // 1. Remap: Root has 0x14 (CapsLock) and 0x91 (ScrollLock)
    manager.global_profile.app_remaps.push(KeyRemapRule::new(0x14, 0x11));
    manager.global_profile.app_remaps.push(KeyRemapRule::new(0x91, 0xAD));
    // Root mutes 0x14
    manager.global_profile.muted_remaps.push(0x14);

    // 2. Disable: Root disables 0x2D (Insert) and 0x13 (Pause)
    manager.global_profile.app_disabled.push(0x2D);
    manager.global_profile.app_disabled.push(0x13);
    // Root turns off block for 0x2D
    manager.global_profile.unmuted_disabled.push(0x2D);

    // 3. Macro: Root has "btw" and "omw"
    let global_macros = vec![
        MacroEntry {
            key: "btw".into(),
            value: "by the way".into(),
            macro_type: MacroType::Normal,
        },
        MacroEntry {
            key: "omw".into(),
            value: "on my way".into(),
            macro_type: MacroType::Normal,
        },
    ];
    // Root mutes "btw"
    manager.global_profile.muted_macros.push("btw".into());

    // 4. Key Macro: Root has 0x70 (F1) and 0x71 (F2)
    manager.global_profile.app_key_macros.push(KeyMacroRule::new(
        0x70,
        "F1 Macro",
        vec![MacroAction::Press("Space".into())],
    ));
    manager.global_profile.app_key_macros.push(KeyMacroRule::new(
        0x71,
        "F2 Macro",
        vec![MacroAction::Press("Enter".into())],
    ));
    // Root mutes 0x70
    manager.global_profile.muted_key_macros.push(0x70);

    // Child profile inherits everything without overriding
    let child = AppProfile::new_child("app1", "App 1", "app1.exe", "⭐");
    manager.add_profile(child);

    // Verify Effective Rules: Child inherits the turned off state!
    let eff_remap = manager.resolve_effective_remaps("app1");
    assert_eq!(eff_remap.len(), 1);
    assert_eq!(eff_remap[0].from_vk, 0x91);

    let eff_disable = manager.resolve_effective_disabled("app1");
    assert_eq!(eff_disable.len(), 1);
    assert_eq!(eff_disable[0], 0x13);

    let eff_macro = manager.resolve_effective_macros("app1", &global_macros);
    assert_eq!(eff_macro.len(), 1);
    assert_eq!(eff_macro[0].key, "omw");

    let eff_km = manager.resolve_effective_key_macros("app1");
    assert_eq!(eff_km.len(), 1);
    assert_eq!(eff_km[0].trigger_vk, 0x71);

    // Verify UI Items: Root muted items appear as MutedGlobal and inactive
    let ui_remap = manager.resolve_remap_items_for_ui("app1");
    let remap_14 = ui_remap.iter().find(|r| r.from_vk == 0x14).unwrap();
    assert_eq!(remap_14.origin, RuleOrigin::MutedGlobal);
    assert!(!remap_14.is_active);

    let ui_disable = manager.resolve_disable_items_for_ui("app1");
    let disable_2d = ui_disable.iter().find(|d| d.vk == 0x2D).unwrap();
    assert_eq!(disable_2d.origin, RuleOrigin::MutedGlobal);
    assert!(!disable_2d.is_active);

    let ui_macro = manager.resolve_macro_items_for_ui("app1", &global_macros);
    let macro_btw = ui_macro.iter().find(|m| m.key == "btw").unwrap();
    assert_eq!(macro_btw.origin, RuleOrigin::MutedGlobal);
    assert!(!macro_btw.is_active);

    let ui_km = manager.resolve_key_macro_items_for_ui("app1");
    let km_70 = ui_km.iter().find(|k| k.trigger_vk == 0x70).unwrap();
    assert_eq!(km_70.origin, RuleOrigin::MutedGlobal);
    assert!(!km_70.is_active);

    // Now, child app1 enables 0x14 (CapsLock -> Ctrl) specifically
    let child_mut = manager.get_profile_mut("app1").unwrap();
    child_mut.app_remaps.push(KeyRemapRule::new(0x14, 0x11));
    child_mut.app_disabled.push(0x2D); // Enables 0x2D block specifically

    // Effective remap for app1 now has both rules!
    let eff_remap2 = manager.resolve_effective_remaps("app1");
    assert_eq!(eff_remap2.len(), 2);
    assert!(eff_remap2.iter().any(|r| r.from_vk == 0x14));

    // Effective disabled for app1 now has 0x2D blocked!
    let eff_disable2 = manager.resolve_effective_disabled("app1");
    assert_eq!(eff_disable2.len(), 2);
    assert!(eff_disable2.contains(&0x2D));

    // Global profile is STILL muted!
    assert!(manager.resolve_effective_remaps("global").iter().all(|r| r.from_vk != 0x14));
    assert!(!manager.resolve_effective_disabled("global").contains(&0x2D));

    // UI in app1 displays 0x14 as AppSpecific and active!
    let ui_remap2 = manager.resolve_remap_items_for_ui("app1");
    let remap_14_ui = ui_remap2.iter().find(|r| r.from_vk == 0x14).unwrap();
    assert_eq!(remap_14_ui.origin, RuleOrigin::AppSpecific);
    assert!(remap_14_ui.is_active);
}

#[test]
fn test_key_management_row_toggle_remap_disable_key_macro() {
    use kora::engine::profile::{KeyMacroRule, MacroAction};
    use kora::VietnameseEngine;

    let mut engine = VietnameseEngine::new(EngineConfig {
        remap_enabled: true,
        disable_enabled: true,
        key_macro_enabled: true,
        ..Default::default()
    });

    let pm = &mut engine.profile_manager;
    pm.global_profile.remap_enabled = true;
    pm.global_profile.disable_enabled = true;
    pm.global_profile.key_macro_enabled = true;

    // 1. Remap: CapsLock (0x14) -> Ctrl (0x11), ScrollLock (0x91) -> Mute (0xAD)
    pm.global_profile.app_remaps.push(KeyRemapRule::new(0x14, 0x11));
    pm.global_profile.app_remaps.push(KeyRemapRule::new(0x91, 0xAD));

    // 2. Disable: Insert (0x2D), Pause (0x13)
    pm.global_profile.app_disabled.push(0x2D);
    pm.global_profile.app_disabled.push(0x13);

    // 3. Key Macro: F1 (0x70), F2 (0x71)
    pm.global_profile.app_key_macros.push(KeyMacroRule::new(
        0x70,
        "F1 Macro",
        vec![MacroAction::Press("Space".into())],
    ));
    pm.global_profile.app_key_macros.push(KeyMacroRule::new(
        0x71,
        "F2 Macro",
        vec![MacroAction::Press("Enter".into())],
    ));

    // Verify initial active state: All rules are effective
    assert_eq!(engine.effective_remaps().len(), 2);
    assert!(engine.effective_remaps().iter().any(|r| r.from_vk == 0x14));
    assert!(engine.effective_remaps().iter().any(|r| r.from_vk == 0x91));

    assert_eq!(engine.effective_disabled().len(), 2);
    assert!(engine.effective_disabled().contains(&0x2D));
    assert!(engine.effective_disabled().contains(&0x13));

    assert_eq!(engine.effective_key_macros().len(), 2);
    assert!(engine.effective_key_macros().iter().any(|m| m.trigger_vk == 0x70));
    assert!(engine.effective_key_macros().iter().any(|m| m.trigger_vk == 0x71));

    // --- TOGGLE OFF (Mute each row in Global Profile) ---
    // Remap: Mute CapsLock (0x14)
    engine.profile_manager.global_profile.muted_remaps.push(0x14);
    // Disable: Mute disable (unblock) Insert (0x2D)
    engine.profile_manager.global_profile.unmuted_disabled.push(0x2D);
    // Key Macro: Mute F1 (0x70)
    engine.profile_manager.global_profile.muted_key_macros.push(0x70);

    // Verify Effective: Muted rules must NOT be in effective lists!
    let eff_remap = engine.effective_remaps();
    assert_eq!(eff_remap.len(), 1);
    assert_eq!(eff_remap[0].from_vk, 0x91); // Only 0x91 remains
    assert!(eff_remap.iter().all(|r| r.from_vk != 0x14)); // 0x14 is gone!

    let eff_disable = engine.effective_disabled();
    assert_eq!(eff_disable.len(), 1);
    assert_eq!(eff_disable[0], 0x13); // Only 0x13 remains
    assert!(!eff_disable.contains(&0x2D)); // 0x2D is unblocked!

    let eff_km = engine.effective_key_macros();
    assert_eq!(eff_km.len(), 1);
    assert_eq!(eff_km[0].trigger_vk, 0x71); // Only 0x71 remains
    assert!(eff_km.iter().all(|m| m.trigger_vk != 0x70)); // 0x70 is gone!

    // Verify UI reflects inactive status
    let ui_remaps = engine.profile_manager.resolve_remap_items_for_ui("global");
    let ui_14 = ui_remaps.iter().find(|r| r.from_vk == 0x14).unwrap();
    assert!(!ui_14.is_active);

    let ui_disabled = engine.profile_manager.resolve_disable_items_for_ui("global");
    let ui_2d = ui_disabled.iter().find(|d| d.vk == 0x2D).unwrap();
    assert!(!ui_2d.is_active);

    let ui_km = engine.profile_manager.resolve_key_macro_items_for_ui("global");
    let ui_70 = ui_km.iter().find(|k| k.trigger_vk == 0x70).unwrap();
    assert!(!ui_70.is_active);

    // --- TOGGLE ON (Unmute each row in Global Profile) ---
    engine.profile_manager.global_profile.muted_remaps.retain(|&v| v != 0x14);
    engine.profile_manager.global_profile.unmuted_disabled.retain(|&v| v != 0x2D);
    engine.profile_manager.global_profile.muted_key_macros.retain(|&v| v != 0x70);

    // Verify Effective: All rules are restored and effective again!
    assert_eq!(engine.effective_remaps().len(), 2);
    assert!(engine.effective_remaps().iter().any(|r| r.from_vk == 0x14));

    assert_eq!(engine.effective_disabled().len(), 2);
    assert!(engine.effective_disabled().contains(&0x2D));

    assert_eq!(engine.effective_key_macros().len(), 2);
    assert!(engine.effective_key_macros().iter().any(|m| m.trigger_vk == 0x70));
}



