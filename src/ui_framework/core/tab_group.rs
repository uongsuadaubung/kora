//! Declarative TabGroup / Segmented Control Component
//!
//! Creates a horizontal pill-segmented tab group matching Windows Fluent Design.

use super::element::{Element, IntoElement, Length, Styleable, WidgetKind};
use super::style::Style;
use std::sync::Arc;

/// Segmented Pill TabGroup builder with type-safe modifiers.
pub struct TabGroup<M> {
    pub options: Vec<String>,
    pub selected: usize,
    pub on_select: Option<Arc<dyn Fn(usize) -> M + Send + Sync>>,
    pub style: Style,
}

/// Creates a modern Segmented Pill TabGroup element.
pub fn tab_group<M: Send + Sync + 'static>(
    options: Vec<impl Into<String>>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Send + Sync + 'static,
) -> TabGroup<M> {
    let opts: Vec<String> = options.into_iter().map(|s| s.into()).collect();
    let mut tg = TabGroup {
        options: opts,
        selected,
        on_select: Some(Arc::new(on_select)),
        style: Style::default(),
    };
    tg.style.height = Length::Fixed(32);
    tg.style.width = Length::Full;
    tg
}

/// Convenience alias for [`tab_group`].
#[inline]
pub fn tabs<M: Send + Sync + 'static>(
    options: Vec<impl Into<String>>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Send + Sync + 'static,
) -> TabGroup<M> {
    tab_group(options, selected, on_select)
}

impl<M> TabGroup<M> {
    #[inline]
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) -> M + Send + Sync + 'static,
    {
        self.on_select = Some(Arc::new(f));
        self
    }
}

impl<M> Styleable for TabGroup<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for TabGroup<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::TabGroup {
            options: self.options,
            selected: self.selected,
            on_select: self.on_select,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<TabGroup<M>> for Element<M> {
    #[inline]
    fn from(tg: TabGroup<M>) -> Self {
        tg.into_element()
    }
}
