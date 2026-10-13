//! Multi-language & Localization (i18n) subsystem for Kora
//! Eliminates hardcoded UI strings and provides type-safe, zero-allocation localized texts.

pub mod en;
pub mod vi;

use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};

/// Supported user interface languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Vietnamese,
    English,
}

impl Language {
    /// Two-letter ISO code for configuration storage
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Vietnamese => "vi",
            Self::English => "en",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for Language {
    fn from(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Self::English,
            _ => Self::Vietnamese,
        }
    }
}

impl FromStr for Language {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<Language> for usize {
    fn from(l: Language) -> Self {
        match l {
            Language::Vietnamese => 0,
            Language::English => 1,
        }
    }
}

impl From<usize> for Language {
    fn from(idx: usize) -> Self {
        match idx {
            1 => Self::English,
            _ => Self::Vietnamese,
        }
    }
}

/// Comprehensive compile-time checked localized string dictionary for Kora UI
pub struct LanguageStrings {
    // Window & General
    pub window_title: &'static str,
    pub helper_window_title: &'static str,

    // Header Card
    pub label_method: &'static str,
    pub method_telex: &'static str,
    pub method_vni: &'static str,
    pub method_simple_telex: &'static str,
    pub label_mode: &'static str,
    pub mode_vietnamese: &'static str,
    pub mode_english: &'static str,
    pub label_switch: &'static str,
    pub check_ctrl_shift: &'static str,
    pub check_switch_key: &'static str,

    // Navigation TabBar
    pub tab_typing: &'static str,
    pub tab_remap: &'static str,
    pub tab_sound: &'static str,
    pub tab_macro: &'static str,
    pub tab_system: &'static str,
    pub tab_about: &'static str,

    // Tab 0: Typing & Sound
    pub typing_title: &'static str,
    pub check_restore_wrong: &'static str,
    pub check_auto_upper: &'static str,
    pub check_inherit_on: &'static str,
    pub check_inherit_off: &'static str,
    pub sound_title: &'static str,
    pub check_sound_enabled: &'static str,
    pub label_switch_type: &'static str,
    pub label_volume: &'static str,
    pub btn_test_sound: &'static str,
    pub sound_profile_default: &'static str,
    pub soundpack_card_title: &'static str,
    pub btn_download_soundpack: &'static str,
    pub btn_downloading_soundpack: &'static str,
    pub btn_update_soundpack: &'static str,
    pub soundpack_note_missing: &'static str,
    pub soundpack_note_installed: &'static str,
    pub soundpack_download_success: &'static str,
    pub soundpack_download_error: &'static str,

    // Tab 1: Macro
    pub check_use_macro: &'static str,
    pub check_macro_in_english: &'static str,
    pub edit_macro_key_placeholder: &'static str,
    pub edit_macro_val_placeholder: &'static str,
    pub btn_add_macro: &'static str,
    pub btn_edit_macro: &'static str,
    pub btn_cancel_macro: &'static str,
    pub btn_del_macro: &'static str,
    pub col_macro_key: &'static str,
    pub col_macro_val: &'static str,
    pub col_macro_type: &'static str,
    pub col_action: &'static str,
    pub action_edit: &'static str,
    pub action_delete: &'static str,
    pub action_mute: &'static str,
    pub action_unmute: &'static str,
    pub action_combo: &'static str,
    pub macro_type_normal: &'static str,
    pub macro_type_start: &'static str,
    pub macro_type_end: &'static str,

    // Tab 2: System
    pub sys_header_title: &'static str,
    pub sys_header_sub: &'static str,
    pub sys_title: &'static str,
    pub check_autostart: &'static str,
    pub check_show_dialog: &'static str,
    pub check_debug_log: &'static str,
    pub btn_open_log: &'static str,
    pub btn_check_update: &'static str,
    pub btn_checking_update: &'static str,
    pub label_theme_title: &'static str,
    pub label_theme: &'static str,
    pub theme_auto: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    pub label_language: &'static str,

    // Tab 3: About
    pub about_header_title: &'static str,
    pub about_header_sub: &'static str,
    pub about_title: &'static str,
    pub about_ver: &'static str,
    pub about_author: &'static str,
    pub about_email: &'static str,
    pub about_github: &'static str,
    pub about_engine_desc: &'static str,
    pub about_github_btn: &'static str,

    // Footer Buttons
    pub btn_exit: &'static str,
    pub btn_defaults: &'static str,
    pub btn_close: &'static str,

