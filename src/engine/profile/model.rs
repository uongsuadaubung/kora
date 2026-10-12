//! Data models for Profile-First Keyboard Workspace Hub
//!
//! Supports Live Inheritance (Delta Overlay) model:
//! Global is the Single Source of Truth.
//! Child profiles only store deltas (overrides, app-specific additions, and muted tombstones).

use crate::engine::config::{Hotkey, InputMethod};
use crate::engine::macro_table::{MacroEntry, MacroType};
use std::collections::HashMap;

/// Language mode: Inherit from Global, Vietnamese, or English
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageMode {
    #[default]
    Inherit,
    Vietnamese,
    English,
}

impl LanguageMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Vietnamese => "vietnamese",
            Self::English => "english",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "vietnamese" | "viet" | "vn" | "vi" => Self::Vietnamese,
            "english" | "eng" | "en" => Self::English,
            _ => Self::Inherit,
        }
    }
}

/// Vietnamese Input Method mode: Inherit or Specific
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MethodMode {
    #[default]
    Inherit,
    Specific(InputMethod),
}

impl MethodMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Specific(m) => m.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed == "inherit" || trimmed.is_empty() {
            Self::Inherit
        } else {
            let m: InputMethod = trimmed.parse().unwrap_or_default();
            Self::Specific(m)
        }
    }
}

/// Single key remapping rule (Virtual Key Code and friendly name)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRemapRule {
    pub from_vk: u32,
    pub to_vk: u32,
    pub from_name: String,
    pub to_name: String,
}

impl KeyRemapRule {
    pub fn new(from_vk: u32, to_vk: u32) -> Self {
        Self {
            from_vk,
            to_vk,
            from_name: crate::engine::config::hotkey::vk_to_name(from_vk).to_string(),
            to_name: crate::engine::config::hotkey::vk_to_name(to_vk).to_string(),
        }
    }

    pub fn with_names(from_vk: u32, to_vk: u32, from_name: String, to_name: String) -> Self {
        Self {
            from_vk,
            to_vk,
            from_name,
            to_name,
        }
    }
}

/// Origin status of a rule (used for UI status badge in 4-column ListView)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleOrigin {
    Root,
    InheritedGlobal,
    OverriddenGlobal,
    MutedGlobal,
    AppSpecific,
}

impl RuleOrigin {
    pub fn badge_label(&self) -> &'static str {
        let s = crate::language::current();
        self.localized_badge_label(s)
    }

    pub fn localized_badge_label(&self, s: &crate::language::LanguageStrings) -> &'static str {
        match self {
            Self::Root => "",
            Self::InheritedGlobal => s.origin_inherited,
            Self::OverriddenGlobal => s.origin_overridden,
            Self::MutedGlobal => s.origin_muted,
            Self::AppSpecific => s.origin_app_specific,
        }
    }
}

/// Resolved item for Remap table in UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRemapItem {
    pub from_vk: u32,
    pub to_vk: u32,
    pub from_name: String,
    pub to_name: String,
    pub origin: RuleOrigin,
    pub is_active: bool,
}

/// Resolved item for Disable table in UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDisableItem {
    pub vk: u32,
    pub key_name: String,
    pub origin: RuleOrigin,
    pub is_active: bool,
}

/// Resolved item for Macro table in UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMacroItem {
    pub key: String,
    pub value: String,
    pub macro_type: MacroType,
    pub origin: RuleOrigin,
    pub is_active: bool,
}

/// Application profile model
#[derive(Debug, Clone)]
pub struct AppProfile {
    pub id: String,
    pub name: String,
    pub process_names: Vec<String>,
    pub icon: String,
    pub is_root: bool,

    // 1. Vietnamese Typing & Language mode
    pub mode: LanguageMode,
    pub method: MethodMode,
    pub switch_key: Option<Hotkey>,
    pub switch_key_enabled: Option<bool>,
    pub restore_on_wrong_spelling: Option<bool>,
    pub auto_uppercase_first_char: Option<bool>,
    pub bracket_w: Option<bool>,
    pub remember_history_across_space: Option<bool>,

