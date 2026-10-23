//! Declarative Text Input Component (`<input type="text">`)
//!
//! Renders a native-backed text input field with Fluent Design rounded styling,
//! caret blinking, full IME / Vietnamese typing support, and focus rings.

use super::element::{Element, WidgetKind};
use std::sync::Arc;

/// Creates a declarative single-line text input field.
#[inline]
pub fn input<M>(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_change: impl Fn(String) -> M + Send + Sync + 'static,
) -> Element<M> {
    Element::new(WidgetKind::Input {
        key: 0,
        value: value.into(),
        placeholder: placeholder.into(),
        disabled: false,
        password: false,
        auto_focus: false,
        on_change: Some(Arc::new(on_change)),
    })
}

/// Creates a declarative password input field with masked characters.
#[inline]
pub fn password_input<M>(
    value: impl Into<String>,
    placeholder: impl Into<String>,
    on_change: impl Fn(String) -> M + Send + Sync + 'static,
) -> Element<M> {
    Element::new(WidgetKind::Input {
        key: 0,
        value: value.into(),
        placeholder: placeholder.into(),
        disabled: false,
        password: true,
        auto_focus: false,
        on_change: Some(Arc::new(on_change)),
    })
}

