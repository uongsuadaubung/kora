//! Declarative Form / UI Label Component (`<label>`)

use super::element::{
    Element, FontScale, IntoElement, Size, Styleable, TextAlign, TextTone, WidgetKind,
};
use super::style::{Style, Variant};

/// Form / UI Label element builder with typography modifiers.
pub struct Label<M> {
    pub content: String,
    pub font: FontScale,
    pub tone: TextTone,
    pub align: TextAlign,
    pub on_click: Option<M>,
    pub style: Style,
}

/// Form / UI Label element (`<label class="form-label">`).
///
/// Single-line intrinsic text label, ideal for field labels, section titles,
/// and clickable links (`label("...").on_click(msg)`).
#[inline]
pub fn label<M>(content: impl Into<String>) -> Label<M> {
    Label {
        content: content.into(),
        font: FontScale::Normal,
        tone: TextTone::Primary,
        align: TextAlign::Left,
        on_click: None,
        style: Style::default(),
    }
}

impl<M> Label<M> {
    #[inline]
    pub fn bold(mut self) -> Self {
        self.font = FontScale::Bold;
        self
    }

    #[inline]
    pub fn title_font(mut self) -> Self {
        self.font = FontScale::Title;
        self
    }

    #[inline]
    pub fn hero_font(mut self) -> Self {
        self.font = FontScale::Hero;
        self
    }

    #[inline]
    pub fn small(mut self) -> Self {
        self.font = FontScale::Small;
        self
    }

    #[inline]
    pub fn size(mut self, s: Size) -> Self {
        self.font = match s {
            Size::Sm => FontScale::Small,
            Size::Md => FontScale::Normal,
            Size::Lg => FontScale::Title,
        };
        self
    }

    #[inline]
    pub fn secondary(mut self) -> Self {
        self.tone = TextTone::Secondary;
        self
    }

    #[inline]
    pub fn muted(self) -> Self {
        self.secondary()
    }

    #[inline]
    pub fn accent(mut self) -> Self {
        self.tone = TextTone::Accent;
        self
    }

    #[inline]
    pub fn warning(self) -> Self {
        self.tone_variant(Variant::Warning)
    }

    #[inline]
    pub fn tone(mut self, t: TextTone) -> Self {
        self.tone = t;
        self
    }

    #[inline]
    pub fn tone_variant(mut self, v: Variant) -> Self {
        self.tone = TextTone::Variant(v);
        self
    }

    #[inline]
    pub fn text_center(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }

    #[inline]
    pub fn text_right(mut self) -> Self {
        self.align = TextAlign::Right;
        self
    }

    #[inline]
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M> Styleable for Label<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Label<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Label {
            content: self.content,
            font: self.font,
            tone: self.tone,
            align: self.align,
        });
        el.style = self.style;
        el.on_click = self.on_click;
        el
    }
}

impl<M> From<Label<M>> for Element<M> {
    #[inline]
    fn from(l: Label<M>) -> Self {
        l.into_element()
    }
}
