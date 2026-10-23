//! Declarative TabGroup / Segmented Control Component
//!
//! Creates a horizontal pill-segmented tab group matching Windows Fluent Design.

use super::element::{Element, WidgetKind};
use std::sync::Arc;

/// Creates a modern Segmented Pill TabGroup element.
pub fn tab_group<M: Send + Sync + 'static>(
    options: Vec<impl Into<String>>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Send + Sync + 'static,
) -> Element<M> {
    let opts: Vec<String> = options.into_iter().map(|s| s.into()).collect();
    let el = Element::new(WidgetKind::TabGroup {
        options: opts,
        selected,
        on_select: Some(Arc::new(on_select)),
    });
    el.h(32).w_full()
}

/// Convenience alias for [`tab_group`].
#[inline]
pub fn tabs<M: Send + Sync + 'static>(
    options: Vec<impl Into<String>>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Send + Sync + 'static,
) -> Element<M> {
    tab_group(options, selected, on_select)
}

