//! Declarative Form / UI Label Component (`<label>`)

use super::element::{Element, FontScale, TextAlign, TextTone, WidgetKind};

/// Form / UI Label element (`<label class="form-label">`).
///
/// Single-line intrinsic text label, ideal for field labels, section titles,
/// and clickable links (`label("...").on_click(msg)`).
#[inline]
pub fn label<M>(content: impl Into<String>) -> Element<M> {
    Element::new(WidgetKind::Label {
        content: content.into(),
        font: FontScale::Normal,
        tone: TextTone::Primary,
        align: TextAlign::Left,
    })
}

