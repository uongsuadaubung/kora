//! Declarative Checkbox Component (`<input type="checkbox">`)

use super::element::{Element, WidgetKind};

/// Modern Checkbox (`15x15` rounded box with vector checkmark / tri-state bar).
#[inline]
pub fn checkbox<M>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> M,
) -> Element<M> {
    Element::new(WidgetKind::Checkbox {
        checked,
        indeterminate: false,
        label: label.into(),
        note: String::new(),
        disabled: false,
        on_toggle: Some(on_toggle(!checked)),
    })
}

