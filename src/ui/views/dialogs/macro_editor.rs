//! Declarative Modal Dialog for Adding / Editing Macro Shorthands (Gõ tắt)
//!
//! Uses [`crate::ui_framework::core::modal::{open_modal_with_wndproc, modal_dialog, modal_footer}`]
//! and declarative widgets [`input`, `select`, `button`, `text`] to render a clean, modern
//! 3-section modal dialog without manual Win32 child HWND coordinates or GDI subclassing.

use crate::engine::macro_table::MacroType;
use crate::ui_framework::core::*;
use crate::ui_framework::win32::{WM_KEYDOWN, WM_SYSKEYDOWN};
use std::sync::{Arc, Mutex};

/// Data transferred into or out of the Macro Editor dialog.
#[derive(Debug, Clone)]
pub struct MacroEditData {
    pub original_key: String,
    pub key: String,
    pub value: String,
    pub macro_type: MacroType,
}

#[derive(Clone, Debug)]
pub struct MacroEditorState {
    pub original_key: Option<String>,
    pub key: String,
    pub val: String,
    pub macro_type_idx: usize, // 0: Normal, 1: StartConsonant, 2: EndConsonant
}

#[derive(Clone, Debug, PartialEq)]
pub enum MacroEditorMsg {
    KeyChanged(String),
    ValChanged(String),
    TypeChanged(usize),
    Submit,
    Cancel,
}

pub fn render_macro_editor_view(state: &MacroEditorState) -> Element<MacroEditorMsg> {
    let strings = crate::language::current();
    let can_submit = !state.key.trim().is_empty() && !state.val.trim().is_empty();

    modal_dialog()
        .body(
            col(elements![
                row(elements![
                    col(elements![
                        text(strings.dlg_macro_lbl_key).bold().secondary().small(),
                        input(
                            &state.key,
                            strings.edit_macro_key_placeholder,
                            MacroEditorMsg::KeyChanged,
                        )
                        .key(101)
                        .auto_focus(state.original_key.is_none()),
                    ])
                    .flex_1()
                    .gap(1),
                    col(elements![
                        text(strings.dlg_macro_lbl_type).bold().secondary().small(),
                        select(
                            [
                                strings.macro_type_normal,
                                strings.macro_type_start,
                                strings.macro_type_end,
                            ],
                            state.macro_type_idx,
                            MacroEditorMsg::TypeChanged,
                        )
                        .key(102)
                        .h(32),
                    ])
                    .flex_1()
                    .gap(1),
                ])
                .gap(3),
                col(elements![
                    text(strings.dlg_macro_lbl_val).bold().secondary().small(),
                    input(
                        &state.val,
                        strings.edit_macro_val_placeholder,
                        MacroEditorMsg::ValChanged,
                    )
                    .key(103)
                    .auto_focus(state.original_key.is_some()),
                ])
                .gap(1),
            ])
            .gap(3)
            .h_flex(1)
            .justify_center(),
        )
        .footer(modal_footer(elements![
            button(strings.btn_cancel, Variant::Secondary, MacroEditorMsg::Cancel).w(84),
            button(strings.btn_save, Variant::Primary, MacroEditorMsg::Submit)
                .disabled(!can_submit)
                .w(88),
        ]))
        .build()
}

/// Displays the declarative modal dialog for adding or editing a shorthand macro entry.
pub fn show_macro_dialog(parent: isize, edit_data: Option<MacroEditData>) {
    let strings = crate::language::current();
    let title = if edit_data.is_some() {
        strings.dlg_macro_title_edit
    } else {
        strings.dlg_macro_title_add
    };

    let (key, val, macro_type_idx, original_key) = if let Some(data) = edit_data {
        let type_idx = match data.macro_type {
            MacroType::Normal => 0,
            MacroType::StartConsonant => 1,
            MacroType::EndConsonant => 2,
        };
        (data.key, data.value, type_idx, Some(data.original_key))
    } else {
        (String::new(), String::new(), 0, None)
    };

    let state = Arc::new(Mutex::new(MacroEditorState {
        original_key,
        key,
        val,
        macro_type_idx,
    }));

    let state_for_render = Arc::clone(&state);
    let state_for_msg = Arc::clone(&state);

    let initial_view = render_macro_editor_view(&state_for_render.lock().unwrap());
    let config = ModalConfig::new(title, 440, 205);

    open_modal_with_wndproc(
        parent,
        config,
        initial_view,
        move |handle, msg| {
            let mut st = state_for_msg.lock().unwrap();
            match msg {
                MacroEditorMsg::KeyChanged(new_key) => {
                    st.key = new_key;
                    let view = render_macro_editor_view(&st);
                    drop(st);
                    handle.render(view);
                }
                MacroEditorMsg::ValChanged(new_val) => {
                    st.val = new_val;
                    let view = render_macro_editor_view(&st);
                    drop(st);
                    handle.render(view);
                }
                MacroEditorMsg::TypeChanged(new_idx) => {
                    st.macro_type_idx = new_idx;
                    let view = render_macro_editor_view(&st);
                    drop(st);
                    handle.render(view);
                }
                MacroEditorMsg::Submit => {
                    let key = st.key.trim().to_string();
                    let val = st.val.trim().to_string();
                    if !key.is_empty() && !val.is_empty() {
                        let mtype = match st.macro_type_idx {
                            1 => MacroType::StartConsonant,
                            2 => MacroType::EndConsonant,
                            _ => MacroType::Normal,
                        };
                        let orig_key = st.original_key.clone();
                        drop(st);
                        crate::ui::controller::save_macro(orig_key.as_deref(), &key, &val, mtype);
                        handle.close();
                    }
                }
                MacroEditorMsg::Cancel => {
                    drop(st);
                    handle.close();
                }
            }
        },
        move |handle, msg, wparam, _lparam| match msg {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if wparam == 0x0D
                /* VK_RETURN */
                {
                    handle.dispatch(MacroEditorMsg::Submit);
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        },
    );
}
