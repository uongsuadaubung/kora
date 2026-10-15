//! UI Framework Theme Color Definitions
//! Centralized color constants for Light and Dark modes.
//! All colors in the UI framework and application MUST be defined here in ThemePalette.
//! No component should construct or hardcode arbitrary RGB colors.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    /// Window background (dialog background, header, footer)
    pub bg_window: u32,
    /// Sidebar background
    pub bg_sidebar: u32,
    /// Card background (tab interior card body)
    pub bg_card: u32,
    /// Header background for listviews/tables
    pub bg_card_header: u32,
    /// Input background (edit boxes, list views)
    pub bg_input: u32,
    /// Primary text color (labels, checkboxes, button texts, table header text)
    pub text_primary: u32,
    /// Secondary / muted text color (descriptions, hints, placeholder cue banner)
    pub text_secondary: u32,
    /// Text inside edit boxes and list views
    pub text_input: u32,
    /// Border color for cards, listviews, frames
    pub border: u32,
    /// Subtle divider line between columns or sections
    pub border_separator: u32,
    /// Segmented tab track background
    pub tab_track_bg: u32,
    /// Segmented tab track border
    pub tab_track_border: u32,
    /// Active tab pill background
    pub tab_active_bg: u32,
    /// Active tab pill border
    pub tab_active_border: u32,
    /// Inactive tab hover background
    pub tab_hover_bg: u32,
    /// Hyperlink text color (email, github links)
    pub text_link: u32,
    /// Accent color (primary brand, active buttons, pills)
    pub accent: u32,
    /// Accent hover / pressed color
    pub accent_hover: u32,
    /// Root badge pill background
    pub badge_root_bg: u32,
    /// Pure white text (buttons, badges, icons)
    pub text_white: u32,
    /// Active status indicator dot (green)
    pub indicator_active: u32,
    /// Hover border for secondary buttons and controls
    pub border_hover: u32,
    /// Row hover background for tables / list views
    pub row_hover_bg: u32,
    /// Row selected background for tables / list views
    pub row_selected_bg: u32,
    /// Row selected text color for tables / list views
    pub row_selected_text: u32,
    /// Danger button background
    pub danger_bg: u32,
    /// Danger button hover background
    pub danger_bg_hover: u32,
    /// Danger button border
    pub danger_border: u32,
    /// Danger button text
    pub danger_text: u32,
    /// Danger button hover text
    pub danger_text_hover: u32,
    /// Info / blue badge background
    pub info_bg: u32,
    /// Info badge border
    pub info_border: u32,
    /// Info badge text
    pub info_text: u32,
    /// Warning / amber badge background
    pub warning_bg: u32,
    /// Warning badge border
    pub warning_border: u32,
    /// Warning badge text
    pub warning_text: u32,
    /// Success / green badge background
    pub success_bg: u32,
    /// Success badge border
    pub success_border: u32,
    /// Success badge text
    pub success_text: u32,
    /// Purple / violet badge background
    pub purple_bg: u32,
    /// Purple badge border
    pub purple_border: u32,
    /// Purple badge text
    pub purple_text: u32,
}

