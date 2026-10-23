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

/// Dark Theme Palette (Modern Dark Slate & GitHub Dark Dimmed inspired)
/// Balances soft charcoal slate backgrounds with distinct elevated card surfaces,
/// eliminating the oppressive, murky pitch-black feel while remaining soothing on the eyes.
pub const DARK_PALETTE: ThemePalette = ThemePalette {
    bg_window: rgb(34, 39, 46),         // #22272E - Soothing deep slate canvas
    bg_sidebar: rgb(28, 33, 40),        // #1C2128 - Subtle inset sidebar
    bg_card: rgb(45, 51, 59),           // #2D333B - Clearly elevated card layer
    bg_card_header: rgb(37, 42, 49),    // #252A31 - Distinct header for cards/tables
    bg_input: rgb(28, 33, 40),          // #1C2128 - Inset text input with crisp border
    text_primary: rgb(230, 237, 243),   // #E6EDF3 - Soft crisp off-white, anti-glare
    text_secondary: rgb(139, 148, 158), // #8B949E - Balanced muted text
    text_input: rgb(240, 246, 252),     // #F0F6FC - High-clarity input text
    border: rgb(68, 76, 86),            // #444C56 - Crisp 1px boundary
    border_separator: rgb(55, 62, 71),  // #373E47 - Subtle divider
    tab_track_bg: rgb(28, 33, 40),      // #1C2128 - Inset pill track
    tab_track_border: rgb(55, 62, 71),  // #373E47 - Track border
    tab_active_bg: rgb(55, 62, 71),     // #373E47 - Active elevated tab pill
    tab_active_border: rgb(84, 93, 105), // #545D69 - Subtle tab border
    tab_hover_bg: rgb(45, 51, 59),      // #2D333B - Hover highlight
    text_link: rgb(88, 166, 255),       // #58A6FF - Fluent link blue
    accent: rgb(47, 129, 247),          // #2F81F7 - Radiant electric blue
    accent_hover: rgb(56, 139, 253),    // #388BFD - Vibrant blue hover
    badge_root_bg: rgb(30, 50, 80),     // Inset blue badge for root
    text_white: rgb(255, 255, 255),
    indicator_active: rgb(63, 185, 80), // #3FB950 - Modern vibrant green
    border_hover: rgb(118, 131, 144),   // #768390 - Highlighted border on hover
    row_hover_bg: rgb(55, 62, 71),      // #373E47
    row_selected_bg: rgb(34, 68, 115),  // Highlighted row selection
    row_selected_text: rgb(255, 255, 255),
    danger_bg: rgb(73, 27, 29), // Rich wine red
    danger_bg_hover: rgb(90, 32, 35),
    danger_border: rgb(248, 81, 73), // Vibrant danger border
    danger_text: rgb(255, 123, 114), // Crisp danger text
    danger_text_hover: rgb(255, 161, 155),
    info_bg: rgb(27, 60, 104), // Deep navy
    info_border: rgb(56, 139, 253),
    info_text: rgb(121, 192, 255),
    warning_bg: rgb(77, 56, 0), // Warm amber
    warning_border: rgb(210, 153, 34),
    warning_text: rgb(227, 179, 65),
    success_bg: rgb(22, 59, 38), // Forest emerald
    success_border: rgb(46, 160, 67),
    success_text: rgb(86, 211, 100),
    purple_bg: rgb(56, 35, 84), // Royal violet
    purple_border: rgb(163, 113, 247),
    purple_text: rgb(210, 168, 255),
};

/// Light Theme Palette (Modern Clean Slate & GitHub Light inspired)
/// Features a subtle cool-gray backdrop (#F6F8FA) that makes crisp white (#FFFFFF)
/// cards and containers pop with clean, professional depth and zero eye strain.
pub const LIGHT_PALETTE: ThemePalette = ThemePalette {
    bg_window: rgb(246, 248, 250),      // #F6F8FA - Subtle off-white canvas
    bg_sidebar: rgb(238, 242, 246),     // #EEF2F6 - Soft gray sidebar
    bg_card: rgb(255, 255, 255),        // #FFFFFF - Crisp elevated white card
    bg_card_header: rgb(246, 248, 250), // #F6F8FA - Soft header
    bg_input: rgb(255, 255, 255),       // #FFFFFF - Pure white input
    text_primary: rgb(31, 35, 40),      // #1F2328 - Deep slate ink, easy on eyes
    text_secondary: rgb(101, 109, 118), // #656D76 - High legibility muted text
    text_input: rgb(31, 35, 40),        // #1F2328
    border: rgb(208, 215, 222),         // #D0D7DE - Crisp modern border
    border_separator: rgb(216, 222, 228), // #D8DEE4 - Subtle divider
    tab_track_bg: rgb(234, 238, 242),   // #EAEEF2 - Track container
    tab_track_border: rgb(208, 215, 222), // #D0D7DE
    tab_active_bg: rgb(255, 255, 255),  // #FFFFFF - Elevated active tab
    tab_active_border: rgb(208, 215, 222),
    tab_hover_bg: rgb(243, 244, 246),  // #F3F4F6
    text_link: rgb(9, 105, 218),       // #0969DA
    accent: rgb(9, 105, 218),          // #0969DA - Modern brand blue
    accent_hover: rgb(8, 96, 202),     // #0860CA
    badge_root_bg: rgb(221, 244, 255), // #DDF4FF
    text_white: rgb(255, 255, 255),
    indicator_active: rgb(26, 127, 55), // #1A7F37
    border_hover: rgb(140, 149, 159),   // #8C959F
    row_hover_bg: rgb(243, 245, 248),
    row_selected_bg: rgb(221, 244, 255),
    row_selected_text: rgb(9, 105, 218),
    danger_bg: rgb(255, 235, 233), // Soft pink-red
    danger_bg_hover: rgb(255, 220, 217),
    danger_border: rgb(255, 129, 130),
    danger_text: rgb(207, 34, 46),
    danger_text_hover: rgb(164, 15, 26),
    info_bg: rgb(221, 244, 255),
    info_border: rgb(84, 174, 255),
    info_text: rgb(9, 105, 218),
    warning_bg: rgb(255, 248, 197),
    warning_border: rgb(212, 167, 44),
    warning_text: rgb(154, 103, 0),
    success_bg: rgb(218, 251, 225),
    success_border: rgb(74, 194, 107),
    success_text: rgb(26, 127, 55),
    purple_bg: rgb(251, 239, 255),
    purple_border: rgb(209, 168, 255),
    purple_text: rgb(130, 80, 223),
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
