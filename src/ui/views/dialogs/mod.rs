//! Modal Dialog Views for Kora UI

pub mod add_profile;
pub mod confirm;
pub mod disable_key;
pub mod hotkey;
pub mod macro_editor;
pub mod remap_key;

pub use add_profile::show_add_profile_dialog;
pub use confirm::{show_confirm_dialog, ConfirmDialogOptions};
pub use disable_key::show_disable_dialog;
pub use hotkey::show_hotkey_dialog;
pub use macro_editor::{show_macro_dialog, MacroEditData};
pub use remap_key::{show_remap_dialog, show_remap_dialog_with_keys};

