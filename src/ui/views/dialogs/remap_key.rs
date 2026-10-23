//! Declarative Modal Dialog for Key Remapping (Đổi Phím)
//! High-polish declarative Web-like modal built on [`crate::ui_framework::core::modal`].

use std::sync::{Arc, Mutex};

use super::base::{active_profile_display_name, dialog_callout, key_capture_box};
use crate::engine::config::hotkey::vk_to_name;
use crate::platform::win32::{
    WM_SINGLE_KEY_CAPTURED, start_single_key_capture, stop_single_key_capture,
};
use crate::ui_framework::core::*;
use crate::ui_framework::win32::{WM_ACTIVATE, WM_KEYDOWN, WM_SYSKEYDOWN};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveCaptureTarget {
    None,
    From,
    To,
}

#[derive(Clone, Debug)]
struct RemapState {
    active_target: ActiveCaptureTarget,
    original_from_vk: Option<u32>,
    from_vk: Option<u32>,
    to_vk: Option<u32>,
    target_display: String,
}

#[derive(Clone, Debug)]
enum RemapMsg {
    ToggleCapture(ActiveCaptureTarget),
    KeyCaptured(u32),
    SetPreset(u32, u32),
    Submit,
    Cancel,
    FocusLost,
}

fn render_view(state: &RemapState) -> Element<RemapMsg> {
    let strings = crate::language::current();

    let (box_from_text, from_is_placeholder) = if state.active_target == ActiveCaptureTarget::From {
        (format!("⌨️  {}", strings.dlg_remap_listening), false)
    } else if let Some(vk) = state.from_vk {
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

    let (box_to_text, to_is_placeholder) = if state.active_target == ActiveCaptureTarget::To {
        (format!("⌨️  {}", strings.dlg_remap_listening), false)
    } else if let Some(vk) = state.to_vk {
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

    let presets = [
        ("Caps Lock ➔ Ctrl", 0x14, 0x11),
        ("Caps Lock ➔ Escape", 0x14, 0x1B),
        ("Insert ➔ Delete", 0x2D, 0x2E),
        ("F1 ➔ Escape", 0x70, 0x1B),
    ];

    let can_submit = state.from_vk.is_some()
        && state.to_vk.is_some()
        && state.from_vk != state.to_vk;

    modal_dialog()
        .header(dialog_callout(strings.dlg_remap_prompt))
        .body(
            col(elements![
                row(elements![
                    col(elements![
                        text(strings.dlg_remap_from).secondary().bold(),
                        key_capture_box(
                            &box_from_text,
                            state.active_target == ActiveCaptureTarget::From,
                            from_is_placeholder,
                            RemapMsg::ToggleCapture(ActiveCaptureTarget::From),
                        ),
                        text(strings.dlg_remap_click_hint).secondary().small(),
                    ])
                    .flex_1()
                    .gap(1),
                    col(elements![text("➜").accent().title_font()])
                        .w(48)
                        .items_center()
                        .justify_center()
                        .pad_xy(2, 0),
                    col(elements![
                        text(strings.dlg_remap_to).secondary().bold(),
                        key_capture_box(
                            &box_to_text,
                            state.active_target == ActiveCaptureTarget::To,
                            to_is_placeholder,
                            RemapMsg::ToggleCapture(ActiveCaptureTarget::To),
                        ),
                        text(strings.dlg_remap_click_hint).secondary().small(),
                    ])
                    .flex_1()
                    .gap(1),
                ])
                .items_center(),
                col(elements![
                    text(strings.dlg_remap_presets_title).secondary().bold(),
                    row(presets.map(|(label, from_vk, to_vk)| {
                        button(label, Variant::Secondary, RemapMsg::SetPreset(from_vk, to_vk)).into()
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
                button(strings.dlg_remap_btn_cancel, Variant::Secondary, RemapMsg::Cancel),
                button(
                    strings.dlg_remap_btn_apply,
                    Variant::Primary,
                    RemapMsg::Submit,
                )
                .disabled(!can_submit),
            ])
            .gap(2)
            .items_center(),
        )
        .build()
}

/// Displays the modal Key Remap Dialog centered over the parent control panel
pub fn show_remap_dialog(parent: isize) {
    show_remap_dialog_with_keys(parent, None, None);
}

/// Displays the modal Key Remap Dialog with optional initial keys for editing
pub fn show_remap_dialog_with_keys(
    parent: isize,
    initial_from: Option<u32>,
    initial_to: Option<u32>,
) {
    let strings = crate::language::current();
    let target_display = active_profile_display_name();

    let title_str = if initial_from.is_some() {
        format!("{} ({})", strings.action_edit, strings.dlg_remap_title)
    } else {
        strings.dlg_remap_title.to_string()
    };

    let state = Arc::new(Mutex::new(RemapState {
        active_target: ActiveCaptureTarget::None,
        original_from_vk: initial_from,
        from_vk: initial_from,
        to_vk: initial_to,
        target_display,
    }));

    let state_for_render = Arc::clone(&state);
    let state_for_msg = Arc::clone(&state);
    let state_for_raw = Arc::clone(&state);

    let initial_view = render_view(&state_for_render.lock().unwrap());
    let config = ModalConfig::new(&title_str, 500, 320);

    open_modal_with_wndproc(
        parent,
        config,
        initial_view,
        move |handle, msg| {
            let mut st = state_for_msg.lock().unwrap();
            match msg {
                RemapMsg::ToggleCapture(target) => {
                    if st.active_target == target {
                        st.active_target = ActiveCaptureTarget::None;
                        stop_single_key_capture();
                    } else {
                        st.active_target = target;
                        start_single_key_capture(handle.hwnd);
                    }
                    handle.render(render_view(&st));
                }
                RemapMsg::KeyCaptured(vk) => {
                    match st.active_target {
                        ActiveCaptureTarget::From => {
                            st.from_vk = Some(vk);
                            st.active_target = ActiveCaptureTarget::None;
                            stop_single_key_capture();
                        }
                        ActiveCaptureTarget::To => {
                            st.to_vk = Some(vk);
                            st.active_target = ActiveCaptureTarget::None;
                            stop_single_key_capture();
                        }
                        ActiveCaptureTarget::None => {}
                    }
                    handle.render(render_view(&st));
                }
                RemapMsg::SetPreset(from, to) => {
                    st.from_vk = Some(from);
                    st.to_vk = Some(to);
                    st.active_target = ActiveCaptureTarget::None;
                    stop_single_key_capture();
                    handle.render(render_view(&st));
                }
                RemapMsg::Submit => {
                    stop_single_key_capture();
                    let (orig, from_opt, to_opt) =
                        (st.original_from_vk, st.from_vk, st.to_vk);
                    drop(st);
                    if let (Some(from), Some(to)) = (from_opt, to_opt)
                        && from != 0
                        && to != 0
                        && from != to
                    {
                        crate::ui::controller::save_remap(orig, from, to);
                    }
                    handle.close();
                }
                RemapMsg::Cancel => {
                    stop_single_key_capture();
                    drop(st);
                    handle.close();
                }
                RemapMsg::FocusLost => {
                    if st.active_target != ActiveCaptureTarget::None {
                        st.active_target = ActiveCaptureTarget::None;
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
                handle.dispatch(RemapMsg::KeyCaptured(wparam as u32));
                Some(0)
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let should_dispatch = state_for_raw
                        .try_lock()
                        .map(|st| st.active_target != ActiveCaptureTarget::None)
                        .unwrap_or(false);
                    if should_dispatch {
                        handle.dispatch(RemapMsg::FocusLost);
                    }
                }
                None
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if wparam == 0x1B /* VK_ESCAPE */ {
                    let was_capturing = state_for_raw
                        .try_lock()
                        .map(|st| st.active_target != ActiveCaptureTarget::None)
                        .unwrap_or(false);
                    if was_capturing {
                        handle.dispatch(RemapMsg::ToggleCapture(ActiveCaptureTarget::None));
                        return Some(0);
                    }
                }
                None
            }
            _ => None,
        },
    );
}
