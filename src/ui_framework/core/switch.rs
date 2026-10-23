//! Declarative Modern Fluent Toggle Switch Component

use super::element::{Element, IntoElement, Styleable, WidgetKind};
use super::style::Style;

/// Modern Fluent Toggle Switch builder with type-safe modifiers.
pub struct Switch<M> {
    pub label: String,
    pub checked: bool,
    pub note: String,
    pub disabled: bool,
    pub on_toggle: Option<M>,
    pub style: Style,
}

/// Modern Fluent Toggle Switch (`36x20` pill track with sliding thumb).
#[inline]
pub fn switch<M>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> M,
) -> Switch<M> {
    Switch {
        label: label.into(),
        checked,
        note: String::new(),
        disabled: false,
        on_toggle: Some(on_toggle(!checked)),
        style: Style::default(),
    }
}

impl<M> Switch<M> {
    #[inline]
    pub fn note(mut self, note_str: impl Into<String>) -> Self {
        self.note = note_str.into();
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
    pub fn on_toggle(mut self, msg: M) -> Self {
        self.on_toggle = Some(msg);
        self
    }
}

impl<M> Styleable for Switch<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Switch<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Switch {
            checked: self.checked,
            label: self.label,
            note: self.note,
            disabled: self.disabled,
            on_toggle: self.on_toggle,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<Switch<M>> for Element<M> {
    #[inline]
    fn from(s: Switch<M>) -> Self {
        s.into_element()
    }
}
