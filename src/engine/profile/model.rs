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

/// Vietnamese Charset mode: Inherit or Specific
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetMode {
    #[default]
    Inherit,
    Specific(crate::vietnamese::Charset),
}

impl CharsetMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Specific(c) => c.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed == "inherit" || trimmed.is_empty() {
            Self::Inherit
        } else {
            let c: crate::vietnamese::Charset = trimmed.parse().unwrap_or_default();
            Self::Specific(c)
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

/// Single action within a Key Macro automation sequence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroAction {
    Press(String),
    KeyDown(String),
    KeyUp(String),
    Delay(u32),
    Text(String),
}

impl MacroAction {
    pub fn action_type_str(&self) -> &'static str {
        match self {
            Self::Press(_) => "press",
            Self::KeyDown(_) => "down",
            Self::KeyUp(_) => "up",
            Self::Delay(_) => "delay",
            Self::Text(_) => "text",
        }
    }

    pub fn value_str(&self) -> String {
        match self {
            Self::Press(s) | Self::KeyDown(s) | Self::KeyUp(s) | Self::Text(s) => s.clone(),
            Self::Delay(ms) => ms.to_string(),
        }
    }

    pub fn to_dsl(&self) -> String {
        match self {
            Self::Press(s) => format!("press({})", s),
            Self::KeyDown(s) => format!("down({})", s),
            Self::KeyUp(s) => format!("up({})", s),
            Self::Delay(ms) => format!("delay({})", ms),
            Self::Text(s) => format!("text({})", s),
        }
    }

    pub fn actions_to_dsl(actions: &[MacroAction]) -> String {
        if actions.is_empty() {
            return String::new();
        }
        actions
            .iter()
            .map(|a| a.to_dsl())
            .collect::<Vec<_>>()
            .join(" + ")
    }

    pub fn dsl_to_actions(dsl: &str) -> Vec<MacroAction> {
        let mut actions = Vec::new();
        let trimmed = dsl.trim();
        if trimmed.is_empty() {
            return actions;
        }

        let mut chars = trimmed.char_indices().peekable();
        while let Some(&(start_idx, ch)) = chars.peek() {
            if ch.is_whitespace() || ch == '+' {
                chars.next();
                continue;
            }

            let open_paren = match trimmed[start_idx..].find('(') {
                Some(pos) => start_idx + pos,
                None => break,
            };
            let func_name = trimmed[start_idx..open_paren].trim().to_ascii_lowercase();

            let close_paren = match trimmed[open_paren + 1..].find(')') {
                Some(pos) => open_paren + 1 + pos,
                None => break,
            };
            let content = trimmed[open_paren + 1..close_paren].trim();

            match func_name.as_str() {
                "press" => actions.push(MacroAction::Press(content.to_string())),
                "down" | "keydown" => actions.push(MacroAction::KeyDown(content.to_string())),
                "up" | "keyup" => actions.push(MacroAction::KeyUp(content.to_string())),
                "delay" | "sleep" | "wait" => {
                    let ms = content.parse::<u32>().unwrap_or(0);
                    actions.push(MacroAction::Delay(ms));
                }
                "text" => actions.push(MacroAction::Text(content.to_string())),
                _ => {}
            }

            while let Some(&(i, _)) = chars.peek() {
                chars.next();
                if i >= close_paren {
                    break;
                }
            }
        }

        actions
    }
}

/// Single hardware key macro automation rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMacroRule {
    pub trigger_vk: u32,
    pub trigger_name: String,
    pub name: String,
    pub actions: Vec<MacroAction>,
}

impl KeyMacroRule {
    pub fn new(trigger_vk: u32, name: impl Into<String>, actions: Vec<MacroAction>) -> Self {
        Self {
            trigger_vk,
            trigger_name: crate::engine::config::hotkey::vk_to_name(trigger_vk).to_string(),
            name: name.into(),
            actions,
        }
    }

    pub fn with_trigger_name(
        trigger_vk: u32,
        trigger_name: String,
        name: impl Into<String>,
        actions: Vec<MacroAction>,
    ) -> Self {
        Self {
            trigger_vk,
            trigger_name,
            name: name.into(),
            actions,
        }
    }
}

/// Resolved item for Key Macro table in UI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedKeyMacroItem {
    pub trigger_vk: u32,
    pub trigger_name: String,
    pub name: String,
    pub actions: Vec<MacroAction>,
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

/// Typestate marker for the Global Root profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Root;

/// Typestate marker for an Application Child profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Child;

/// Trait associating each compile-time profile role marker `R` with its exact payload `R::Data`.
pub trait Role {
    type Data: Clone + std::fmt::Debug;
}

/// Payload for [`AppProfile<Root>`] (`global_profile`):
/// Stores only global rule lists and root feature flags without child inheritance deltas.
#[derive(Debug, Clone)]
pub struct RootProfileData {
    pub id: String,
    pub icon: String,
    pub remap_enabled: bool,
    pub muted_remaps: Vec<u32>,
    pub app_remaps: Vec<KeyRemapRule>,
    pub disable_enabled: bool,
    pub unmuted_disabled: Vec<u32>,
    pub app_disabled: Vec<u32>,
    pub muted_macros: Vec<String>,
    pub key_macro_enabled: bool,
    pub muted_key_macros: Vec<u32>,
    pub app_key_macros: Vec<KeyMacroRule>,
}

/// Payload for [`AppProfile<Child>`]:
/// Stores child application metadata and delta inheritance overrides (`None` = inherit from Global).
#[derive(Debug, Clone)]
pub struct ChildProfileData {
    pub id: String,
    pub name: String,
    pub process_names: Vec<String>,
    pub icon: String,

