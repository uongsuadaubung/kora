//! Modal Dialog Views for Kora UI

pub mod add_profile;
pub mod base;
pub mod confirm;
pub mod disable_key;
pub mod hotkey;
pub mod key_macro;
pub mod macro_editor;
pub mod remap_key;

pub use add_profile::show_add_profile_dialog;
pub use base::*;
pub use confirm::{
    show_confirm_dialog, show_danger_confirm, show_delete_confirm, ConfirmDialogOptions,
};
pub use disable_key::show_disable_dialog;
pub use hotkey::{show_hotkey_dialog, show_target_capture_hotkey_dialog};
pub use key_macro::{show_key_macro_dialog, show_key_macro_dialog_with_key};
pub use macro_editor::{show_macro_dialog, MacroEditData};
pub use remap_key::{show_remap_dialog, show_remap_dialog_with_keys};
