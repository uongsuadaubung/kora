//! Modal Confirmation Dialog for Kora UI
//!
//! Uses [`crate::ui_framework::core::modal::{open_modal, modal_dialog, modal_footer}`]
//! to render a clean, tight, 3-section confirmation dialog without empty footer dead space.

use crate::ui_framework::core::*;
use std::sync::{Arc, Mutex};

/// Configuration options for confirmation dialogs.
#[derive(Debug, Clone)]
pub struct ConfirmDialogOptions {
    pub title: String,
    pub message: String,
    pub yes_text: String,
    pub no_text: String,
    pub is_danger: bool,
}

impl ConfirmDialogOptions {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let strings = crate::language::current();
        Self {
            title: title.into(),
            message: message.into(),
            yes_text: strings.btn_yes.to_string(),
            no_text: strings.btn_no.to_string(),
            is_danger: false,
        }
    }

    pub fn danger(mut self, is_danger: bool) -> Self {
        self.is_danger = is_danger;
        self
    }

    pub fn yes_text(mut self, text: impl Into<String>) -> Self {
        self.yes_text = text.into();
        self
    }

    pub fn no_text(mut self, text: impl Into<String>) -> Self {
        self.no_text = text.into();
        self
    }
}

pub fn render_confirm_view(options: &ConfirmDialogOptions, height: i32) -> Element<bool> {
    let is_danger = options.is_danger;

    let el = modal_dialog()
        .body(
            col(elements![text(&options.message),])
                .h_flex(1)
                .justify_center(),
        )
        .footer(modal_footer(elements![
            button(&options.no_text, Variant::Secondary, false).w(84),
            button(
                &options.yes_text,
                if is_danger {
                    Variant::Danger
                } else {
                    Variant::Primary
                },
                true,
            )
            .w(88),
        ]))
        .build();

    if height > 0 { el.h(height) } else { el }
}

/// Displays a modern, web-style modal confirmation dialog with Yes / No actions.
pub fn show_confirm_dialog<F>(parent: isize, options: ConfirmDialogOptions, on_confirm: F)
where
    F: FnOnce() + Send + 'static,
{
    let cb = Arc::new(Mutex::new(Some(on_confirm)));
    let width = 390;
    let height = 115;
    let config = ModalConfig::new(&options.title, width, height);

    let view = render_confirm_view(&options, height);

    open_modal(parent, config, view, move |modal, is_yes| {
        if is_yes {
            if let Some(f) = cb.lock().unwrap().take() {
                f();
            }
        }
        modal.close();
    });
}

/// Displays a danger-styled confirmation dialog (`btn_yes` and `btn_no` automatically localized).
pub fn show_danger_confirm<F>(
    parent: isize,
    title: impl Into<String>,
    message: impl Into<String>,
    on_confirm: F,
) where
    F: FnOnce() + Send + 'static,
{
    show_confirm_dialog(
        parent,
        ConfirmDialogOptions::new(title, message).danger(true),
        on_confirm,
    );
}

/// Displays a standard danger-styled delete confirmation dialog.
pub fn show_delete_confirm<F>(parent: isize, message: impl Into<String>, on_confirm: F)
where
    F: FnOnce() + Send + 'static,
{
    let strings = crate::language::current();
    show_danger_confirm(parent, strings.confirm_delete_title, message, on_confirm);
}
