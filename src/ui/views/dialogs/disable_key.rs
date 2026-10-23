//! Declarative Modal Dialog for Key Disabling (Khóa / Vô Hiệu Hóa Phím)
//! High-polish declarative Web-like modal built on [`crate::ui_framework::core::modal`].

use std::sync::{Arc, Mutex};

use super::base::{active_profile_display_name, dialog_callout, key_capture_box};
use crate::engine::config::hotkey::vk_to_name;
use crate::platform::win32::{
    WM_SINGLE_KEY_CAPTURED, start_single_key_capture, stop_single_key_capture,
};
use crate::ui_framework::core::*;
use crate::ui_framework::win32::{WM_ACTIVATE, WM_KEYDOWN, WM_SYSKEYDOWN};

#[derive(Clone, Debug)]
struct DisableState {
    target_vk: Option<u32>,
    is_listening: bool,
    target_display: String,
}

#[derive(Clone, Debug)]
enum DisableMsg {
    ToggleListening,
    KeyCaptured(u32),
    SetPreset(u32),
    Submit,
    Cancel,
    FocusLost,
}

fn render_view(state: &DisableState) -> Element<DisableMsg> {
    let strings = crate::language::current();

    let (box_text, is_placeholder) = if state.is_listening {
        (format!("⌨️  {}", strings.dlg_remap_listening), false)
    } else if let Some(vk) = state.target_vk {
        let name = vk_to_name(vk);
        (
            if name.is_empty() {
                "Key".to_string()
            } else {
                name.to_string()
            },
            false,
        )
    } else {
        (strings.dlg_remap_unassigned.to_string(), true)
    };

    let presets_row1 = [
        ("🚫 Left Windows", 0x5B),
        ("🚫 Right Windows", 0x5C),
        ("🚫 Insert", 0x2D),
        ("🚫 Caps Lock", 0x14),
    ];
    let presets_row2 = [
        ("🚫 F1 (Trợ giúp)", 0x70),
        ("🚫 Scroll Lock", 0x91),
    ];

    modal_dialog()
        .header(dialog_callout(strings.dlg_disable_prompt))
        .body(
            col(elements![
                col(elements![
                    text(strings.dlg_disable_key).secondary().bold(),
                    key_capture_box(
                        &box_text,
                        state.is_listening,
                        is_placeholder,
                        DisableMsg::ToggleListening,
                    ),
                    text(strings.dlg_disable_click_hint).secondary().small(),
                ])
                .gap(1),
                col(elements![
                    text(strings.dlg_disable_presets_title).secondary().bold(),
                    row(presets_row1.map(|(label, vk)| {
                        button(label, Variant::Secondary, DisableMsg::SetPreset(vk)).into()
                    }))
                    .gap(2),
                    row(presets_row2.map(|(label, vk)| {
                        button(label, Variant::Secondary, DisableMsg::SetPreset(vk)).into()
                    }))
                    .gap(2),
                ])
                .gap(1),
            ])
            .gap(3),
        )
        .footer(
            row(elements![
                text(&state.target_display).secondary().small(),
                spacer(),
                button(strings.dlg_disable_btn_cancel, Variant::Secondary, DisableMsg::Cancel),
                button(
                    strings.dlg_disable_btn_apply,
                    Variant::Danger,
                    DisableMsg::Submit,
                )
                .disabled(state.target_vk.is_none()),
            ])
            .gap(2)
            .items_center(),
        )
        .build()
}

/// Displays the modal Key Disabling Dialog centered over the parent control panel
pub fn show_disable_dialog(parent: isize) {
    let strings = crate::language::current();
    let target_display = active_profile_display_name();

    let state = Arc::new(Mutex::new(DisableState {
        target_vk: None,
        is_listening: false,
        target_display,
    }));

    let state_for_render = Arc::clone(&state);
    let state_for_msg = Arc::clone(&state);
    let state_for_raw = Arc::clone(&state);

    let initial_view = render_view(&state_for_render.lock().unwrap());
    let config = ModalConfig::new(strings.dlg_disable_title, 480, 360);

    open_modal_with_wndproc(
        parent,
        config,
        initial_view,
        move |handle, msg| {
            let mut st = state_for_msg.lock().unwrap();
            match msg {
                DisableMsg::ToggleListening => {
                    if st.is_listening {
                        st.is_listening = false;
                        stop_single_key_capture();
                    } else {
                        st.is_listening = true;
                        start_single_key_capture(handle.hwnd);
                    }
                    handle.render(render_view(&st));
                }
                DisableMsg::KeyCaptured(vk) => {
                    st.target_vk = Some(vk);
                    st.is_listening = false;
                    stop_single_key_capture();
                    handle.render(render_view(&st));
                }
                DisableMsg::SetPreset(vk) => {
                    st.target_vk = Some(vk);
                    st.is_listening = false;
                    stop_single_key_capture();
                    handle.render(render_view(&st));
                }
                DisableMsg::Submit => {
                    stop_single_key_capture();
                    let target_vk = st.target_vk;
                    drop(st);
                    if let Some(vk) = target_vk
                        && vk != 0
                    {
                        crate::ui::controller::save_disable(vk);
                    }
                    handle.close();
                }
                DisableMsg::Cancel => {
                    stop_single_key_capture();
                    drop(st);
                    handle.close();
                }
                DisableMsg::FocusLost => {
                    if st.is_listening {
                        st.is_listening = false;
                        stop_single_key_capture();
                        let view = render_view(&st);
                        drop(st);
                        handle.render(view);
                    }
                }
            }
        },
        move |handle, msg, wparam, _lparam| match msg {
            WM_SINGLE_KEY_CAPTURED => {
                handle.dispatch(DisableMsg::KeyCaptured(wparam as u32));
                Some(0)
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let should_dispatch = state_for_raw
                        .try_lock()
                        .map(|st| st.is_listening)
                        .unwrap_or(false);
                    if should_dispatch {
                        handle.dispatch(DisableMsg::FocusLost);
                    }
                }
                None
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if wparam == 0x1B /* VK_ESCAPE */ {
                    let was_listening = state_for_raw
                        .try_lock()
                        .map(|st| st.is_listening)
                        .unwrap_or(false);
                    if was_listening {
                        handle.dispatch(DisableMsg::ToggleListening);
                        return Some(0);
                    }
                }
                None
            }
            _ => None,
        },
    );
}
