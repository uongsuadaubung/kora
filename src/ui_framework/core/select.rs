//! Declarative Select Dropdown Component (`<select>`)

use super::element::{Element, IntoElement, Styleable, WidgetKind};
use super::style::Style;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT_SELECT_KEY: AtomicU32 = AtomicU32::new(1);

/// Modern Select Dropdown builder with type-safe modifiers.
pub struct Select<M> {
    pub key: u32,
    pub selected: usize,
    pub options: Vec<String>,
    pub disabled: bool,
    pub on_select: Option<Arc<dyn Fn(usize) -> M + Send + Sync>>,
    pub style: Style,
}

/// Modern Select Dropdown (`key` is automatically assigned).
#[inline]
pub fn select<M, S: Into<String>>(
    options: impl IntoIterator<Item = S>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Send + Sync + 'static,
) -> Select<M> {
    let key = NEXT_SELECT_KEY.fetch_add(1, Ordering::Relaxed);
    Select {
        key,
        selected,
        options: options.into_iter().map(Into::into).collect(),
        disabled: false,
        on_select: Some(Arc::new(on_select)),
        style: Style::default(),
    }
}

impl<M> Select<M> {
    #[inline]
    pub fn key(mut self, k: u32) -> Self {
        self.key = k;
        self
    }

    #[inline]
    pub fn disabled(mut self, is_disabled: bool) -> Self {
        self.disabled = is_disabled;
        self
    }

    #[inline]
    pub fn enabled(self, is_enabled: bool) -> Self {
        self.disabled(!is_enabled)
    }

    #[inline]
    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(usize) -> M + Send + Sync + 'static,
    {
        self.on_select = Some(Arc::new(f));
        self
    }
}

impl<M> Styleable for Select<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M> IntoElement<M> for Select<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        let mut el = Element::new(WidgetKind::Select {
            key: self.key,
            selected: self.selected,
            options: self.options,
            disabled: self.disabled,
            on_select: self.on_select,
        });
        el.style = self.style;
        el
    }
}

impl<M> From<Select<M>> for Element<M> {
    #[inline]
    fn from(s: Select<M>) -> Self {
        s.into_element()
    }
}
