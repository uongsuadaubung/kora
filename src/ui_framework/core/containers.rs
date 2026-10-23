//! Declarative Layout & Container Primitives (`row`, `col`, `grid`, `card`, `alert`, `divider`, `spacer`, `empty`)

use super::element::{Element, Length, WidgetKind};
use super::style::Variant;
use super::text::text;

/// Invisible placeholder node (`0x0`, skipped by flex gaps).
#[inline]
pub fn empty<M>() -> Element<M> {
    Element::new(WidgetKind::Empty)
}

/// Flexible spacer that expands to fill remaining main-axis space (`flex: 1`).
#[inline]
pub fn spacer<M>() -> Element<M> {
    let mut el = Element::new(WidgetKind::Empty);
    el.style.width = Length::Flex(1);
    el.style.height = Length::Flex(1);
    el
}

/// 1px horizontal divider line.
#[inline]
pub fn divider<M>() -> Element<M> {
    Element::new(WidgetKind::Divider).w_full().h(1)
}

/// Horizontal flex container (`display: flex; flex-direction: row`).
#[inline]
pub fn row<M>(children: impl IntoIterator<Item = Element<M>>) -> Element<M> {
    Element::new(WidgetKind::Row(Vec::new())).children(children)
}

/// Vertical flex container (`display: flex; flex-direction: column`).
#[inline]
pub fn col<M>(children: impl IntoIterator<Item = Element<M>>) -> Element<M> {
    Element::new(WidgetKind::Col(Vec::new()))
        .w_full()
        .children(children)
}

/// Equal-width N-column grid container (`display: grid; grid-template-columns: repeat(cols, 1fr)`).
#[inline]
pub fn grid<M>(cols: usize, children: impl IntoIterator<Item = Element<M>>) -> Element<M> {
    Element::new(WidgetKind::Grid {
        cols: cols.max(1),
        children: Vec::new(),
    })
    .w_full()
    .gap(3)
    .children(children)
}

/// Standard Bootstrap Card container (`bg_card + border + rounded_card + padding`).
#[inline]
pub fn card<M>(children: impl IntoIterator<Item = Element<M>>) -> Element<M> {
    col(children).bg_card().border().rounded_card().p(3).gap(2)
}

/// Bootstrap Card with distinct header bar and body section (`.card-header` + `.card-body`).
pub fn card_grouped<M>(
    header: Element<M>,
    body_children: impl IntoIterator<Item = Element<M>>,
) -> Element<M> {
    col([
        row([header])
            .bg_header()
            .px(3)
            .py(2)
            .border_bottom()
            .items_center()
            .justify_between(),
        col(body_children),
    ])
    .bg_card()
    .border()
    .rounded_card()
}

/// Bootstrap Alert banner with a 4px colored left accent border.
pub fn alert<M>(variant: Variant, title: impl Into<String>, desc: impl Into<String>) -> Element<M> {
    let title_str = title.into();
    let desc_str = desc.into();
    col([
        if !title_str.is_empty() {
            text(title_str).bold().tone_variant(variant)
        } else {
            empty()
        },
        if !desc_str.is_empty() {
            text(desc_str).secondary()
        } else {
            empty()
        },
    ])
    .bg_subtle(variant)
    .border_variant(variant)
    .border_left(4, variant)
    .rounded_md()
    .px(3)
    .py(2)
    .gap(1)
}

