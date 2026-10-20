//! Key Conflict Detection & Diagnostics Engine
//!
//! Analyzes cross-feature conflicts among keyboard customization rules according
//! to the Windows Low-Level Keyboard Hook execution priority:
//!
//! Priority Hierarchy:
//! 1. Key Disabling (Chặn phím) - Swallows event immediately.
//! 2. Key Remapping (Đổi phím) - Dispatches target key and swallows original.
//! 3. Key Macro (Phím Macro) - Dispatches macro sequence and swallows trigger.
//! 4. Switch Mode Hotkey (Phím chuyển E ⇄ V) - Toggles typing mode.

use crate::engine::config::Hotkey;
use crate::engine::config::hotkey::vk_to_name;
use crate::engine::profile::{ResolvedDisableItem, ResolvedKeyMacroItem, ResolvedRemapItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictFeature {
    Disable,
    Remap,
    KeyMacro,
    SwitchKey,
}

impl ConflictFeature {
    pub fn localized_name(&self, s: &crate::language::LanguageStrings) -> &'static str {
        match self {
            Self::Disable => s.feature_disable,
            Self::Remap => s.feature_remap,
            Self::KeyMacro => s.feature_key_macro,
            Self::SwitchKey => s.feature_switch_key,
        }
    }

    pub fn name_vi(&self) -> &'static str {
        match self {
            Self::Disable => "Chặn phím",
            Self::Remap => "Đổi phím",
            Self::KeyMacro => "Phím Macro",
            Self::SwitchKey => "Phím chuyển E ⇄ V",
        }
    }

    pub fn name_en(&self) -> &'static str {
        match self {
            Self::Disable => "Disable Key",
            Self::Remap => "Remap Key",
            Self::KeyMacro => "Key Macro",
            Self::SwitchKey => "Switch Mode Hotkey",
        }
    }
}

/// A diagnosed key conflict between two features on the same physical key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyConflict {
    /// Physical key Virtual-Key code
    pub vk: u32,
    /// Human-readable key name
    pub key_name: String,
    /// The feature that wins and takes precedence
    pub winner: ConflictFeature,
    /// The feature that is shadowed / neutralized
    pub shadowed: ConflictFeature,
    /// Detailed diagnostic description
    pub reason: String,
}

/// Summary report of keyboard customizations and diagnostics
#[derive(Debug, Clone, Default)]
pub struct KeyDiagnostics {
    pub conflicts: Vec<KeyConflict>,
    pub total_remap: usize,
    pub active_remap: usize,
    pub total_disable: usize,
    pub active_disable: usize,
    pub total_key_macro: usize,
    pub active_key_macro: usize,
    pub switch_key_enabled: bool,
    pub switch_key: Hotkey,
}

