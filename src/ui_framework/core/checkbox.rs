//! Declarative Checkbox Component (`<input type="checkbox">`)

use super::element::{Element, IntoElement, Styleable, WidgetKind};
use super::style::Style;

/// Modern Checkbox builder with type-safe modifiers.
pub struct Checkbox<M> {
    pub label: String,
    pub checked: bool,
    pub indeterminate: bool,
    pub note: String,
    pub disabled: bool,
    pub on_toggle: Option<M>,
    pub style: Style,
}

/// Modern Checkbox (`15x15` rounded box with vector checkmark / tri-state bar).
#[inline]
pub fn checkbox<M>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> M,
) -> Checkbox<M> {
    Checkbox {
        label: label.into(),
        checked,
        indeterminate: false,
        note: String::new(),
        disabled: false,
        on_toggle: Some(on_toggle(!checked)),
        style: Style::default(),
    }
}

impl<M> Checkbox<M> {
    #[inline]
    pub fn indeterminate(mut self, is_indet: bool) -> Self {
        self.indeterminate = is_indet;
        self
    }

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

impl<M> Styleable for Checkbox<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Checkbox<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Checkbox {
            checked: self.checked,
            indeterminate: self.indeterminate,
            label: self.label,
            note: self.note,
            disabled: self.disabled,
            on_toggle: self.on_toggle,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<Checkbox<M>> for Element<M> {
    #[inline]
    fn from(c: Checkbox<M>) -> Self {
        c.into_element()
    }
}

