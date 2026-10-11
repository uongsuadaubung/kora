//! Centralized UI Design Tokens & Control Identifiers
//! Prevents magic numbers and coordinate mismatches across views and components.

/// Typography tokens (Font sizes and weights)
pub mod typography {
    pub const FONT_SIZE_HERO: i32 = 18;
    pub const FONT_SIZE_TITLE: i32 = 13;
    pub const FONT_SIZE_CARD_HEADER: i32 = 10;
    pub const FONT_SIZE_BODY: i32 = 9;
    pub const FONT_SIZE_SUBTITLE: i32 = 9;
    pub const FONT_SIZE_BADGE: i32 = 8;
}

/// Spacing scale (used for margins, paddings, gaps, and offsets)
pub mod spacing {
    pub const SPACE_2: i32 = 2;
    pub const SPACE_4: i32 = 4;
    pub const SPACE_6: i32 = 6;
    pub const SPACE_8: i32 = 8;
    pub const SPACE_10: i32 = 10;
    pub const SPACE_12: i32 = 12;
    pub const SPACE_14: i32 = 14;
    pub const SPACE_16: i32 = 16;
    pub const SPACE_20: i32 = 20;
    pub const SPACE_24: i32 = 24;
}

/// Border tokens (border widths)
pub mod borders {
    pub const WIDTH_THIN: i32 = 1;
    pub const WIDTH_FOCUS: i32 = 2;
}

/// Corner radius tokens (used with RoundRect for borders and fills)
pub mod radii {
    pub const RADIUS_TINY: i32 = 2;       // Small indicators, checkboxes (13x13)
    pub const RADIUS_SM: i32 = 4;         // Small tags, badges
    pub const RADIUS_MD: i32 = 6;         // Standard buttons, list items, badge pills
    pub const RADIUS_CARD: i32 = 8;       // Main content cards, tabs
    pub const RADIUS_LG: i32 = 10;        // Hero logos, dialog containers
    pub const RADIUS_PILL: i32 = 12;      // Segmented tab bar track, pill badges
}

/// Layout and coordinate tokens (Dimensions, Margins, and Positions)
pub mod layout {
    use super::{radii, spacing};

    // Window Client Area
    pub const WINDOW_CLIENT_WIDTH: i32 = 820;
    pub const WINDOW_CLIENT_HEIGHT: i32 = 590;

    // Sidebar dimensions
    pub const SIDEBAR_X: i32 = spacing::SPACE_8;
    pub const SIDEBAR_WIDTH: i32 = 214;
    pub const SIDEBAR_DIVIDER_X: i32 = 230;
    pub const SIDEBAR_TITLE_Y: i32 = spacing::SPACE_14;
    pub const SIDEBAR_TITLE_HEIGHT: i32 = 22;
    pub const SIDEBAR_LIST_Y: i32 = 44;
    pub const SIDEBAR_LIST_HEIGHT: i32 = 452;
    pub const SIDEBAR_ITEM_HEIGHT: i32 = 48;
    pub const SIDEBAR_SEPARATOR_Y: i32 = 504;
    pub const SIDEBAR_BTN_SETTINGS_Y: i32 = 512;
    pub const SIDEBAR_BTN_ABOUT_Y: i32 = 548;
    pub const SIDEBAR_BTN_HEIGHT: i32 = 32;

    // Unified Header Section (Used across ProfileBanner, TabSystem, and TabAbout)
    pub const HEADER_X: i32 = 246;
    pub const HEADER_TITLE_Y: i32 = spacing::SPACE_14;
    pub const HEADER_TITLE_WIDTH: i32 = 500;
    pub const HEADER_TITLE_HEIGHT: i32 = 26;

    pub const HEADER_SUB_Y: i32 = 42;
    pub const HEADER_SUB_WIDTH: i32 = 500;
    pub const HEADER_SUB_HEIGHT: i32 = 20;

    pub const HEADER_DIVIDER_Y: i32 = 72;

    // Profile Subtabs Bar
    pub const TAB_BAR_X: i32 = 244;
    pub const TAB_BAR_Y: i32 = 66;
    pub const TAB_BAR_WIDTH: i32 = 562;
    pub const TAB_BAR_HEIGHT: i32 = 32;

    // Content cards & bounds
    pub const CONTENT_X: i32 = 246;
    pub const CARD_MARGIN_RIGHT: i32 = spacing::SPACE_14;
    pub const CARD_CORNER_RADIUS: i32 = radii::RADIUS_CARD;

    // Profile Subtab Content Cards
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
    pub const FOOTER_HEIGHT: i32 = 32;
}