impl KeyDiagnostics {
    pub fn is_healthy(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Finds if a specific VK code in a given feature is shadowed by a higher-priority feature
    pub fn is_shadowed(&self, vk: u32, feature: ConflictFeature) -> Option<&KeyConflict> {
        self.conflicts
            .iter()
            .find(|c| c.vk == vk && c.shadowed == feature)
    }

    /// Formats a concise localized summary string of total rules
    pub fn localized_summary(&self, s: &crate::language::LanguageStrings) -> String {
        format!(
            "{} {} • {} {} • {} {}",
            self.active_remap,
            s.feature_remap,
            self.active_disable,
            s.feature_disable,
            self.active_key_macro,
            s.feature_key_macro
        )
    }

    pub fn summary_vi(&self) -> String {
        format!(
            "{} Đổi phím • {} Chặn phím • {} Phím Macro",
            self.active_remap, self.active_disable, self.active_key_macro
        )
    }

    pub fn summary_en(&self) -> String {
        format!(
            "{} Remap • {} Block • {} Macro",
            self.active_remap, self.active_disable, self.active_key_macro
        )
    }
}

/// Detects all cross-feature conflicts for the selected profile and global config
#[allow(clippy::too_many_arguments)]
pub fn check_profile_conflicts(
    remaps: &[ResolvedRemapItem],
    remap_feature_enabled: bool,
    disabled: &[ResolvedDisableItem],
    disable_feature_enabled: bool,
    key_macros: &[ResolvedKeyMacroItem],
    key_macro_feature_enabled: bool,
    switch_key: Hotkey,
    switch_key_enabled: bool,
) -> KeyDiagnostics {
    let mut conflicts = Vec::new();

    let total_remap = remaps.len();
    let active_remaps: Vec<&ResolvedRemapItem> = if remap_feature_enabled {
        remaps.iter().filter(|r| r.is_active).collect()
    } else {
        Vec::new()
    };
    let active_remap = active_remaps.len();

    let total_disable = disabled.len();
    let active_disabled: Vec<&ResolvedDisableItem> = if disable_feature_enabled {
        disabled.iter().filter(|d| d.is_active).collect()
    } else {
        Vec::new()
    };
    let active_disable = active_disabled.len();

    let total_key_macro = key_macros.len();
    let active_key_macros: Vec<&ResolvedKeyMacroItem> = if key_macro_feature_enabled {
        key_macros.iter().filter(|m| m.is_active).collect()
    } else {
        Vec::new()
    };
    let active_key_macro = active_key_macros.len();

    // 1. Disable vs Remap (Disable wins)
    for r in &active_remaps {
        if let Some(d) = active_disabled.iter().find(|d| d.vk == r.from_vk) {
            let name = if !r.from_name.is_empty() {
                r.from_name.clone()
            } else {
                d.key_name.clone()
            };
            conflicts.push(KeyConflict {
                vk: r.from_vk,
                key_name: name.clone(),
                winner: ConflictFeature::Disable,
                shadowed: ConflictFeature::Remap,
                reason: format!(
                    "Phím [{}] đang bị 'Chặn phím' nuốt, quy tắc 'Đổi phím' sẽ không thể kích hoạt.",
                    name
                ),
            });
        }
    }

    // 2. Disable vs KeyMacro (Disable wins)
    for m in &active_key_macros {
        if let Some(d) = active_disabled.iter().find(|d| d.vk == m.trigger_vk) {
            let name = if !m.trigger_name.is_empty() {
                m.trigger_name.clone()
            } else {
                d.key_name.clone()
            };
            conflicts.push(KeyConflict {
                vk: m.trigger_vk,
                key_name: name.clone(),
                winner: ConflictFeature::Disable,
                shadowed: ConflictFeature::KeyMacro,
                reason: format!(
                    "Phím [{}] đang bị 'Chặn phím' nuốt, 'Phím Macro' ({}) sẽ không thể kích hoạt.",
                    name, m.name
                ),
            });
        }
    }

    // 3. Remap vs KeyMacro (Remap wins)
    for m in &active_key_macros {
        // Only if not already shadowed by Disable
        if active_disabled.iter().any(|d| d.vk == m.trigger_vk) {
            continue;
        }
        if let Some(r) = active_remaps.iter().find(|r| r.from_vk == m.trigger_vk) {
            let name = if !m.trigger_name.is_empty() {
                m.trigger_name.clone()
            } else {
                r.from_name.clone()
            };
            conflicts.push(KeyConflict {
                vk: m.trigger_vk,
                key_name: name.clone(),
                winner: ConflictFeature::Remap,
                shadowed: ConflictFeature::KeyMacro,
                reason: format!(
                    "Quy tắc 'Đổi phím' [{}] ưu tiên cao hơn sẽ nuốt phím trước, 'Phím Macro' ({}) sẽ không thể kích hoạt.",
                    name, m.name
                ),
            });
        }
    }

    // 4. Conflicts with Switch Mode Hotkey (Phím chuyển E ⇄ V)
    if switch_key_enabled {
        let switch_name = switch_key.display_text();

        // 4A. If Switch Key has a non-zero main VK
        if switch_key.vk != 0 {
            let vk = switch_key.vk;
            let key_name = {
                let s = vk_to_name(vk);
                if !s.is_empty() {
                    s.to_string()
                } else {
                    format!("VK {:#04X}", vk)
                }
            };

            // Check if main VK is in Disable
            if active_disabled.iter().any(|d| d.vk == vk) {
                conflicts.push(KeyConflict {
                    vk,
                    key_name: key_name.clone(),
                    winner: ConflictFeature::Disable,
                    shadowed: ConflictFeature::SwitchKey,
                    reason: format!(
                        "Phím chính [{}] của Phím chuyển E ⇄ V ({}) đang bị 'Chặn phím' nuốt hoàn toàn.",
                        key_name, switch_name
                    ),
                });
            } else if active_remaps.iter().any(|r| r.from_vk == vk) {
                conflicts.push(KeyConflict {
                    vk,
                    key_name: key_name.clone(),
                    winner: ConflictFeature::Remap,
                    shadowed: ConflictFeature::SwitchKey,
                    reason: format!(
                        "Phím chính [{}] của Phím chuyển E ⇄ V ({}) đang bị 'Đổi phím' sang phím khác trước khi phím tắt kích hoạt.",
                        key_name, switch_name
                    ),
                });
            } else if active_key_macros.iter().any(|m| m.trigger_vk == vk) {
                conflicts.push(KeyConflict {
                    vk,
                    key_name: key_name.clone(),
                    winner: ConflictFeature::KeyMacro,
                    shadowed: ConflictFeature::SwitchKey,
                    reason: format!(
                        "Phím chính [{}] của Phím chuyển E ⇄ V ({}) trùng với 'Phím Macro' và bị nuốt trước.",
                        key_name, switch_name
                    ),
                });
            }
        }

        // 4B. Check Switch Key Modifiers (Ctrl, Shift, Alt, Win) against Disable & Remap
        let mut check_mod = |is_used: bool, mod_vks: &[u32], mod_name: &str| {
            if !is_used {
                return;
            }
            for &m_vk in mod_vks {
                if active_disabled.iter().any(|d| d.vk == m_vk) {
                    conflicts.push(KeyConflict {
                        vk: m_vk,
                        key_name: mod_name.to_string(),
                        winner: ConflictFeature::Disable,
                        shadowed: ConflictFeature::SwitchKey,
                        reason: format!(
                            "Phím bổ trợ [{}] của Phím chuyển E ⇄ V ({}) đang bị 'Chặn phím' nuốt.",
                            mod_name, switch_name
                        ),
                    });
                    break;
                } else if active_remaps.iter().any(|r| r.from_vk == m_vk) {
                    conflicts.push(KeyConflict {
                        vk: m_vk,
                        key_name: mod_name.to_string(),
                        winner: ConflictFeature::Remap,
                        shadowed: ConflictFeature::SwitchKey,
                        reason: format!(
                            "Phím bổ trợ [{}] của Phím chuyển E ⇄ V ({}) đang bị 'Đổi phím' sang phím khác.",
                            mod_name, switch_name
                        ),
                    });
                    break;
                }
            }
        };

        check_mod(switch_key.ctrl, &[0x11, 0xA2, 0xA3], "Ctrl");
        check_mod(switch_key.shift, &[0x10, 0xA0, 0xA1], "Shift");
        check_mod(switch_key.alt, &[0x12, 0xA4, 0xA5], "Alt");
        check_mod(switch_key.win, &[0x5B, 0x5C], "Win");
    }

    KeyDiagnostics {
        conflicts,
        total_remap,
        active_remap,
        total_disable,
        active_disable,
        total_key_macro,
        active_key_macro,
        switch_key_enabled,
        switch_key,
    }
}