    // 2. Key Remapping (Delta inheritance)
    pub remap_enabled: Option<bool>,
    pub muted_remaps: Vec<u32>,
    pub overridden_remaps: HashMap<u32, u32>,
    pub app_remaps: Vec<KeyRemapRule>,

    // 3. Key Disabling (Delta inheritance)
    pub disable_enabled: Option<bool>,
    pub unmuted_disabled: Vec<u32>,
    pub app_disabled: Vec<u32>,

    // 4. Typing Sound (Mechanical Switch)
    pub sound_enabled: Option<bool>,
    pub sound_profile: Option<std::sync::Arc<str>>,
    pub sound_volume: Option<u8>,

    // 5. Macro (Delta inheritance)
    pub use_macro: Option<bool>,
    pub use_macro_in_english_mode: Option<bool>,
    pub muted_macros: Vec<String>,
    pub overridden_macros: HashMap<String, (String, MacroType)>,
    pub app_macros: Vec<MacroEntry>,
}

impl AppProfile {
    /// Creates the global root profile
    pub fn new_global() -> Self {
        Self {
            id: "global".to_string(),
            name: "Toàn hệ thống".to_string(),
            process_names: Vec::new(),
            icon: "🌐".to_string(),
            is_root: true,
            mode: LanguageMode::Vietnamese,
            method: MethodMode::Specific(InputMethod::Telex),
            switch_key: Some(Hotkey::default()),
            switch_key_enabled: Some(true),
            restore_on_wrong_spelling: Some(true),
            auto_uppercase_first_char: Some(false),
            bracket_w: Some(true),
            remember_history_across_space: Some(true),
            remap_enabled: Some(false),
            muted_remaps: Vec::new(),
            overridden_remaps: HashMap::new(),
            app_remaps: Vec::new(),
            unmuted_disabled: Vec::new(),
            disable_enabled: Some(false),
            app_disabled: Vec::new(),
            sound_enabled: Some(false),
            sound_profile: Some(std::sync::Arc::from("Holy Panda")),
            sound_volume: Some(50),
            use_macro: Some(false),
            use_macro_in_english_mode: Some(true),
            muted_macros: Vec::new(),
            overridden_macros: HashMap::new(),
            app_macros: Vec::new(),
        }
    }

    /// Creates a new child application profile inheriting everything from Global by default
    pub fn new_child(id: impl Into<String>, name: impl Into<String>, process: impl Into<String>, icon: impl Into<String>) -> Self {
        let p_name = process.into();
        let process_names = if p_name.trim().is_empty() {
            Vec::new()
        } else {
            vec![p_name.trim().to_lowercase()]
        };

        Self {
            id: id.into(),
            name: name.into(),
            process_names,
            icon: icon.into(),
            is_root: false,
            mode: LanguageMode::Inherit,
            method: MethodMode::Inherit,
            switch_key: None,
            switch_key_enabled: None,
            restore_on_wrong_spelling: None,
            auto_uppercase_first_char: None,
            bracket_w: None,
            remember_history_across_space: None,
            remap_enabled: None,
            muted_remaps: Vec::new(),
            overridden_remaps: HashMap::new(),
            app_remaps: Vec::new(),
            disable_enabled: None,
            unmuted_disabled: Vec::new(),
            app_disabled: Vec::new(),
            sound_enabled: None,
            sound_profile: None,
            sound_volume: None,
            use_macro: None,
            use_macro_in_english_mode: None,
            muted_macros: Vec::new(),
            overridden_macros: HashMap::new(),
            app_macros: Vec::new(),
        }
    }

    /// Checks if this profile matches an executable name
    pub fn matches_process(&self, exe: &str) -> bool {
        let exe_lower = exe.trim().to_lowercase();
        self.process_names.iter().any(|p| p.to_lowercase() == exe_lower)
    }
}
