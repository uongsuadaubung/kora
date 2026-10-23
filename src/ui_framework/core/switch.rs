//! Declarative Modern Fluent Toggle Switch Component

use super::element::{Element, WidgetKind};

/// Modern Fluent Toggle Switch (`36x20` pill track with sliding thumb).
#[inline]
pub fn switch<M>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> M,
) -> Element<M> {
    Element::new(WidgetKind::Switch {
        checked,
        label: label.into(),
        note: String::new(),
        disabled: false,
        on_toggle: Some(on_toggle(!checked)),
    })
}

