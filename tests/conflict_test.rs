//! Integration & Diagnostic Tests for Cross-Feature Key Conflicts

use kora::engine::config::Hotkey;
use kora::engine::profile::{
    check_profile_conflicts, ConflictFeature, ResolvedDisableItem, ResolvedKeyMacroItem,
    ResolvedRemapItem, RuleOrigin,
};

#[test]
fn test_no_conflicts_when_clean() {
    let remaps = vec![ResolvedRemapItem {
        from_vk: 0x14, // CapsLock
        to_vk: 0x11,   // Ctrl
        from_name: "Caps Lock".into(),
        to_name: "Ctrl".into(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];
    let disabled = vec![ResolvedDisableItem {
        vk: 0x70, // F1
        key_name: "F1".into(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];
    let key_macros = vec![ResolvedKeyMacroItem {
        trigger_vk: 0x71, // F2
        trigger_name: "F2".into(),
        name: "Test Macro".into(),
        actions: Vec::new(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];
    let diag = check_profile_conflicts(
        &remaps,
        true,
        &disabled,
        true,
        &key_macros,
        true,
        Hotkey::default(),
        true,
    );
    assert!(diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 0);
}

#[test]
fn test_disable_shadows_remap_and_macro() {
    let remaps = vec![ResolvedRemapItem {
        from_vk: 0x70, // F1
        to_vk: 0x71,
        from_name: "F1".into(),
        to_name: "F2".into(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];
    let disabled = vec![ResolvedDisableItem {
        vk: 0x70, // F1
        key_name: "F1".into(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];
    let key_macros = vec![ResolvedKeyMacroItem {
        trigger_vk: 0x70, // F1
        trigger_name: "F1".into(),
        name: "Test Macro".into(),
        actions: Vec::new(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];

    let diag = check_profile_conflicts(
        &remaps,
        true,
        &disabled,
        true,
        &key_macros,
        true,
        Hotkey::default(),
        true,
    );
    assert!(!diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 2);
    assert!(diag.is_shadowed(0x70, ConflictFeature::Remap).is_some());
    assert!(diag.is_shadowed(0x70, ConflictFeature::KeyMacro).is_some());
}

#[test]
fn test_conflict_with_switch_key() {
    let switch_key = Hotkey::new(true, false, false, false, 0x20); // Ctrl + Space
    let disabled = vec![ResolvedDisableItem {
        vk: 0x20, // Space
        key_name: "Space".into(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];
    let diag = check_profile_conflicts(&[], true, &disabled, true, &[], true, switch_key, true);
    assert!(!diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 1);
    assert_eq!(diag.conflicts[0].shadowed, ConflictFeature::SwitchKey);
}

#[test]
fn test_conflict_with_switch_key_modifier() {
    let switch_key = Hotkey::default(); // Ctrl + Shift (vk = 0)
    let remaps = vec![ResolvedRemapItem {
        from_vk: 0x11, // Ctrl
        to_vk: 0x14,
        from_name: "Ctrl".into(),
        to_name: "Caps Lock".into(),
        origin: RuleOrigin::Root,
        is_active: true,
    }];
    let diag = check_profile_conflicts(&remaps, true, &[], true, &[], true, switch_key, true);
    assert!(!diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 1);
    assert_eq!(diag.conflicts[0].shadowed, ConflictFeature::SwitchKey);
}

#[test]
fn test_inheritance_cross_conflict_detection() {
    let mut manager = kora::engine::profile::ProfileManager::new();
    manager.global_profile.disable_enabled = true;
    manager.global_profile.remap_enabled = true;

    // Global disables F1 (0x70)
    manager.global_profile.app_disabled.push(0x70);

    // Child profile "cs2" inherits Disable(F1) and adds an AppSpecific Remap F1 -> F2 (0x71)
    let mut cs2 = kora::engine::profile::AppProfile::new_child("cs2", "CS2", "cs2.exe", "🎮");
    cs2.app_remaps.push(kora::engine::profile::KeyRemapRule::new(0x70, 0x71));
    manager.add_profile(cs2);

    let base_cfg = kora::engine::config::EngineConfig::default();
    let eff = manager.resolve_effective_config("cs2", &base_cfg);
    let remaps = manager.resolve_remap_items_for_ui("cs2");
    let disabled = manager.resolve_disable_items_for_ui("cs2");
    let key_macros = manager.resolve_key_macro_items_for_ui("cs2");

    let diag = check_profile_conflicts(
        &remaps,
        eff.remap_enabled,
        &disabled,
        eff.disable_enabled,
        &key_macros,
        eff.key_macro_enabled,
        eff.switch_key,
        eff.switch_key_enabled,
    );

    // Conflict: Inherited Disable(F1) shadows AppSpecific Remap(F1)
    assert!(!diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 1);
    assert_eq!(diag.conflicts[0].winner, ConflictFeature::Disable);
    assert_eq!(diag.conflicts[0].shadowed, ConflictFeature::Remap);
}

#[test]
fn test_inheritance_mute_resolves_conflict() {
    let mut manager = kora::engine::profile::ProfileManager::new();
    manager.global_profile.disable_enabled = true;
    manager.global_profile.remap_enabled = true;

    // Global has both Disable(F1) and Remap(F1 -> F2) -> Global has conflict!
    manager.global_profile.app_disabled.push(0x70);
    manager.global_profile.app_remaps.push(kora::engine::profile::KeyRemapRule::new(0x70, 0x71));

    // Child profile "photoshop" unblocks F1 (unmuted_disabled) to use the remap!
    let mut ps = kora::engine::profile::AppProfile::new_child("photoshop", "Photoshop", "photoshop.exe", "🎨");
    ps.unmuted_disabled.push(0x70);
    manager.add_profile(ps);

    let base_cfg = kora::engine::config::EngineConfig::default();
    let eff = manager.resolve_effective_config("photoshop", &base_cfg);
    let remaps = manager.resolve_remap_items_for_ui("photoshop");
    let disabled = manager.resolve_disable_items_for_ui("photoshop");
    let key_macros = manager.resolve_key_macro_items_for_ui("photoshop");

    let diag = check_profile_conflicts(
        &remaps,
        eff.remap_enabled,
        &disabled,
        eff.disable_enabled,
        &key_macros,
        eff.key_macro_enabled,
        eff.switch_key,
        eff.switch_key_enabled,
    );

    // In Photoshop profile, Disable(F1) is muted (is_active == false), so no conflict exists!
    assert!(diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 0);
}

#[test]
fn test_child_profile_custom_switch_key_conflict() {
    let mut manager = kora::engine::profile::ProfileManager::new();
    manager.global_profile.disable_enabled = true;
    // Global disables Space (0x20)
    manager.global_profile.app_disabled.push(0x20);

    // Global switch key is Ctrl + Shift (vk = 0) -> no conflict in Global
    let base_cfg = kora::engine::config::EngineConfig::default();

    // Child profile customizes switch key to Ctrl + Space (0x20)
    let mut child = kora::engine::profile::AppProfile::new_child("code", "VSCode", "code.exe", "💻");
    child.switch_key = Some(Hotkey::new(true, false, false, false, 0x20));
    manager.add_profile(child);

    let eff = manager.resolve_effective_config("code", &base_cfg);
    let remaps = manager.resolve_remap_items_for_ui("code");
    let disabled = manager.resolve_disable_items_for_ui("code");
    let key_macros = manager.resolve_key_macro_items_for_ui("code");

    let diag = check_profile_conflicts(
        &remaps,
        eff.remap_enabled,
        &disabled,
        eff.disable_enabled,
        &key_macros,
        eff.key_macro_enabled,
        eff.switch_key,
        eff.switch_key_enabled,
    );

    // Conflict: Inherited Disable(Space) swallows Child's custom SwitchKey(Ctrl + Space)
    assert!(!diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 1);
    assert_eq!(diag.conflicts[0].shadowed, ConflictFeature::SwitchKey);
}

#[test]
fn test_child_profile_feature_disabled_suppresses_conflict() {
    let mut manager = kora::engine::profile::ProfileManager::new();
    manager.global_profile.remap_enabled = true;
    manager.global_profile.key_macro_enabled = true;

    // Global has Remap(F1 -> F2) and KeyMacro(F1) -> Conflict in Global!
    manager.global_profile.app_remaps.push(kora::engine::profile::KeyRemapRule::new(0x70, 0x71));
    manager.global_profile.app_key_macros.push(kora::engine::profile::KeyMacroRule::new(
        0x70,
        "Test Macro",
        Vec::new(),
    ));

    // Child explicitly turns off Remap feature (remap_enabled = Some(false))
    let mut child = kora::engine::profile::AppProfile::new_child("game", "Game", "game.exe", "🎮");
    child.remap_enabled = Some(false);
    manager.add_profile(child);

    let base_cfg = kora::engine::config::EngineConfig::default();
    let eff = manager.resolve_effective_config("game", &base_cfg);
    let remaps = manager.resolve_remap_items_for_ui("game");
    let disabled = manager.resolve_disable_items_for_ui("game");
    let key_macros = manager.resolve_key_macro_items_for_ui("game");

    let diag = check_profile_conflicts(
        &remaps,
        eff.remap_enabled,
        &disabled,
        eff.disable_enabled,
        &key_macros,
        eff.key_macro_enabled,
        eff.switch_key,
        eff.switch_key_enabled,
    );

    // Since remap_enabled is false in this profile, Remap is inactive and does not shadow KeyMacro!
    assert!(diag.is_healthy());
    assert_eq!(diag.conflicts.len(), 0);
}

#[test]
fn test_conflict_and_diagnostics_i18n() {
    use kora::language::{get_strings, Language};

    let strings_vi = get_strings(Language::Vietnamese);
    let strings_en = get_strings(Language::English);

    // Verify ConflictFeature localized_name
    assert_eq!(ConflictFeature::Disable.localized_name(strings_vi), "Chặn phím");
    assert_eq!(ConflictFeature::Disable.localized_name(strings_en), "Block");
    assert_eq!(ConflictFeature::Remap.localized_name(strings_vi), "Đổi phím");
    assert_eq!(ConflictFeature::Remap.localized_name(strings_en), "Remap");
    assert_eq!(ConflictFeature::KeyMacro.localized_name(strings_vi), "Phím Macro");
    assert_eq!(ConflictFeature::KeyMacro.localized_name(strings_en), "Key Macro");
    assert_eq!(ConflictFeature::SwitchKey.localized_name(strings_vi), "Chuyển E ⇄ V");
    assert_eq!(ConflictFeature::SwitchKey.localized_name(strings_en), "Mode Switch");

    // Verify format templates
    let warning_vi = strings_vi.overview_conflicts_format.replace("{count}", "3");
    assert_eq!(warning_vi, "⚠️ Phát hiện 3 xung đột phím");
    let warning_en = strings_en.overview_conflicts_format.replace("{count}", "3");
    assert_eq!(warning_en, "⚠️ Detected 3 key conflict(s)");

    let shadowed_vi = strings_vi
        .status_shadowed_by
        .replace("{feature}", ConflictFeature::Disable.localized_name(strings_vi));
    assert_eq!(shadowed_vi, "⚠️ Bị Chặn phím nuốt");
    let shadowed_en = strings_en
        .status_shadowed_by
        .replace("{feature}", ConflictFeature::Disable.localized_name(strings_en));
    assert_eq!(shadowed_en, "⚠️ Shadowed by Block");

    assert_eq!(strings_vi.action_toggle_vietnamese, "Bật / Tắt Tiếng Việt");
    assert_eq!(strings_en.action_toggle_vietnamese, "Toggle Vietnamese");

    // Verify KeyDiagnostics localized_summary
    let diag = kora::engine::profile::KeyDiagnostics {
        active_remap: 2,
        active_disable: 1,
        active_key_macro: 4,
        ..Default::default()
    };
    assert_eq!(
        diag.localized_summary(strings_vi),
        "2 Đổi phím • 1 Chặn phím • 4 Phím Macro"
    );
    assert_eq!(
        diag.localized_summary(strings_en),
        "2 Remap • 1 Block • 4 Key Macro"
    );
}

