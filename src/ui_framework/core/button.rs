//! Declarative Push Button Component (`<button>`)

use super::element::{Element, IntoElement, Size, Styleable, WidgetKind};
use super::style::{Style, Variant};

/// Push Button builder with type-safe modifiers.
pub struct Button<M> {
    pub label: String,
    pub variant: Variant,
    pub size: Size,
    pub disabled: bool,
    pub on_click: Option<M>,
    pub style: Style,
}

/// Bootstrap Push Button (`<button class="btn">`).
#[inline]
pub fn button<M>(label: impl Into<String>, variant: Variant, on_click: M) -> Button<M> {
    Button {
        label: label.into(),
        variant,
        size: Size::Md,
        disabled: false,
        on_click: Some(on_click),
        style: Style::default(),
    }
}

impl<M> Button<M> {
    #[inline]
    pub fn sm(mut self) -> Self {
        self.size = Size::Sm;
        self
    }

    #[inline]
    pub fn lg(mut self) -> Self {
        self.size = Size::Lg;
        self
    }

    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
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
}

impl<M> Styleable for Button<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Button<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Button {
            label: self.label,
            variant: self.variant,
            size: self.size,
            disabled: self.disabled,
            on_click: self.on_click,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<Button<M>> for Element<M> {
    #[inline]
    fn from(b: Button<M>) -> Self {
        b.into_element()
    }
}

