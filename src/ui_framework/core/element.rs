//! React-Style Declarative Virtual Element Tree & Bootstrap Atomic Components
//!
//! All UI views are pure functions of state returning [`Element<M>`].
//! Components compose arbitrarily via `.child(...)` and `.children(...)` and are styled
//! via chainable Bootstrap utility modifiers (`.p(3)`, `.gap(2)`, `.justify_between()`, `.items_center()`, etc.).

pub use super::style::*;
use crate::ui_framework::tokens::radii;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontScale {
    #[default]
    Normal,
    Bold,
    Title,
    Hero,
    Small,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

pub enum WidgetKind<M> {
    Empty,
    Divider,
    Row(Vec<Element<M>>),
    Col(Vec<Element<M>>),
    Grid {
        cols: usize,
        children: Vec<Element<M>>,
    },
    Text {
        content: String,
        font: FontScale,
        tone: TextTone,
        align: TextAlign,
        wrap: bool,
        on_click: Option<M>,
    },
    Label {
        content: String,
        font: FontScale,
        tone: TextTone,
        align: TextAlign,
    },
    Badge {
        text: String,
        variant: Variant,
        is_pill: bool,
        disabled: bool,
        on_click: Option<M>,
    },
    Button {
        label: String,
        variant: Variant,
        size: Size,
        disabled: bool,
        on_click: Option<M>,
    },
    Switch {
        checked: bool,
        label: String,
        note: String,
        disabled: bool,
        on_toggle: Option<M>,
    },
    Checkbox {
        checked: bool,
        indeterminate: bool,
        label: String,
        note: String,
        disabled: bool,
        on_toggle: Option<M>,
    },
    Select {
        key: u32,
        selected: usize,
        options: Vec<String>,
        disabled: bool,
        on_select: Option<Arc<dyn Fn(usize) -> M + Send + Sync>>,
    },
    Slider {
        value: i32,
        min: i32,
        max: i32,
        disabled: bool,
        on_change: Option<Arc<dyn Fn(i32, bool) -> M + Send + Sync>>,
    },
    TabGroup {
        options: Vec<String>,
        selected: usize,
        on_select: Option<Arc<dyn Fn(usize) -> M + Send + Sync>>,
    },
    Input {
        key: u32,
        value: String,
        placeholder: String,
        disabled: bool,
        password: bool,
        auto_focus: bool,
        on_change: Option<Arc<dyn Fn(String) -> M + Send + Sync>>,
    },
}

/// A declarative UI node parameterized over the application message type `M`.
pub struct Element<M> {
    pub kind: WidgetKind<M>,
    pub style: Style,
    pub on_click: Option<M>,
}

impl<M> Element<M> {
    #[inline]
    pub fn new(kind: WidgetKind<M>) -> Self {
        Self {
            kind,
            style: Style::default(),
            on_click: None,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self.kind, WidgetKind::Empty)
            && matches!(self.style.width, Length::Auto)
            && matches!(self.style.height, Length::Auto)
    }

    // =========================================================================
    // Child Composition (React-style)
    // =========================================================================

    pub fn child(mut self, child: Element<M>) -> Self {
        if child.is_empty() {
            return self;
        }
        match &mut self.kind {
            WidgetKind::Row(list) | WidgetKind::Col(list) => list.push(child),
            WidgetKind::Grid { children, .. } => children.push(child),
            _ => {}
        }
        self
    }

    pub fn children<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = Element<M>>,
    {
        for c in iter {
            if !c.is_empty() {
                match &mut self.kind {
                    WidgetKind::Row(list) | WidgetKind::Col(list) => list.push(c),
                    WidgetKind::Grid { children, .. } => children.push(c),
                    _ => {}
                }
            }
        }
        self
    }

    // =========================================================================
    // Bootstrap Spacing Utilities (p, px, py, pt, pb, pl, pr, gap)
    // =========================================================================

    #[inline]
    pub fn p(mut self, scale: i32) -> Self {
        self.style.padding = Edges::all(spacing_scale(scale));
        self
    }

    #[inline]
    pub fn px(mut self, scale: i32) -> Self {
        let px = spacing_scale(scale);
        self.style.padding.left = px;
        self.style.padding.right = px;
        self
    }

    #[inline]
    pub fn py(mut self, scale: i32) -> Self {
        let py = spacing_scale(scale);
        self.style.padding.top = py;
        self.style.padding.bottom = py;
        self
    }

    #[inline]
    pub fn pt(mut self, scale: i32) -> Self {
        self.style.padding.top = spacing_scale(scale);
        self
    }

    #[inline]
    pub fn pb(mut self, scale: i32) -> Self {
        self.style.padding.bottom = spacing_scale(scale);
        self
    }

    #[inline]
    pub fn pl(mut self, scale: i32) -> Self {
        self.style.padding.left = spacing_scale(scale);
        self
    }

    #[inline]
    pub fn pr(mut self, scale: i32) -> Self {
        self.style.padding.right = spacing_scale(scale);
        self
    }

    #[inline]
    pub fn padding(mut self, edges: Edges) -> Self {
        self.style.padding = edges;
        self
    }

    #[inline]
    pub fn pad_xy(mut self, x_px: i32, y_px: i32) -> Self {
        self.style.padding = Edges::xy(x_px, y_px);
        self
    }

    #[inline]
    pub fn gap(mut self, scale: i32) -> Self {
        self.style.gap = spacing_scale(scale);
        self
    }

    // =========================================================================
    // Bootstrap Flexbox & Sizing Utilities
    // =========================================================================

    #[inline]
    pub fn w(mut self, px: i32) -> Self {
        self.style.width = Length::Fixed(px);
        self
    }

    #[inline]
    pub fn h(mut self, px: i32) -> Self {
        self.style.height = Length::Fixed(px);
        self
    }

    #[inline]
    pub fn min_w(mut self, px: i32) -> Self {
        self.style.min_w = px;
        self
    }

    #[inline]
    pub fn min_h(mut self, px: i32) -> Self {
        self.style.min_h = px;
        self
    }

    #[inline]
    pub fn w_full(mut self) -> Self {
        self.style.width = Length::Full;
        self
    }

    #[inline]
    pub fn flex(mut self, weight: u16) -> Self {
        self.style.width = Length::Flex(weight.max(1));
        self
    }

    #[inline]
    pub fn flex_1(self) -> Self {
        self.flex(1)
    }

    #[inline]
    pub fn h_flex(mut self, weight: u16) -> Self {
        self.style.height = Length::Flex(weight.max(1));
        self
    }

    // =========================================================================
    // Bootstrap 12-Column Grid Modifiers (col-12, col-6, col-4, col-3, etc.)
    // =========================================================================

    /// Bootstrap 12-column grid fraction: allocates `span` shares out of the total row weight.
    #[inline]
    pub fn col_span(self, span: u16) -> Self {
        self.flex(span)
    }

    /// Full width / 12 of 12 columns (100%).
    #[inline]
    pub fn col_12(self) -> Self {
        self.flex(12)
    }

    /// Half width / 6 of 12 columns (50%).
    #[inline]
    pub fn col_6(self) -> Self {
        self.flex(6)
    }

    /// One-third width / 4 of 12 columns (33.33%).
    #[inline]
    pub fn col_4(self) -> Self {
        self.flex(4)
    }

    /// One-fourth width / 3 of 12 columns (25%).
    #[inline]
    pub fn col_3(self) -> Self {
        self.flex(3)
    }

    /// Two-thirds width / 8 of 12 columns (66.67%).
    #[inline]
    pub fn col_8(self) -> Self {
        self.flex(8)
    }

    /// Three-fourths width / 9 of 12 columns (75%).
    #[inline]
    pub fn col_9(self) -> Self {
        self.flex(9)
    }

    #[inline]
    pub fn items_start(mut self) -> Self {
        self.style.align = Align::Start;
        self
    }

    #[inline]
    pub fn items_center(mut self) -> Self {
        self.style.align = Align::Center;
        self
    }

    #[inline]
    pub fn items_end(mut self) -> Self {
        self.style.align = Align::End;
        self
    }

    #[inline]
    pub fn items_stretch(mut self) -> Self {
        self.style.align = Align::Stretch;
        self
    }

    #[inline]
    pub fn justify_start(mut self) -> Self {
        self.style.justify = Justify::Start;
        self
    }

    #[inline]
    pub fn justify_center(mut self) -> Self {
        self.style.justify = Justify::Center;
        self
    }

    #[inline]
    pub fn justify_end(mut self) -> Self {
        self.style.justify = Justify::End;
        self
    }

    #[inline]
    pub fn justify_between(mut self) -> Self {
        self.style.justify = Justify::Between;
        self
    }

    // =========================================================================
    // Bootstrap Background, Border & Radius Utilities
    // =========================================================================

    #[inline]
    pub fn bg(mut self, bg: BgStyle) -> Self {
        self.style.bg = bg;
        self
    }

    #[inline]
    pub fn bg_card(mut self) -> Self {
        self.style.bg = BgStyle::Card;
        self
    }

    #[inline]
    pub fn bg_header(mut self) -> Self {
        self.style.bg = BgStyle::CardHeader;
        self
    }

    #[inline]
    pub fn bg_input(mut self) -> Self {
        self.style.bg = BgStyle::Input;
        self
    }

    #[inline]
    pub fn bg_window(mut self) -> Self {
        self.style.bg = BgStyle::Window;
        self
    }

    #[inline]
    pub fn bg_sidebar(mut self) -> Self {
        self.style.bg = BgStyle::Sidebar;
        self
    }

    #[inline]
    pub fn bg_tab_active(mut self) -> Self {
        self.style.bg = BgStyle::TabActive;
        self
    }

    #[inline]
    pub fn bg_subtle(mut self, variant: Variant) -> Self {
        self.style.bg = BgStyle::Subtle(variant);
        self
    }

    #[inline]
    pub fn border_style(mut self, border: BorderStyle) -> Self {
        self.style.border = border;
        self
    }

    #[inline]
    pub fn border(mut self) -> Self {
        self.style.border = BorderStyle::Normal;
        self
    }

    #[inline]
    pub fn border_subtle(mut self) -> Self {
        self.style.border = BorderStyle::Subtle;
        self
    }

    #[inline]
    pub fn border_accent(mut self) -> Self {
        self.style.border = BorderStyle::Accent;
        self
    }

    #[inline]
    pub fn border_dashed(mut self) -> Self {
        self.style.border = BorderStyle::Dashed;
        self
    }

    #[inline]
    pub fn border_variant(mut self, variant: Variant) -> Self {
        self.style.border = BorderStyle::Variant(variant);
        self
    }

    #[inline]
    pub fn border_bottom(mut self) -> Self {
        self.style.border_bottom = true;
        self
    }

    #[inline]
    pub fn border_top(mut self) -> Self {
        self.style.border_top = true;
        self
    }

    #[inline]
    pub fn border_left(mut self, width_px: i32, variant: Variant) -> Self {
        self.style.border_left_accent = Some((width_px, variant));
        self
    }

    #[inline]
    pub fn rounded(mut self, radius: i32) -> Self {
        self.style.radius = radius;
        self
    }

    #[inline]
    pub fn rounded_md(mut self) -> Self {
        self.style.radius = radii::RADIUS_MD;
        self
    }

    #[inline]
    pub fn rounded_card(mut self) -> Self {
        self.style.radius = radii::RADIUS_CARD;
        self
    }

    #[inline]
    pub fn rounded_pill(mut self) -> Self {
        self.style.radius = radii::RADIUS_PILL;
        self
    }

    // =========================================================================
    // Widget-Specific Modifiers (Typography, Variants, Events)
    // =========================================================================

    #[inline]
    pub fn bold(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { font, .. } | WidgetKind::Label { font, .. } => {
                *font = FontScale::Bold;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn title_font(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { font, .. } | WidgetKind::Label { font, .. } => {
                *font = FontScale::Title;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn hero_font(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { font, .. } | WidgetKind::Label { font, .. } => {
                *font = FontScale::Hero;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn small(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { font, .. } | WidgetKind::Label { font, .. } => {
                *font = FontScale::Small;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn secondary(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { tone, .. } | WidgetKind::Label { tone, .. } => {
                *tone = TextTone::Secondary;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn muted(self) -> Self {
        self.secondary()
    }

    #[inline]
    pub fn warning(self) -> Self {
        self.tone_variant(Variant::Warning)
    }

    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        match &mut self.kind {
            WidgetKind::Text { font, .. } | WidgetKind::Label { font, .. } => {
                *font = match s {
                    Size::Sm => FontScale::Small,
                    Size::Md => FontScale::Normal,
                    Size::Lg => FontScale::Title,
                };
            }
            WidgetKind::Button { size, .. } => {
                *size = s;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn enabled(self, is_enabled: bool) -> Self {
        self.disabled(!is_enabled)
    }

    #[inline]
    pub fn accent(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { tone, .. } | WidgetKind::Label { tone, .. } => {
                *tone = TextTone::Accent;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn tone(mut self, t: TextTone) -> Self {
        match &mut self.kind {
            WidgetKind::Text { tone, .. } | WidgetKind::Label { tone, .. } => {
                *tone = t;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn tone_variant(mut self, v: Variant) -> Self {
        match &mut self.kind {
            WidgetKind::Text { tone, .. } | WidgetKind::Label { tone, .. } => {
                *tone = TextTone::Variant(v);
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn text_center(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { align, .. } | WidgetKind::Label { align, .. } => {
                *align = TextAlign::Center;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn text_right(mut self) -> Self {
        match &mut self.kind {
            WidgetKind::Text { align, .. } | WidgetKind::Label { align, .. } => {
                *align = TextAlign::Right;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn nowrap(mut self) -> Self {
        if let WidgetKind::Text { wrap, .. } = &mut self.kind {
            *wrap = false;
        }
        self
    }

    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        match &mut self.kind {
            WidgetKind::Badge { variant, .. } | WidgetKind::Button { variant, .. } => {
                *variant = v;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn pill(mut self) -> Self {
        if let WidgetKind::Badge { is_pill, .. } = &mut self.kind {
            *is_pill = true;
        }
        self
    }

    #[inline]
    pub fn sm(mut self) -> Self {
        if let WidgetKind::Button { size, .. } = &mut self.kind {
            *size = Size::Sm;
        }
        self
    }

    #[inline]
    pub fn lg(mut self) -> Self {
        if let WidgetKind::Button { size, .. } = &mut self.kind {
            *size = Size::Lg;
        }
        self
    }

    #[inline]
    pub fn disabled(mut self, is_disabled: bool) -> Self {
        match &mut self.kind {
            WidgetKind::Button { disabled, .. }
            | WidgetKind::Badge { disabled, .. }
            | WidgetKind::Switch { disabled, .. }
            | WidgetKind::Checkbox { disabled, .. }
            | WidgetKind::Select { disabled, .. }
            | WidgetKind::Slider { disabled, .. }
            | WidgetKind::Input { disabled, .. } => {
                *disabled = is_disabled;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn password(mut self, is_pw: bool) -> Self {
        if let WidgetKind::Input { password, .. } = &mut self.kind {
            *password = is_pw;
        }
        self
    }

    #[inline]
    pub fn auto_focus(mut self, auto: bool) -> Self {
        if let WidgetKind::Input { auto_focus, .. } = &mut self.kind {
            *auto_focus = auto;
        }
        self
    }

    #[inline]
    pub fn key(mut self, k: u32) -> Self {
        match &mut self.kind {
            WidgetKind::Select { key, .. } | WidgetKind::Input { key, .. } => {
                *key = k;
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn indeterminate(mut self, is_indet: bool) -> Self {
        if let WidgetKind::Checkbox { indeterminate, .. } = &mut self.kind {
            *indeterminate = is_indet;
        }
        self
    }

    #[inline]
    pub fn note(mut self, note_str: impl Into<String>) -> Self {
        match &mut self.kind {
            WidgetKind::Switch { note, .. } | WidgetKind::Checkbox { note, .. } => {
                *note = note_str.into();
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }

    #[inline]
    pub fn on_toggle(mut self, msg: M) -> Self {
        match &mut self.kind {
            WidgetKind::Switch { on_toggle, .. } | WidgetKind::Checkbox { on_toggle, .. } => {
                *on_toggle = Some(msg);
            }
            _ => {}
        }
        self
    }

    #[inline]
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) -> M + Send + Sync + 'static,
    {
        if let WidgetKind::Select { on_select, .. } = &mut self.kind {
            *on_select = Some(Arc::new(f));
        }
        self
    }

    #[inline]
    pub fn on_slide<F>(mut self, f: F) -> Self
    where
        F: Fn(i32, bool) -> M + Send + Sync + 'static,
    {
        if let WidgetKind::Slider { on_change, .. } = &mut self.kind {
            *on_change = Some(Arc::new(f));
        }
        self
    }
}
