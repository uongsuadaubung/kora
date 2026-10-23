//! Declarative Badge & Pill Component (`<span class="badge">`)

use super::element::{Element, WidgetKind};
use super::style::Variant;

/// Bootstrap Badge / Pill (`<span class="badge">`).
#[inline]
pub fn badge<M>(label: impl Into<String>, variant: Variant) -> Element<M> {
    Element::new(WidgetKind::Badge {
        text: label.into(),
        variant,
        is_pill: false,
        disabled: false,
        on_click: None,
    })
}