/// Private helper to pack RGB bytes into a Win32 COLORREF (0x00BBGGRR).
/// Strictly private to this module so all colors are centralized in ThemePalette.
const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// Dark Theme Palette (Windows 11 Fluent Dark inspired)
pub const DARK_PALETTE: ThemePalette = ThemePalette {
    bg_window: rgb(32, 32, 32),
    bg_sidebar: rgb(26, 26, 28),
    bg_card: rgb(43, 43, 44),
    bg_card_header: rgb(36, 36, 38),
    bg_input: rgb(48, 48, 50),
    text_primary: rgb(245, 245, 245),
    text_secondary: rgb(170, 170, 170),
    text_input: rgb(255, 255, 255),
    border: rgb(68, 68, 70),
    border_separator: rgb(58, 58, 62),
    tab_track_bg: rgb(36, 36, 38),
    tab_track_border: rgb(52, 52, 56),
    tab_active_bg: rgb(56, 56, 60),
    tab_active_border: rgb(76, 76, 82),
    tab_hover_bg: rgb(44, 44, 48),
    text_link: rgb(88, 166, 255),
    accent: rgb(24, 135, 230),
    accent_hover: rgb(0, 99, 177),
    badge_root_bg: rgb(24, 45, 68),
    text_white: rgb(255, 255, 255),
    indicator_active: rgb(50, 205, 50),
    border_hover: rgb(101, 101, 101),
    row_hover_bg: rgb(56, 56, 60),
    row_selected_bg: rgb(64, 68, 76),
    row_selected_text: rgb(255, 255, 255),
    danger_bg: rgb(43, 27, 27),
    danger_bg_hover: rgb(50, 31, 31),
    danger_border: rgb(216, 68, 58),
    danger_text: rgb(255, 119, 102),
    danger_text_hover: rgb(255, 136, 120),
    info_bg: rgb(20, 38, 60),
    info_border: rgb(35, 75, 120),
    info_text: rgb(88, 166, 255),
    warning_bg: rgb(50, 36, 18),
    warning_border: rgb(110, 80, 28),
    warning_text: rgb(245, 175, 65),
    success_bg: rgb(18, 42, 26),
    success_border: rgb(35, 90, 52),
    success_text: rgb(75, 210, 120),
    purple_bg: rgb(40, 25, 55),
    purple_border: rgb(90, 50, 125),
    purple_text: rgb(195, 145, 250),
};

/// Light Theme Palette (Windows 11 Fluent Light inspired)
pub const LIGHT_PALETTE: ThemePalette = ThemePalette {
    bg_window: rgb(243, 243, 243),
    bg_sidebar: rgb(235, 235, 238),
    bg_card: rgb(255, 255, 255),
    bg_card_header: rgb(240, 240, 243),
    bg_input: rgb(255, 255, 255),
    text_primary: rgb(25, 25, 25),
    text_secondary: rgb(100, 100, 100),
    text_input: rgb(25, 25, 25),
    border: rgb(220, 220, 224),
    border_separator: rgb(215, 215, 220),
    tab_track_bg: rgb(234, 234, 238),
    tab_track_border: rgb(220, 220, 224),
    tab_active_bg: rgb(255, 255, 255),
    tab_active_border: rgb(212, 212, 216),
    tab_hover_bg: rgb(242, 242, 246),
    text_link: rgb(0, 102, 204),
    accent: rgb(0, 120, 212),
    accent_hover: rgb(0, 99, 177),
    badge_root_bg: rgb(228, 240, 255),
    text_white: rgb(255, 255, 255),
    indicator_active: rgb(16, 124, 65),
    border_hover: rgb(176, 176, 176),
    row_hover_bg: rgb(245, 247, 250),
    row_selected_bg: rgb(228, 240, 255),
    row_selected_text: rgb(25, 25, 25),
    danger_bg: rgb(253, 232, 232),
    danger_bg_hover: rgb(254, 226, 226),
    danger_border: rgb(216, 68, 58),
    danger_text: rgb(197, 21, 21),
    danger_text_hover: rgb(197, 21, 21),
    info_bg: rgb(230, 242, 255),
    info_border: rgb(180, 215, 255),
    info_text: rgb(0, 102, 204),
    warning_bg: rgb(255, 248, 230),
    warning_border: rgb(245, 215, 140),
    warning_text: rgb(180, 100, 10),
    success_bg: rgb(232, 250, 238),
    success_border: rgb(160, 230, 185),
    success_text: rgb(16, 124, 65),
    purple_bg: rgb(248, 238, 255),
    purple_border: rgb(220, 185, 250),
    purple_text: rgb(125, 45, 185),
};

impl ThemePalette {
    pub const fn get(is_dark: bool) -> &'static ThemePalette {
        if is_dark {
            &DARK_PALETTE
        } else {
            &LIGHT_PALETTE
        }
    }
}

