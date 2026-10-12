//! Integration test suite for Kora typing engine.
//! Split into domain-specific modules for clean maintenance.

#[path = "engine_test/common.rs"]
mod common;

#[path = "engine_test/backspace.rs"]
mod backspace;
#[path = "engine_test/english_bypass.rs"]
mod english_bypass;
#[path = "engine_test/macro_rules.rs"]
mod macro_rules;
#[path = "engine_test/spelling.rs"]
mod spelling;
#[path = "engine_test/system_ui.rs"]
mod system_ui;
#[path = "engine_test/typing.rs"]
mod typing;