    // System Settings - Defaults Card
    pub sys_defaults_title: &'static str,
    pub sys_defaults_desc: &'static str,
    pub btn_restore_defaults: &'static str,
    pub confirm_defaults_title: &'static str,
    pub confirm_defaults_msg: &'static str,

    // Hotkey Capture Dialog
    pub hotkey_dialog_title: &'static str,
    pub hotkey_dialog_prompt: &'static str,
    pub hotkey_press_prompt: &'static str,
    pub hotkey_selected_title: &'static str,
    pub hotkey_click_hint: &'static str,
    pub hotkey_presets_title: &'static str,
    pub hotkey_preset_ctrl_shift: &'static str,
    pub hotkey_preset_alt_z: &'static str,
    pub hotkey_preset_win_space: &'static str,
    pub hotkey_preset_shift_space: &'static str,
    pub hotkey_listening: &'static str,
    pub hotkey_apply_prefix: &'static str,
    pub hotkey_btn_save: &'static str,
    pub btn_save: &'static str,
    pub btn_clear: &'static str,
    pub btn_cancel: &'static str,

    // System Tray Menu & Tooltips
    pub tray_tooltip_vi: &'static str,
    pub tray_tooltip_en: &'static str,
    pub tray_toggle_vi: &'static str,
    pub tray_method_telex: &'static str,
    pub tray_method_vni: &'static str,
    pub tray_method_simple_telex: &'static str,
    pub tray_control_panel: &'static str,
    pub tray_exit: &'static str,

    // Sidebar
    pub sidebar_profiles_title: &'static str,
    pub sidebar_root_desc: &'static str,
    pub sidebar_unassigned: &'static str,
    pub sidebar_btn_settings: &'static str,
    pub sidebar_btn_about: &'static str,

    // Profile Banner
    pub banner_root_title: &'static str,
    pub banner_root_desc: &'static str,
    pub banner_btn_delete: &'static str,
    pub banner_status_active_prefix: &'static str,
    pub banner_status_inactive_prefix: &'static str,

    // Tab 0: Typing
    pub typing_group_main: &'static str,
    pub typing_group_switch: &'static str,
    pub typing_group_features: &'static str,
    pub typing_lbl_mode: &'static str,
    pub typing_lbl_method: &'static str,
    pub typing_lbl_current_key: &'static str,
    pub typing_btn_change_key: &'static str,
    pub typing_inherit_global: &'static str,
    pub typing_inherit_prefix: &'static str,

    // Tab 2: Remap, Disable & Key Macro
    pub remap_table_title: &'static str,
    pub disable_table_title: &'static str,
    pub key_macro_table_title: &'static str,
    pub keys_sub_remap: &'static str,
    pub keys_sub_disable: &'static str,
    pub keys_sub_macro: &'static str,
    pub status_active: &'static str,
    pub status_disabled: &'static str,
    pub status_blocked: &'static str,
    pub status_unblocked: &'static str,
    pub origin_inherited: &'static str,
    pub origin_overridden: &'static str,
    pub origin_muted: &'static str,
    pub origin_app_specific: &'static str,
    pub col_remap_from: &'static str,
    pub col_remap_to: &'static str,
    pub col_origin: &'static str,
    pub col_status: &'static str,
    pub col_disable_key: &'static str,
    pub col_key_macro_trigger: &'static str,
    pub col_key_macro_name: &'static str,
    pub btn_add_remap: &'static str,
    pub btn_del_remap: &'static str,
    pub btn_add_disable: &'static str,
    pub btn_del_disable: &'static str,
    pub btn_add_key_macro: &'static str,
    pub btn_del_key_macro: &'static str,

