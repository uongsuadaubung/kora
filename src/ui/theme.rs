//! Kora Application Theme Consumer (`src/ui/theme.rs`)
//!
//! Kết nối cấu hình `UiTheme` của Engine và `ControlPanelControls` với `crate::ui_framework::theme`.

pub use crate::ui_framework::theme::*;

use crate::engine::config::UiTheme;
use crate::ui::views::ControlPanelControls;

/// Resolves whether dark mode should be active given the theme setting
pub fn resolve_is_dark(theme: UiTheme) -> bool {
    match theme {
        UiTheme::Auto => crate::ui::tray::is_windows_dark_apps(),
        UiTheme::Light => false,
        UiTheme::Dark => true,
    }
}

/// Applies cohesive, modern Light or Dark theme styling to all Control Panel elements.
/// `apply_window_theme` automatically enumerates and themes all child components
/// (including `ActionTable`, `Slider`, `PushButton`, `CheckBox`, `ComboBox`, `TextBox`, `Label`).
pub fn apply_ui_theme(hwnd: isize, _controls: &ControlPanelControls, is_dark: bool) {
    apply_window_theme(hwnd, is_dark);
}
