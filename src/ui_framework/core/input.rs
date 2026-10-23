//! Declarative Text Input Component (`<input type="text">`)
//!
//! Renders a native-backed text input field with Fluent Design rounded styling,
//! caret blinking, full IME / Vietnamese typing support, and focus rings.

use super::element::{Element, IntoElement, Styleable, WidgetKind};
use super::style::Style;
use std::sync::Arc;

/// Text Input builder with type-safe modifiers.
pub struct Input<M> {
    pub key: u32,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub password: bool,
    pub auto_focus: bool,
    pub on_change: Option<Arc<dyn Fn(String) -> M + Send + Sync>>,
    pub style: Style,
}

/// Creates a declarative single-line text input field.
#[inline]
pub fn input<M>(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_change: impl Fn(String) -> M + Send + Sync + 'static,
) -> Input<M> {
    Input {
        key: 0,
        value: value.into(),
        placeholder: placeholder.into(),
        disabled: false,
        password: false,
        auto_focus: false,
        on_change: Some(Arc::new(on_change)),
        style: Style::default(),
    }
}

/// Creates a declarative password input field with masked characters.
#[inline]
pub fn password_input<M>(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_change: impl Fn(String) -> M + Send + Sync + 'static,
) -> Input<M> {
    input(value, placeholder, on_change).password(true)
}

impl<M> Input<M> {
    #[inline]
    pub fn password(mut self, is_pw: bool) -> Self {
        self.password = is_pw;
        self
    }

    #[inline]
    pub fn auto_focus(mut self, auto: bool) -> Self {
        self.auto_focus = auto;
        self
    }

    #[inline]
    pub fn key(mut self, k: u32) -> Self {
        self.key = k;
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

impl<M> Styleable for Input<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Input<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Input {
            key: self.key,
            value: self.value,
            placeholder: self.placeholder,
            disabled: self.disabled,
            password: self.password,
            auto_focus: self.auto_focus,
            on_change: self.on_change,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<Input<M>> for Element<M> {
    #[inline]
    fn from(i: Input<M>) -> Self {
        i.into_element()
    }
}

