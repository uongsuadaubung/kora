//! Persistent configuration store and debounced I/O for Kora

pub mod codec;
pub mod debounced;
pub mod paths;

pub use codec::*;
pub use debounced::*;
pub use paths::*;

use crate::engine::config::EngineConfig;
use crate::engine::macro_table::MacroTable;
use crate::engine::profile::ProfileManager;
use std::fs;

/// Loads EngineConfig, MacroTable, and ProfileManager from disk, falling back to defaults if not found.
/// Automatically sets the UI localization language based on the loaded configuration.
pub fn load_config_all() -> (EngineConfig, MacroTable, ProfileManager) {
    let path = get_config_path();
    let (config, macros, profiles) = if !path.exists() {
        let config = EngineConfig::default();
        let macros = MacroTable::with_defaults();
        let profiles = ProfileManager::new();
        let _ = save_config_all(&config, &macros, &profiles);
        (config, macros, profiles)
    } else {
        match fs::read_to_string(&path) {
            Ok(content) => parse_config_all(&content),
            Err(_) => (
                EngineConfig::default(),
                MacroTable::with_defaults(),
                ProfileManager::new(),
            ),
        }
    };
    crate::language::set_current_language(config.language);
    (config, macros, profiles)
}

/// Loads EngineConfig and MacroTable from disk (legacy helper).
pub fn load_config_and_macros() -> (EngineConfig, MacroTable) {
    let (config, macros, _) = load_config_all();
    (config, macros)
}