    // 1. Vietnamese Typing & Language mode
    pub mode: LanguageMode,
    pub method: MethodMode,
    pub charset: CharsetMode,
    pub switch_key: Option<Hotkey>,
    pub switch_key_enabled: Option<bool>,
    pub target_capture_hotkey: Option<Hotkey>,
    pub target_capture_enabled: Option<bool>,
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

    // 6. Key Macro (Hardware Key Automation, Delta inheritance)
    pub key_macro_enabled: Option<bool>,
    pub muted_key_macros: Vec<u32>,
    pub overridden_key_macros: HashMap<u32, (String, Vec<MacroAction>)>,
    pub app_key_macros: Vec<KeyMacroRule>,

    // 7. Surgical target key injection rules
    pub app_strategy: Option<KeyInjectionStrategy>,
    pub target_rules: Vec<TargetRule>,
}

/// Key injection strategy for target text fields (e.g. Omnibox, Excel cell, code editor, game console)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyInjectionStrategy {
    #[default]
    AutocompleteGuard,
    ClipboardPaste,
}

impl KeyInjectionStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AutocompleteGuard => "autocomplete_guard",
            Self::ClipboardPaste => "clipboard_paste",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "clipboard_paste" | "clipboard" | "paste" => Self::ClipboardPaste,
            _ => Self::AutocompleteGuard,
        }
    }

    pub fn to_raw_u8(&self) -> u8 {
        match self {
            Self::AutocompleteGuard => 1,
            Self::ClipboardPaste => 2,
        }
    }

    pub fn from_raw_u8(val: u8) -> Self {
        match val {
            2 | 4 => Self::ClipboardPaste,
            _ => Self::AutocompleteGuard,
        }
    }

    pub fn localized_label(&self, s: &crate::language::LanguageStrings) -> &'static str {
        match self {
            Self::AutocompleteGuard => s.strategy_autocomplete_guard,
            Self::ClipboardPaste => s.strategy_clipboard_paste,
        }
    }

    pub fn localized_desc(&self, s: &crate::language::LanguageStrings) -> &'static str {
        match self {
            Self::AutocompleteGuard => s.strategy_desc_autocomplete_guard,
            Self::ClipboardPaste => s.strategy_desc_clipboard_paste,
        }
    }
}

/// Surgical text-field key injection target rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRule {
    pub target_id: String,
    pub strategy: KeyInjectionStrategy,
}

impl TargetRule {
    pub fn new(target_id: impl Into<String>, strategy: KeyInjectionStrategy) -> Self {
        Self {
            target_id: target_id.into(),
            strategy,
        }
    }
}

impl Role for Root {
    type Data = RootProfileData;
}

impl Role for Child {
    type Data = ChildProfileData;
}

/// Borrowed reference to either the Global Root profile or an Application Child profile.
#[derive(Debug, Clone, Copy)]
pub enum ProfileRef<'a> {
    Root(&'a AppProfile<Root>),
    Child(&'a AppProfile<Child>),
}

impl<'a> ProfileRef<'a> {
    #[inline]
    pub fn id(self) -> &'a str {
        match self {
            Self::Root(p) => &p.id,
            Self::Child(p) => &p.id,
        }
    }
}

/// Application profile model parameterized by role typestate (`Root` or `Child`).
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct AppProfile<R: Role = Child> {
    pub data: R::Data,
    pub _role: std::marker::PhantomData<R>,
}

impl<R: Role> std::ops::Deref for AppProfile<R> {
    type Target = R::Data;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<R: Role> std::ops::DerefMut for AppProfile<R> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl AppProfile<Root> {
    /// Creates the global root profile (`AppProfile<Root>`).
    pub fn new_global() -> Self {
        Self {
            data: RootProfileData {
                id: "global".to_string(),
                icon: "🌐".to_string(),
                remap_enabled: false,
                muted_remaps: Vec::new(),
                app_remaps: Vec::new(),
                disable_enabled: false,
                unmuted_disabled: Vec::new(),
                app_disabled: Vec::new(),
                muted_macros: Vec::new(),
                key_macro_enabled: false,
                muted_key_macros: Vec::new(),
                app_key_macros: Vec::new(),
            },
            _role: std::marker::PhantomData,
        }
    }
}

impl AppProfile<Child> {
    /// Creates a new child application profile (`AppProfile<Child>`) inheriting everything from Global by default.
    pub fn new_child(
        id: impl Into<String>,
        name: impl Into<String>,
        process: impl Into<String>,
        icon: impl Into<String>,
    ) -> Self {
        let p_name = process.into();
        let process_names = if p_name.trim().is_empty() {
            Vec::new()
        } else {
            vec![p_name.trim().to_lowercase()]
        };

        Self {
            data: ChildProfileData {
                id: id.into(),
                name: name.into(),
                process_names,
                icon: icon.into(),
                mode: LanguageMode::Inherit,
                method: MethodMode::Inherit,
                charset: CharsetMode::Inherit,
                switch_key: None,
                switch_key_enabled: None,
                target_capture_hotkey: None,
                target_capture_enabled: None,
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
                key_macro_enabled: None,
                muted_key_macros: Vec::new(),
                overridden_key_macros: HashMap::new(),
                app_key_macros: Vec::new(),
                app_strategy: None,
                target_rules: Vec::new(),
            },
            _role: std::marker::PhantomData,
        }
    }

    /// Checks if this child profile matches an executable name.
    pub fn matches_process(&self, exe: &str) -> bool {
        let exe_lower = exe.trim().to_lowercase();
        self.process_names
            .iter()
            .any(|p| p.to_lowercase() == exe_lower)
    }
}
