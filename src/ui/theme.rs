//! Kora Application Theme Resolver

use crate::engine::config::UiTheme;

/// Resolves whether dark mode should be active given the theme setting
pub fn resolve_is_dark(theme: UiTheme) -> bool {
    match theme {
        UiTheme::Auto => crate::ui::tray::is_windows_dark_apps(),
        UiTheme::Light => false,
        UiTheme::Dark => true,
    }
}
