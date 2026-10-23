//! Declarative Push Button Component (`<button>`)

use super::element::{Element, Size, WidgetKind};
use super::style::Variant;

/// Bootstrap Push Button (`<button class="btn">`).
#[inline]
pub fn button<M>(label: impl Into<String>, variant: Variant, on_click: M) -> Element<M> {
    Element::new(WidgetKind::Button {
        label: label.into(),
        variant,
        size: Size::Md,
        disabled: false,
        on_click: Some(on_click),
    })
}

