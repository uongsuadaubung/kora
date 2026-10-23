//! React-Style Declarative Virtual Element Tree & Bootstrap Atomic Components
//!
//! All UI views are pure functions of state returning [`Element<M>`].
//! Components compose arbitrarily via `.child(...)` and `.children(...)` and are styled
//! via chainable Bootstrap utility modifiers (`.p(3)`, `.gap(2)`, `.justify_between()`, `.items_center()`, etc.).

pub use super::style::*;
use crate::ui_framework::tokens::radii;
use std::sync::Arc;

/// Conversion trait into a virtual DOM [`Element<M>`].
pub trait IntoElement<M> {
    fn into_element(self) -> Element<M>;
}

impl<M> IntoElement<M> for Element<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        self
    }
}

/// Convenience macro to construct a fixed-size stack array of heterogeneous elements
/// converted to [`Element<M>`] via [`IntoElement`].
#[macro_export]
macro_rules! elements {
    ($($elem:expr),* $(,)?) => {
        [ $( $crate::ui_framework::core::IntoElement::into_element($elem) ),* ]
    };
}

pub use elements;

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

    pub fn child(mut self, child: impl IntoElement<M>) -> Self {
        let child = child.into_element();
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

    pub fn children<I, E>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: IntoElement<M>,
    {
        for c in iter {
            let child = c.into_element();
            if !child.is_empty() {
                match &mut self.kind {
                    WidgetKind::Row(list) | WidgetKind::Col(list) => list.push(child),
                    WidgetKind::Grid { children, .. } => children.push(child),
                    _ => {}
                }
            }
        }
        self
    }

    #[inline]
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }

    // =========================================================================
    // Inherent Bootstrap Styling & Spacing Utilities
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

    #[inline]
    pub fn col_span(self, span: u16) -> Self {
        self.flex(span)
    }

    #[inline]
    pub fn col_12(self) -> Self {
        self.flex(12)
    }

    #[inline]
    pub fn col_6(self) -> Self {
        self.flex(6)
    }

    #[inline]
    pub fn col_4(self) -> Self {
        self.flex(4)
    }

    #[inline]
    pub fn col_3(self) -> Self {
        self.flex(3)
    }

    #[inline]
    pub fn col_8(self) -> Self {
        self.flex(8)
    }

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
}

/// Trait providing fluent styling and layout utility methods for any UI component.
pub trait Styleable: Sized {
    fn style_mut(&mut self) -> &mut Style;

    // Spacing
    #[inline]
    fn p(mut self, scale: i32) -> Self {
        self.style_mut().padding = Edges::all(spacing_scale(scale));
        self
    }
    #[inline]
    fn px(mut self, scale: i32) -> Self {
        let px = spacing_scale(scale);
        self.style_mut().padding.left = px;
        self.style_mut().padding.right = px;
        self
    }
    #[inline]
    fn py(mut self, scale: i32) -> Self {
        let py = spacing_scale(scale);
        self.style_mut().padding.top = py;
        self.style_mut().padding.bottom = py;
        self
    }
    #[inline]
    fn pt(mut self, scale: i32) -> Self {
        self.style_mut().padding.top = spacing_scale(scale);
        self
    }
    #[inline]
    fn pb(mut self, scale: i32) -> Self {
        self.style_mut().padding.bottom = spacing_scale(scale);
        self
    }
    #[inline]
    fn pl(mut self, scale: i32) -> Self {
        self.style_mut().padding.left = spacing_scale(scale);
        self
    }
    #[inline]
    fn pr(mut self, scale: i32) -> Self {
        self.style_mut().padding.right = spacing_scale(scale);
        self
    }
    #[inline]
    fn padding(mut self, edges: Edges) -> Self {
        self.style_mut().padding = edges;
        self
    }
    #[inline]
    fn pad_xy(mut self, x_px: i32, y_px: i32) -> Self {
        self.style_mut().padding = Edges::xy(x_px, y_px);
        self
    }
    #[inline]
    fn gap(mut self, scale: i32) -> Self {
        self.style_mut().gap = spacing_scale(scale);
        self
    }

    // Flex & Sizing
    #[inline]
    fn w(mut self, px: i32) -> Self {
        self.style_mut().width = Length::Fixed(px);
        self
    }
    #[inline]
    fn h(mut self, px: i32) -> Self {
        self.style_mut().height = Length::Fixed(px);
        self
    }
    #[inline]
    fn min_w(mut self, px: i32) -> Self {
        self.style_mut().min_w = px;
        self
    }
    #[inline]
    fn min_h(mut self, px: i32) -> Self {
        self.style_mut().min_h = px;
        self
    }
    #[inline]
    fn w_full(mut self) -> Self {
        self.style_mut().width = Length::Full;
        self
    }
    #[inline]
    fn flex(mut self, weight: u16) -> Self {
        self.style_mut().width = Length::Flex(weight.max(1));
        self
    }
    #[inline]
    fn flex_1(self) -> Self {
        self.flex(1)
    }
    #[inline]
    fn h_flex(mut self, weight: u16) -> Self {
        self.style_mut().height = Length::Flex(weight.max(1));
        self
    }

