//! UI Framework Design Tokens (Typography, Spacing, Borders, Radii, Layout)

/// Typography tokens (Font sizes and weights)
pub mod typography {
    pub const FONT_SIZE_HERO: i32 = 18;
    pub const FONT_SIZE_TITLE: i32 = 13;
    pub const FONT_SIZE_SIDEBAR_TITLE: i32 = 11;
    pub const FONT_SIZE_BODY: i32 = 9;
    pub const FONT_SIZE_SUBTITLE: i32 = 9;
}

/// Spacing scale (used for margins, paddings, gaps, and offsets)
pub mod spacing {
    pub const SPACE_4: i32 = 4;
    pub const SPACE_8: i32 = 8;
    pub const SPACE_10: i32 = 10;
    pub const SPACE_12: i32 = 12;
    pub const SPACE_14: i32 = 14;
    pub const SPACE_16: i32 = 16;
}

/// Border tokens (border widths)
pub mod borders {
    pub const WIDTH_THIN: i32 = 1;
    pub const WIDTH_FOCUS: i32 = 2;
}

/// Corner radius tokens (used with RoundRect for borders and fills)
pub mod radii {
    pub const RADIUS_TINY: i32 = 2; // Small indicators, checkboxes (13x13)
    pub const RADIUS_MD: i32 = 6; // Standard buttons, list items, badge pills
    pub const RADIUS_CARD: i32 = 8; // Main content cards, tabs
    pub const RADIUS_LG: i32 = 10; // Hero logos, dialog containers
    pub const RADIUS_PILL: i32 = 12; // Segmented tab bar track, pill badges
}

/// Standard 2-column Sidebar + Card Panel layout metrics
pub mod layout {
    use super::spacing;

    // Window Client Area
    pub const WINDOW_CLIENT_WIDTH: i32 = 820;
    pub const WINDOW_CLIENT_HEIGHT: i32 = 590;

    // Sidebar dimensions
    pub const SIDEBAR_X: i32 = spacing::SPACE_8;
    pub const SIDEBAR_WIDTH: i32 = 214;
    pub const SIDEBAR_DIVIDER_X: i32 = 230;
    pub const SIDEBAR_LIST_Y: i32 = 44;
    pub const SIDEBAR_LIST_HEIGHT: i32 = 452;
    pub const SIDEBAR_ITEM_HEIGHT: i32 = 48;
    pub const SIDEBAR_SEPARATOR_Y: i32 = 504;
    pub const SIDEBAR_BTN_SETTINGS_Y: i32 = 512;
    pub const SIDEBAR_BTN_ABOUT_Y: i32 = 548;
    pub const SIDEBAR_BTN_HEIGHT: i32 = 32;

    // Unified Header Section
    pub const HEADER_X: i32 = 246;
    pub const HEADER_TITLE_Y: i32 = spacing::SPACE_14;
    pub const HEADER_TITLE_WIDTH: i32 = 500;
    pub const HEADER_TITLE_HEIGHT: i32 = 26;

    pub const HEADER_SUB_Y: i32 = 42;
    pub const HEADER_SUB_WIDTH: i32 = 500;
    pub const HEADER_SUB_HEIGHT: i32 = 20;

    pub const HEADER_DIVIDER_Y: i32 = 72;

    // Subtabs Bar
    pub const TAB_BAR_X: i32 = 244;
    pub const TAB_BAR_Y: i32 = 66;
    pub const TAB_BAR_WIDTH: i32 = 562;
    pub const TAB_BAR_HEIGHT: i32 = 32;

    // Content cards & bounds
    pub const CARD_MARGIN_RIGHT: i32 = spacing::SPACE_14;

    // Subtab Content Cards
    pub const CARD_X: i32 = TAB_BAR_X;
    pub const CARD_WIDTH: i32 = TAB_BAR_WIDTH;
    pub const CARD_START_Y: i32 = TAB_BAR_Y + TAB_BAR_HEIGHT + spacing::SPACE_14; // 112
    pub const CARD_GAP: i32 = spacing::SPACE_12; // 12
    pub const CARD_PAD_X: i32 = spacing::SPACE_16; // 16
    pub const CARD_CONTENT_X: i32 = CARD_X + CARD_PAD_X; // 260
    pub const CARD_CONTENT_WIDTH: i32 = CARD_WIDTH - CARD_PAD_X * 2; // 530
    pub const CARD_BOTTOM_Y: i32 = FOOTER_Y - spacing::SPACE_16; // 532

    // Footer dimensions
    pub const FOOTER_Y: i32 = 548;
}
