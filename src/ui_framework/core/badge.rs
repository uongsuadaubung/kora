//! Declarative Badge & Pill Component (`<span class="badge">`)

use super::element::{Element, IntoElement, Styleable, WidgetKind};
use super::style::{Style, Variant};

/// Badge and pill component builder with type-safe modifiers.
pub struct Badge<M> {
    pub text: String,
    pub variant: Variant,
    pub is_pill: bool,
    pub disabled: bool,
    pub on_click: Option<M>,
    pub style: Style,
}

/// Bootstrap Badge / Pill (`<span class="badge">`).
#[inline]
pub fn badge<M>(label: impl Into<String>, variant: Variant) -> Badge<M> {
    Badge {
        text: label.into(),
        variant,
        is_pill: false,
        disabled: false,
        on_click: None,
        style: Style::default(),
    }
}

impl<M> Badge<M> {
    #[inline]
    pub fn pill(mut self) -> Self {
        self.is_pill = true;
        self
    }

    #[inline]
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[inline]
    pub fn disabled(mut self, is_disabled: bool) -> Self {
        self.disabled = is_disabled;
        self
    }

    #[inline]
    pub fn enabled(self, is_enabled: bool) -> Self {
        self.disabled(!is_enabled)
    }

    #[inline]
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M> Styleable for Badge<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Badge<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Badge {
            text: self.text,
            variant: self.variant,
            is_pill: self.is_pill,
            disabled: self.disabled,
            on_click: self.on_click,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<Badge<M>> for Element<M> {
    #[inline]
    fn from(b: Badge<M>) -> Self {
        b.into_element()
    }
}

