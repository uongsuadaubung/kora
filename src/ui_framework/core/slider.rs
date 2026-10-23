//! Declarative Slider Component (`<input type="range">`)

use super::element::{Element, WidgetKind};
use std::sync::Arc;

/// Smooth Horizontal Slider (`min..=max`).
#[inline]
pub fn slider<M>(
    min: i32,
    max: i32,
    value: i32,
    on_change: impl Fn(i32, bool) -> M + Send + Sync + 'static,
) -> Element<M> {
    Element::new(WidgetKind::Slider {
        value,
        min,
        max: max.max(min + 1),
        disabled: false,
        on_change: Some(Arc::new(on_change)),
    })
    .w_full()
    .h(24)
}

