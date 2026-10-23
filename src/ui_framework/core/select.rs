//! Declarative Select Dropdown Component (`<select>`)

use super::element::{Element, WidgetKind};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

static NEXT_SELECT_KEY: AtomicU32 = AtomicU32::new(1);

/// Modern Select Dropdown (`key` is automatically assigned).
#[inline]
pub fn select<M, S: Into<String>>(
    options: impl IntoIterator<Item = S>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Send + Sync + 'static,
) -> Element<M> {
    let key = NEXT_SELECT_KEY.fetch_add(1, Ordering::Relaxed);
    Element::new(WidgetKind::Select {
        key,
        selected,
        options: options.into_iter().map(Into::into).collect(),
        disabled: false,
        on_select: Some(Arc::new(on_select)),
    })
}