    // Dialogs
    pub dlg_add_prof_title: &'static str,
    pub dlg_add_prof_name: &'static str,
    pub dlg_add_prof_name_placeholder: &'static str,
    pub dlg_add_prof_icon: &'static str,
    pub dlg_add_prof_icon_game: &'static str,
    pub dlg_add_prof_icon_code: &'static str,
    pub dlg_add_prof_icon_office: &'static str,
    pub dlg_add_prof_icon_graphics: &'static str,
    pub dlg_add_prof_icon_browser: &'static str,
    pub dlg_add_prof_icon_tools: &'static str,
    pub dlg_add_prof_linked_app: &'static str,
    pub dlg_add_prof_hint: &'static str,
    pub dlg_add_prof_browse: &'static str,
    pub dlg_add_prof_rescan: &'static str,
    pub dlg_add_prof_search_placeholder: &'static str,
    pub dlg_add_prof_footer: &'static str,
    pub dlg_add_prof_exe: &'static str,
    pub dlg_add_prof_exe_placeholder: &'static str,
    pub dlg_add_prof_btn_create: &'static str,
    pub dlg_remap_title: &'static str,
    pub dlg_remap_prompt: &'static str,
    pub dlg_remap_from: &'static str,
    pub dlg_remap_to: &'static str,
    pub dlg_remap_click_hint: &'static str,
    pub dlg_remap_unassigned: &'static str,
    pub dlg_remap_listening: &'static str,
    pub dlg_remap_presets_title: &'static str,
    pub dlg_remap_btn_apply: &'static str,
    pub dlg_remap_btn_cancel: &'static str,
    pub dlg_disable_title: &'static str,
    pub dlg_disable_prompt: &'static str,
    pub dlg_disable_key: &'static str,
    pub dlg_disable_click_hint: &'static str,
    pub dlg_disable_presets_title: &'static str,
    pub dlg_disable_btn_apply: &'static str,
    pub dlg_disable_btn_cancel: &'static str,
    pub dlg_key_capturing: &'static str,
    pub dlg_btn_apply: &'static str,
    pub dlg_btn_cancel: &'static str,
    pub dlg_macro_title_add: &'static str,
    pub dlg_macro_title_edit: &'static str,
    pub dlg_macro_lbl_key: &'static str,
    pub dlg_macro_lbl_val: &'static str,
    pub dlg_macro_lbl_type: &'static str,

    // Key Macro Dialog
    pub dlg_key_macro_title_add: &'static str,
    pub dlg_key_macro_title_edit: &'static str,
    pub dlg_key_macro_trigger_lbl: &'static str,
    pub dlg_key_macro_trigger_hint: &'static str,
    pub dlg_key_macro_name_lbl: &'static str,
    pub dlg_key_macro_name_placeholder: &'static str,
    pub dlg_key_macro_actions_lbl: &'static str,
    pub dlg_key_macro_col_index: &'static str,
    pub dlg_key_macro_col_type: &'static str,
    pub dlg_key_macro_col_val: &'static str,
    pub dlg_key_macro_col_order: &'static str,
    pub dlg_key_macro_col_action: &'static str,
    pub dlg_key_macro_empty_actions: &'static str,
    pub dlg_key_macro_insert_title: &'static str,
    pub dlg_key_macro_btn_add_step: &'static str,
    pub dlg_key_macro_dsl_preview: &'static str,

    // Confirm Dialog
    pub confirm_delete_title: &'static str,
    pub confirm_delete_macro_msg: &'static str,
    pub confirm_delete_key_macro_msg: &'static str,
    pub confirm_delete_remap_msg: &'static str,
    pub confirm_delete_disable_msg: &'static str,
    pub confirm_delete_profile_msg: &'static str,
    pub btn_yes: &'static str,
    pub btn_no: &'static str,

    // Updater
    pub update_title: &'static str,
    pub update_error_title: &'static str,
    pub update_available_msg: &'static str,
    pub update_latest_msg: &'static str,
    pub update_error_msg: &'static str,
}

impl LanguageStrings {
    pub const fn profile_icons(&self) -> [&'static str; 6] {
        [
            self.dlg_add_prof_icon_game,
            self.dlg_add_prof_icon_code,
            self.dlg_add_prof_icon_office,
            self.dlg_add_prof_icon_graphics,
            self.dlg_add_prof_icon_browser,
            self.dlg_add_prof_icon_tools,
        ]
    }

    pub const fn tab_titles(&self) -> [&'static str; 4] {
        [
            self.tab_typing,
            self.tab_remap,
            self.tab_sound,
            self.tab_macro,
        ]
    }
}

static CURRENT_LANG: AtomicU8 = AtomicU8::new(0);

/// Returns the static string dictionary for the given language
pub fn get_strings(lang: Language) -> &'static LanguageStrings {
    match lang {
        Language::Vietnamese => &vi::STRINGS,
        Language::English => &en::STRINGS,
    }
}

/// Gets the currently active language
pub fn current_language() -> Language {
    match CURRENT_LANG.load(Ordering::Relaxed) {
        1 => Language::English,
        _ => Language::Vietnamese,
    }
}

/// Sets the currently active language
pub fn set_current_language(lang: Language) {
    let val = match lang {
        Language::Vietnamese => 0,
        Language::English => 1,
    };
    CURRENT_LANG.store(val, Ordering::Relaxed);
}

/// Convenience helper to fetch the active localized strings
pub fn current() -> &'static LanguageStrings {
    get_strings(current_language())
}