    // Grid Column Modifiers
    #[inline]
    fn col_span(self, span: u16) -> Self {
        self.flex(span)
    }
    #[inline]
    fn col_12(self) -> Self {
        self.flex(12)
    }
    #[inline]
    fn col_6(self) -> Self {
        self.flex(6)
    }
    #[inline]
    fn col_4(self) -> Self {
        self.flex(4)
    }
    #[inline]
    fn col_3(self) -> Self {
        self.flex(3)
    }
    #[inline]
    fn col_8(self) -> Self {
        self.flex(8)
    }
    #[inline]
    fn col_9(self) -> Self {
        self.flex(9)
    }

    // Alignment
    #[inline]
    fn items_start(mut self) -> Self {
        self.style_mut().align = Align::Start;
        self
    }
    #[inline]
    fn items_center(mut self) -> Self {
        self.style_mut().align = Align::Center;
        self
    }
    #[inline]
    fn items_end(mut self) -> Self {
        self.style_mut().align = Align::End;
        self
    }
    #[inline]
    fn items_stretch(mut self) -> Self {
        self.style_mut().align = Align::Stretch;
        self
    }
    #[inline]
    fn justify_start(mut self) -> Self {
        self.style_mut().justify = Justify::Start;
        self
    }
    #[inline]
    fn justify_center(mut self) -> Self {
        self.style_mut().justify = Justify::Center;
        self
    }
    #[inline]
    fn justify_end(mut self) -> Self {
        self.style_mut().justify = Justify::End;
        self
    }
    #[inline]
    fn justify_between(mut self) -> Self {
        self.style_mut().justify = Justify::Between;
        self
    }

    // Backgrounds
    #[inline]
    fn bg(mut self, bg: BgStyle) -> Self {
        self.style_mut().bg = bg;
        self
    }
    #[inline]
    fn bg_card(mut self) -> Self {
        self.style_mut().bg = BgStyle::Card;
        self
    }
    #[inline]
    fn bg_header(mut self) -> Self {
        self.style_mut().bg = BgStyle::CardHeader;
        self
    }
    #[inline]
    fn bg_input(mut self) -> Self {
        self.style_mut().bg = BgStyle::Input;
        self
    }
    #[inline]
    fn bg_window(mut self) -> Self {
        self.style_mut().bg = BgStyle::Window;
        self
    }
    #[inline]
    fn bg_sidebar(mut self) -> Self {
        self.style_mut().bg = BgStyle::Sidebar;
        self
    }
    #[inline]
    fn bg_tab_active(mut self) -> Self {
        self.style_mut().bg = BgStyle::TabActive;
        self
    }
    #[inline]
    fn bg_subtle(mut self, variant: Variant) -> Self {
        self.style_mut().bg = BgStyle::Subtle(variant);
        self
    }

    // Borders
    #[inline]
    fn border_style(mut self, border: BorderStyle) -> Self {
        self.style_mut().border = border;
        self
    }
    #[inline]
    fn border(mut self) -> Self {
        self.style_mut().border = BorderStyle::Normal;
        self
    }
    #[inline]
    fn border_subtle(mut self) -> Self {
        self.style_mut().border = BorderStyle::Subtle;
        self
    }
    #[inline]
    fn border_accent(mut self) -> Self {
        self.style_mut().border = BorderStyle::Accent;
        self
    }
    #[inline]
    fn border_dashed(mut self) -> Self {
        self.style_mut().border = BorderStyle::Dashed;
        self
    }
    #[inline]
    fn border_variant(mut self, variant: Variant) -> Self {
        self.style_mut().border = BorderStyle::Variant(variant);
        self
    }
    #[inline]
    fn border_bottom(mut self) -> Self {
        self.style_mut().border_bottom = true;
        self
    }
    #[inline]
    fn border_top(mut self) -> Self {
        self.style_mut().border_top = true;
        self
    }
    #[inline]
    fn border_left(mut self, width_px: i32, variant: Variant) -> Self {
        self.style_mut().border_left_accent = Some((width_px, variant));
        self
    }

    // Border Radius
    #[inline]
    fn rounded(mut self, radius: i32) -> Self {
        self.style_mut().radius = radius;
        self
    }
    #[inline]
    fn rounded_md(mut self) -> Self {
        self.style_mut().radius = radii::RADIUS_MD;
        self
    }
    #[inline]
    fn rounded_card(mut self) -> Self {
        self.style_mut().radius = radii::RADIUS_CARD;
        self
    }
    #[inline]
    fn rounded_pill(mut self) -> Self {
        self.style_mut().radius = radii::RADIUS_PILL;
        self
    }
}

impl<M> Styleable for Element<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
