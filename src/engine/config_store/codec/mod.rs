//! INI Parser and Serializer for EngineConfig, MacroTable, and ProfileManager
//! Supports Live Inheritance format with [remap], [disable], and [profile:<id>] sections.

pub mod decoder;
pub mod encoder;

pub use decoder::{parse_config_all, parse_config_and_macros};
pub use encoder::{serialize_config_all, serialize_config_and_macros};
