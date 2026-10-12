//! Engine Configuration options and models

pub mod hotkey;
pub mod input_method;
pub mod theme;

pub use hotkey::*;
pub use input_method::*;
pub use theme::*;

/// Unified configuration options for the Vietnamese engine and application preferences.
///
/// Aliased as [`AppConfig`] to reflect its role as the root application configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineConfig {
    // --- 1. Typing Engine Core Settings ---
    pub method: InputMethod,
    /// Bật/tắt chế độ gõ tiếng Việt (true: bật gõ tiếng Việt, false: tiếng Anh)
    pub enabled: bool,
    /// Tự động khôi phục phím nếu gõ sai chính tả
    pub restore_on_wrong_spelling: bool,
    /// Tự động viết hoa chữ cái đầu câu sau dấu chấm / Enter
    pub auto_uppercase_first_char: bool,
    /// Phím gõ tắt ngoặc vuông: [ -> ư, ] -> ơ
    pub bracket_w: bool,
    /// Bật tính năng gõ tắt (Macro) - Mặc định TẮT
    pub use_macro: bool,
    /// Cho phép gõ tắt cả khi ở chế độ tiếng Anh
    pub use_macro_in_english_mode: bool,
    /// Khôi phục từ khi bấm Backspace xoá dấu cách
    pub remember_history_across_space: bool,

    // --- 2. Mode Switch Hotkey ---
    /// Bật/tắt sử dụng phím tắt chuyển chế độ
    pub switch_key_enabled: bool,
    /// Phím chuyển chế độ (mặc định Ctrl + Shift, hoặc bất kỳ phím nào người dùng cài đặt)
    pub switch_key: Hotkey,
    /// Tương thích ngược: true nếu switch_key là Ctrl + Shift
    pub switch_with_ctrl_shift: bool,

    // --- 2.5 Key Remapping & Disabling ---
    /// Bật/tắt tính năng đổi phím (Key Remapping)
    pub remap_enabled: bool,
    /// Bật/tắt tính năng khóa/vô hiệu hóa phím (Disable Keys)
    pub disable_enabled: bool,

    // --- 3. Sound Effects (WASAPI) ---
    /// Bật/tắt âm thanh phím cơ
    pub sound_enabled: bool,
    /// Tên profile switch phím cơ (tương ứng với tên thư mục trong ~/.config/kora/switches/)
    pub sound_profile: std::sync::Arc<str>,
    /// Âm lượng phím cơ (0% - 100%)
    pub sound_volume: u8,

    // --- 4. Application & UI Preferences ---
    /// Giao diện ứng dụng (Auto / Light / Dark)
    pub theme: UiTheme,
    /// Ngôn ngữ giao diện (Tiếng Việt / English)
    pub language: crate::language::Language,
    /// Bật hội thoại này khi khởi động cùng Windows
    pub show_dialog_on_startup: bool,

    // --- 5. Diagnostics & Logging ---
    /// Bật chế độ debug log (ghi nhận vết từng phím, trạng thái state machine và output)
    pub debug: bool,
    /// Đường dẫn file lưu nhật ký debug log trên ổ đĩa
    pub debug_file_path: Option<std::sync::Arc<str>>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            method: InputMethod::Telex,
            theme: UiTheme::Auto,
            language: crate::language::Language::default(),
            enabled: true,
            restore_on_wrong_spelling: true,
            auto_uppercase_first_char: false,
            bracket_w: true,
            use_macro: false,
            use_macro_in_english_mode: true,
            remember_history_across_space: true,
            switch_key_enabled: true,
            switch_key: Hotkey::default(),
            switch_with_ctrl_shift: true,
            remap_enabled: false,
            disable_enabled: false,
            sound_enabled: false,
            sound_profile: std::sync::Arc::from("Holy Panda"),
            sound_volume: 50,
            show_dialog_on_startup: true,
            debug: false,
            debug_file_path: Some(std::sync::Arc::from("kora_debug.log")),
        }
    }
}
