//! Kora Vietnamese Typing Engine in Rust
//!
//! A clean, memory-safe, modular rewrite of Vietnamese input engine.
//! Designed to eliminate global state, buffer corruption, and case-mismatch bugs.
#![allow(non_snake_case)]

pub mod engine;
pub mod language;
pub mod logger;
pub mod platform;
#[cfg(windows)]
pub mod ui;
#[cfg(windows)]
pub mod ui_framework;
pub mod updater;
pub mod vietnamese;

pub use engine::{
    VietnameseEngine,
    action::{CompactString, EngineAction},
    config::{EngineConfig, Hotkey, InputMethod, UiTheme},
    config_store::{
        flush_config_debounced, get_config_path, load_config_all, load_config_and_macros,
        parse_config_all, parse_config_and_macros, save_config_all, save_config_all_debounced,
        save_config_and_macros, save_config_and_macros_debounced, serialize_config_all,
        serialize_config_and_macros, set_windows_autostart,
    },
    macro_table::MacroTable,
    profile::{
        AppProfile, CharsetMode, KeyRemapRule, LanguageMode, MethodMode, ProfileManager, RuleOrigin,
    },
};
pub use language::{Language, LanguageStrings, current as current_strings, get_strings};
pub use vietnamese::{
    BaseVowel, Charset, CharsetStrategy, Diacritic, Tcvn3Charset, Tone, UnicodeCharset,
};