/// Centralized Control Identifiers (Win32 IDC_*)
/// Organized into distinct non-overlapping ranges:
/// - 300..399: Typing & Input Controls
/// - 400..419: Tab Bar & Macro Controls
/// - 420..429: System Settings Controls
/// - 430..439: About Software Controls
/// - 440..449: Sound Effects Controls
/// - 500..509: Sidebar & Profile Banner Controls
/// - 510..519: Remap & Disable Key Controls
/// - 600..609: Footer Actions & Hotkey Dialog
/// - 700..709: Add Profile Dialog
/// - 800..819: Remap / Disable Subdialogs
pub mod id {
    // 300..399: Typing & Input Controls
    pub const IDC_COMBO_TYPING_METHOD: u32 = 301;
    pub const IDC_COMBO_TYPING_MODE: u32 = 302;
    pub const IDC_BTN_SWITCH_KEY: u32 = 303;
    pub const IDC_CHECK_SWITCH_KEY: u32 = 304;
    pub const IDC_CHECK_CTRL_SHIFT: u32 = IDC_CHECK_SWITCH_KEY; // Alias
    pub const IDC_LABEL_SWITCH_KEY_BADGE: u32 = 305;
    pub const IDC_CHECK_RESTORE_WRONG: u32 = 402;
    pub const IDC_CHECK_FREE_MARKING: u32 = 403;
    pub const IDC_CHECK_BRACKET_W: u32 = 403; // Alias
    pub const IDC_CHECK_REMEMBER_HISTORY: u32 = 404;
    pub const IDC_CHECK_AUTO_UPPER: u32 = 405;

    // 400..419: Tab Bar & Macro Controls
    pub const IDC_TAB_MAIN: u32 = 400;
    pub const IDC_CHECK_USE_MACRO: u32 = 406;
    pub const IDC_EDIT_MACRO_KEY: u32 = 407;
    pub const IDC_EDIT_MACRO_VALUE: u32 = 408;
    pub const IDC_BTN_ADD_MACRO: u32 = 409;
    pub const IDC_LIST_MACRO: u32 = 410;
    pub const IDC_BTN_DEL_MACRO: u32 = 411;
    pub const IDC_BTN_CANCEL_MACRO: u32 = 412;
    pub const IDC_BTN_EDIT_MACRO: u32 = 413;
    pub const IDC_CHECK_MACRO_EN: u32 = 414;
    pub const IDC_COMBO_MACRO_TYPE: u32 = 417;

    // 420..429: System Settings Controls
    pub const IDC_CHECK_AUTOSTART: u32 = 420;
    pub const IDC_CHECK_DEBUG_LOG: u32 = 421;
    pub const IDC_BTN_OPEN_LOG: u32 = 422;
    pub const IDC_COMBO_THEME: u32 = 423;
    pub const IDC_CHECK_SHOW_DIALOG: u32 = 424;
    pub const IDC_COMBO_LANG: u32 = 425;
    pub const IDC_BTN_CHECK_UPDATE: u32 = 426;
    pub const IDC_SYS_HEADER_TITLE: u32 = 427;
    pub const IDC_SYS_HEADER_SUB: u32 = 428;
    pub const IDC_LABEL_SYS_TITLE: u32 = 429;

    // 430..439: About Software & Theme Card Labels
    pub const IDC_LABEL_THEME_TITLE: u32 = 430;
    pub const IDC_LABEL_THEME: u32 = 431;
    pub const IDC_LABEL_LANG: u32 = 432;
    pub const IDC_BTN_GITHUB: u32 = 433;
    pub const IDC_ABOUT_HEADER_TITLE: u32 = 434;
    pub const IDC_ABOUT_HEADER_SUB: u32 = 435;
    pub const IDC_ABOUT_APP_TITLE: u32 = 436;
    pub const IDC_ABOUT_APP_AUTHOR: u32 = 437;
    pub const IDC_ABOUT_APP_ENGINE: u32 = 438;

    // 440..449: Sound Effects Controls
    pub const IDC_CHECK_SOUND_ENABLED: u32 = 440;
    pub const IDC_COMBO_SWITCH_TYPE: u32 = 441;
    pub const IDC_SLIDER_SOUND_VOLUME: u32 = 442;
    pub const IDC_BTN_TEST_SOUND: u32 = 443;
    pub const IDC_LABEL_VOLUME_VAL: u32 = 444;

    // 500..509: Sidebar & Profile Banner Controls
    pub const IDC_SIDEBAR_TITLE: u32 = 500;
    pub const IDC_BTN_ADD_PROFILE: u32 = 501;
    pub const IDC_LIST_PROFILES: u32 = 502;
    pub const IDC_BTN_SIDEBAR_SETTINGS: u32 = 503;
    pub const IDC_BTN_SIDEBAR_ABOUT: u32 = 504;
    pub const IDC_BTN_DEL_PROFILE: u32 = 505;
    pub const IDC_BANNER_TITLE: u32 = 506;
    pub const IDC_BANNER_STATUS: u32 = 507;

