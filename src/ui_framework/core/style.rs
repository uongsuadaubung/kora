//! Bootstrap-Inspired Utility & Style System (Project-Agnostic)
//!
//! Provides semantic color variants (`Primary`, `Secondary`, `Success`, `Danger`, `Warning`, `Info`, `Purple`),
//! spacing scales (`0..=6` -> `0px..24px`), flexbox alignment primitives, and box-model borders/backgrounds.

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::tokens::radii;

/// Bootstrap-style semantic visual variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    #[default]
    Default,
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Info,
    Purple,
    Ghost,
}

impl Variant {
    /// Resolves `(bg_color, border_color, text_color)` for badges, alerts, and buttons.
    pub fn colors(self, palette: &ThemePalette, is_hover: bool) -> (u32, u32, u32) {
        match self {
            Self::Default | Self::Secondary => (
                if is_hover {
                    palette.row_hover_bg
                } else {
                    palette.bg_input
                },
                if is_hover {
                    palette.accent
                } else {
                    palette.border
                },
                if is_hover {
                    palette.accent
                } else {
                    palette.text_primary
                },
            ),
            Self::Primary => {
                let c = if is_hover {
                    palette.accent_hover
                } else {
                    palette.accent
                };
                (c, c, palette.text_white)
            }
            Self::Success => (
                palette.success_bg,
                palette.success_border,
                palette.success_text,
            ),
            Self::Danger => (
                if is_hover {
                    palette.danger_bg_hover
                } else {
                    palette.danger_bg
                },
                palette.danger_border,
                if is_hover {
                    palette.danger_text_hover
                } else {
                    palette.danger_text
                },
            ),
            Self::Warning => (
                palette.warning_bg,
                palette.warning_border,
                palette.warning_text,
            ),
            Self::Info => (palette.info_bg, palette.info_border, palette.info_text),
            Self::Purple => (
                palette.purple_bg,
                palette.purple_border,
                palette.purple_text,
            ),
            Self::Ghost => (
                if is_hover {
                    palette.tab_hover_bg
                } else {
                    palette.bg_card
                },
                if is_hover {
                    palette.border
                } else {
                    palette.bg_card
                },
                palette.text_secondary,
            ),
        }
    }
}

/// Component size presets (`btn-sm`, `btn-md`, `btn-lg`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    Sm,
    #[default]
    Md,
    Lg,
}

/// Flexbox cross-axis alignment (`align-items-*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

/// Flexbox main-axis justification (`justify-content-*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    Between,
}

/// Dimension sizing mode for width / height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Length {
    #[default]
    Auto,
    Fixed(i32),
    Flex(u16),
    Full,
}

/// Box padding / margin edges in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Edges {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Edges {
    pub const ZERO: Self = Self {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };

    #[inline]
    pub const fn all(v: i32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    #[inline]
    pub const fn xy(x: i32, y: i32) -> Self {
        Self {
            top: y,
            right: x,
            bottom: y,
            left: x,
        }
    }

    #[inline]
    pub const fn horizontal(self) -> i32 {
        self.left + self.right
    }

    #[inline]
    pub const fn vertical(self) -> i32 {
        self.top + self.bottom
    }
}

/// Converts Bootstrap spacing scale `0..=6` into pixels (`0, 4, 8, 12, 16, 20, 24`).
/// Values `> 6` are treated directly as pixel values for convenience.
#[inline]
pub const fn spacing_scale(scale: i32) -> i32 {
    match scale {
        0 => 0,
        1 => 4,
        2 => 8,
        3 => 12,
        4 => 16,
        5 => 20,
        6 => 24,
        px => px,
    }
}

/// Semantic surface background role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BgStyle {
    #[default]
    None,
    Window,
    Sidebar,
    Card,
    CardHeader,
    Input,
    TabActive,
    TabHover,
    Subtle(Variant),
}

impl BgStyle {
    pub fn resolve(self, palette: &ThemePalette) -> Option<u32> {
        match self {
            Self::None => None,
            Self::Window => Some(palette.bg_window),
            Self::Sidebar => Some(palette.bg_sidebar),
            Self::Card => Some(palette.bg_card),
            Self::CardHeader => Some(palette.bg_card_header),
            Self::Input => Some(palette.bg_input),
            Self::TabActive => Some(palette.tab_active_bg),
            Self::TabHover => Some(palette.tab_hover_bg),
            Self::Subtle(v) => match v {
                Variant::Primary | Variant::Info => Some(palette.info_bg),
                Variant::Secondary | Variant::Default => Some(palette.bg_card),
                Variant::Success => Some(palette.success_bg),
                Variant::Danger => Some(palette.danger_bg),
                Variant::Warning => Some(palette.warning_bg),
                Variant::Purple => Some(palette.purple_bg),
                Variant::Ghost => None,
            },
        }
    }
}

/// Semantic border style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Normal,
    Subtle,
    Accent,
    Dashed,
    Variant(Variant),
}

impl BorderStyle {
    pub fn resolve(self, palette: &ThemePalette) -> Option<(u32, bool)> {
        match self {
            Self::None => None,
            Self::Normal => Some((palette.border, false)),
            Self::Subtle => Some((palette.border_separator, false)),
            Self::Accent => Some((palette.accent, false)),
            Self::Dashed => Some((palette.border, true)),
            Self::Variant(v) => match v {
                Variant::Primary | Variant::Info => Some((palette.info_border, false)),
                Variant::Secondary | Variant::Default => Some((palette.border, false)),
                Variant::Success => Some((palette.success_border, false)),
                Variant::Danger => Some((palette.danger_border, false)),
                Variant::Warning => Some((palette.warning_border, false)),
                Variant::Purple => Some((palette.purple_border, false)),
                Variant::Ghost => None,
            },
        }
    }
}

/// Typography tone & weight styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextTone {
    #[default]
    Primary,
    Secondary,
    Accent,
    White,
    Variant(Variant),
}

impl TextTone {
    pub fn resolve(self, palette: &ThemePalette) -> u32 {
        match self {
            Self::Primary => palette.text_primary,
            Self::Secondary => palette.text_secondary,
            Self::Accent => palette.accent,
            Self::White => palette.text_white,
            Self::Variant(v) => v.colors(palette, false).2,
        }
    }
}

/// Unified Box-Model & Flexbox Style attached to every [`super::Element`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub padding: Edges,
    pub gap: i32,
    pub width: Length,
    pub height: Length,
    pub min_w: i32,
    pub min_h: i32,
    pub align: Align,
    pub justify: Justify,
    pub bg: BgStyle,
    pub border: BorderStyle,
    pub border_bottom: bool,
    pub border_top: bool,
    pub border_left_accent: Option<(i32, Variant)>,
    pub radius: i32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            padding: Edges::ZERO,
            gap: 0,
            width: Length::Auto,
            height: Length::Auto,
            min_w: 0,
            min_h: 0,
            align: Align::Start,
            justify: Justify::Start,
            bg: BgStyle::None,
            border: BorderStyle::None,
            border_bottom: false,
            border_top: false,
            border_left_accent: None,
            radius: radii::RADIUS_CARD,
        }
    }
}

