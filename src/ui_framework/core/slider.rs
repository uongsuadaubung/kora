//! Declarative Slider Component (`<input type="range">`)

use super::element::{Element, IntoElement, Length, Styleable, WidgetKind};
use super::style::Style;
use std::sync::Arc;

/// Smooth Horizontal Slider builder with type-safe modifiers.
pub struct Slider<M> {
    pub value: i32,
    pub min: i32,
    pub max: i32,
    pub disabled: bool,
    pub on_change: Option<Arc<dyn Fn(i32, bool) -> M + Send + Sync>>,
    pub style: Style,
}

/// Smooth Horizontal Slider (`min..=max`).
#[inline]
pub fn slider<M>(
    min: i32,
    max: i32,
    value: i32,
    on_change: impl Fn(i32, bool) -> M + Send + Sync + 'static,
) -> Slider<M> {
    let mut s = Slider {
        value,
        min,
        max: max.max(min + 1),
        disabled: false,
        on_change: Some(Arc::new(on_change)),
        style: Style::default(),
    };
    s.style.width = Length::Full;
    s.style.height = Length::Fixed(24);
    s
}

impl<M> Slider<M> {
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
    pub fn on_slide<F>(mut self, f: F) -> Self
    where
        F: Fn(i32, bool) -> M + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(f));
        self
    }
}

impl<M> Styleable for Slider<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Slider<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Slider {
            value: self.value,
            min: self.min,
            max: self.max,
            disabled: self.disabled,
            on_change: self.on_change,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<Slider<M>> for Element<M> {
    #[inline]
    fn from(s: Slider<M>) -> Self {
        s.into_element()
    }
}