    // 510..519: Remap & Disable Key Controls
    pub const IDC_BTN_ADD_REMAP: u32 = 510;
    pub const IDC_BTN_DEL_REMAP: u32 = 511;
    pub const IDC_LIST_REMAP: u32 = 512;
    pub const IDC_BTN_ADD_DISABLE: u32 = 513;
    pub const IDC_BTN_DEL_DISABLE: u32 = 514;
    pub const IDC_LIST_DISABLE: u32 = 515;
    pub const IDC_CHECK_REMAP_ENABLED: u32 = 516;
    pub const IDC_CHECK_DISABLE_ENABLED: u32 = 517;
    pub const IDC_TAB_KEYS_SUB: u32 = 518;

    // 600..609: Footer Actions & Hotkey Dialog
    pub const IDC_BTN_EXIT: u32 = 600;
    pub const IDC_BTN_DEFAULTS: u32 = 601;
    pub const IDC_BTN_CLOSE: u32 = 602;
    pub const IDC_BTN_HOTKEY_SAVE: u32 = 603;
    pub const IDC_BTN_HOTKEY_CLEAR: u32 = 604;
    pub const IDC_BTN_HOTKEY_CANCEL: u32 = 605;
    pub const IDC_LABEL_HOTKEY_DISPLAY: u32 = 606;
    pub const IDC_BTN_HOTKEY_PRESET_CTRL_SHIFT: u32 = 607;
    pub const IDC_BTN_HOTKEY_PRESET_ALT_Z: u32 = 608;
    pub const IDC_BTN_HOTKEY_PRESET_WIN_SPACE: u32 = 609;
    pub const IDC_BTN_HOTKEY_PRESET_SHIFT_SPACE: u32 = 610;
    pub const IDC_BTN_HOTKEY_PRESET_INHERIT: u32 = 611;

    // 700..709: Add Profile Dialog
    pub const IDC_BTN_CLOSE_ADD_PROFILE: u32 = 700;
    pub const IDC_EDIT_PROFILE_NAME: u32 = 701;
    pub const IDC_BTN_PROFILE_ICON: u32 = 702;
    pub const IDC_LIST_RUNNING_APPS: u32 = 703;
    pub const IDC_EDIT_PROFILE_EXE: u32 = 704;
    pub const IDC_BTN_CREATE_PROFILE: u32 = 705;
    pub const IDC_BTN_CANCEL_PROFILE: u32 = 706;
    pub const IDC_BTN_BROWSE_EXE: u32 = 707;
    pub const IDC_LINK_RESCAN_APPS: u32 = 708;
    pub const IDC_EDIT_SEARCH_APPS: u32 = 709;

    // 800..819: Remap / Disable Dialogs
    pub const IDC_BTN_REMAP_FROM: u32 = 801;
    pub const IDC_BTN_REMAP_TO: u32 = 802;
    pub const IDC_BTN_REMAP_APPLY: u32 = 803;
    pub const IDC_BTN_REMAP_CANCEL: u32 = 804;
    pub const IDC_BTN_REMAP_PRESET_CAPS_CTRL: u32 = 805;
    pub const IDC_BTN_REMAP_PRESET_CAPS_ESC: u32 = 806;
    pub const IDC_BTN_REMAP_PRESET_INS_DEL: u32 = 807;
    pub const IDC_BTN_REMAP_PRESET_F1_ESC: u32 = 808;
    pub const IDC_BTN_DISABLE_KEY: u32 = 811;
    pub const IDC_BTN_DISABLE_APPLY: u32 = 812;
    pub const IDC_BTN_DISABLE_CANCEL: u32 = 813;
    pub const IDC_BTN_DISABLE_PRESET_LWIN: u32 = 814;
    pub const IDC_BTN_DISABLE_PRESET_RWIN: u32 = 815;
    pub const IDC_BTN_DISABLE_PRESET_INS: u32 = 816;
    pub const IDC_BTN_DISABLE_PRESET_CAPS: u32 = 817;
    pub const IDC_BTN_DISABLE_PRESET_F1: u32 = 818;
    pub const IDC_BTN_DISABLE_PRESET_SCROLL: u32 = 819;

    // 820..829: Macro Dialog
    pub const IDC_MACRO_DLG_KEY: u32 = 821;
    pub const IDC_MACRO_DLG_VAL: u32 = 822;
    pub const IDC_MACRO_DLG_TYPE: u32 = 823;
    pub const IDC_MACRO_DLG_SAVE: u32 = 824;
    pub const IDC_MACRO_DLG_CANCEL: u32 = 825;

    // 830..839: Confirm Dialog
    pub const IDC_CONFIRM_BTN_YES: u32 = 831;
    pub const IDC_CONFIRM_BTN_NO: u32 = 832;
}

