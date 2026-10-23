//! Declarative Typography Text Component (`<span>`, `<p>`)

use super::element::{Element, FontScale, TextAlign, TextTone, WidgetKind};

/// Typography text node (supports automatic multiline wrapping via `DT_WORDBREAK`).
#[inline]
pub fn text<M>(content: impl Into<String>) -> Element<M> {
    Element::new(WidgetKind::Text {
        content: content.into(),
        font: FontScale::Normal,
        tone: TextTone::Primary,
        align: TextAlign::Left,
        wrap: true,
        on_click: None,
    })
}

