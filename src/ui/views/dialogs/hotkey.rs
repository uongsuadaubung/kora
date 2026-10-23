//! Declarative Modal Dialog for capturing and customizing Hotkeys / Shortcuts
//! High-polish declarative Web-like modal built on [`crate::ui_framework::core::modal`].

use std::sync::{Arc, Mutex};

use super::base::{dialog_callout, dialog_footer, key_capture_box};
use crate::engine::config::Hotkey;
use crate::language::LanguageStrings;
use crate::platform::win32::{
    ENGINE_INSTANCE, WM_COMBO_CANCELLED, WM_COMBO_CAPTURED, get_captured_display_text,
    get_captured_hotkey, get_pending_modifier_hotkey, set_captured_hotkey, start_combo_capture,
    stop_combo_capture,
};
use crate::ui_framework::core::{
    button, col, modal_dialog, open_modal_with_wndproc, row, Element, ModalConfig, Variant,
};
use crate::ui_framework::win32::{WM_ACTIVATE, WM_KEYDOWN, WM_SYSKEYDOWN};

/// A common / suggested shortcut preset that can be clicked to quickly populate the hotkey.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotkeyPreset {
    pub label: String,
    pub hotkey: Hotkey,
}

impl HotkeyPreset {
    pub fn new(label: impl Into<String>, hotkey: Hotkey) -> Self {
        Self {
            label: label.into(),
            hotkey,
        }
    }
}

/// Configuration options for launching the reusable Declarative Hotkey Modal.
pub struct HotkeyModalConfig {
    pub title: String,
    pub prompt: String,
    pub target_display: String,
    pub current_hotkey: Option<Hotkey>,
    pub inherit_hotkey: Option<Hotkey>,
    pub presets: Vec<HotkeyPreset>,
    pub on_save: Box<dyn Fn(Option<Hotkey>) + Send + Sync + 'static>,
}

impl HotkeyModalConfig {
    pub fn new(
        title: impl Into<String>,
        prompt: impl Into<String>,
        on_save: impl Fn(Option<Hotkey>) + Send + Sync + 'static,
    ) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            target_display: String::new(),
            current_hotkey: None,
            inherit_hotkey: None,
            presets: Vec::new(),
            on_save: Box::new(on_save),
        }
    }

    pub fn with_target_display(mut self, target_display: impl Into<String>) -> Self {
        self.target_display = target_display.into();
        self
    }

    pub fn with_current_hotkey(mut self, current: Option<Hotkey>) -> Self {
        self.current_hotkey = current;
        self
    }

    pub fn with_inherit_hotkey(mut self, inherit: Option<Hotkey>) -> Self {
        self.inherit_hotkey = inherit;
        self
    }

    pub fn with_presets(mut self, presets: impl IntoIterator<Item = HotkeyPreset>) -> Self {
        self.presets = presets.into_iter().collect();
        self
    }
}

/// Standard switch hotkey presets (Ctrl + Shift, Alt + Z, Win + Space, Shift + Space).
pub fn switch_hotkey_presets(strings: &LanguageStrings) -> Vec<HotkeyPreset> {
    vec![
        HotkeyPreset::new(
            strings.hotkey_preset_ctrl_shift,
            Hotkey::new(true, true, false, false, 0),
        ),
        HotkeyPreset::new(
            strings.hotkey_preset_alt_z,
            Hotkey::new(false, false, true, false, 0x5A),
        ),
        HotkeyPreset::new(
            strings.hotkey_preset_win_space,
            Hotkey::new(false, false, false, true, 0x20),
        ),
        HotkeyPreset::new(
            strings.hotkey_preset_shift_space,
            Hotkey::new(false, true, false, false, 0x20),
        ),
    ]
}

/// Standard target capture presets (Ctrl + ~, Ctrl + Alt + K, Ctrl + Shift + K, Alt + ~).
pub fn target_capture_presets(strings: &LanguageStrings) -> Vec<HotkeyPreset> {
    vec![
        HotkeyPreset::new(
            strings.hotkey_preset_ctrl_tilde,
            Hotkey::new(true, false, false, false, 0xC0),
        ),
        HotkeyPreset::new(
            "Ctrl + Alt + K",
            Hotkey::new(true, false, true, false, 0x4B),
        ),
        HotkeyPreset::new(
            "Ctrl + Shift + K",
            Hotkey::new(true, true, false, false, 0x4B),
        ),
        HotkeyPreset::new(
            "Alt + ~",
            Hotkey::new(false, false, true, false, 0xC0),
        ),
    ]
}

#[derive(Clone, Debug)]
pub struct HotkeyState {
    pub prompt: String,
    pub target_display: String,
    pub current_hotkey: Option<Hotkey>,
    pub inherit_hotkey: Option<Hotkey>,
    pub is_inherit: bool,
    pub is_listening: bool,
    pub display_text: String,
    pub presets: Vec<HotkeyPreset>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HotkeyMsg {
    ToggleListening,
    ComboCaptured(Option<Hotkey>, String),
    ComboCancelled,
    SetPreset(usize),
    SetInherit,
    Submit,
    Cancel,
    FocusLost,
    EscapePressed,
}

pub fn render_hotkey_view(state: &HotkeyState) -> Element<HotkeyMsg> {
    let strings = crate::language::current();

    let (box_text, is_placeholder) = if state.is_listening {
        (strings.hotkey_listening.to_string(), false)
    } else if state.is_inherit {
        let global_str = state
            .inherit_hotkey
            .map(|h| h.display_text())
            .unwrap_or_default();
        (
            format!("{} ({})", strings.typing_inherit_prefix, global_str),
            false,
        )
    } else if !state.display_text.is_empty() {
        (state.display_text.clone(), false)
    } else if let Some(hk) = state.current_hotkey {
        (hk.display_text(), false)
    } else {
        (strings.dlg_remap_unassigned.to_string(), true)
    };

    let mut preset_rows = Vec::new();
    let mut idx = 0;
    for chunk in state.presets.chunks(2) {
        let mut row_btns = Vec::new();
        for preset in chunk {
            let p_idx = idx;
            row_btns.push(
                button(&preset.label, Variant::Secondary, HotkeyMsg::SetPreset(p_idx)).flex_1(),
            );
            idx += 1;
        }
        preset_rows.push(row(row_btns).gap(2));
    }

    if let Some(inherit_hk) = state.inherit_hotkey {
        let inherit_label = format!(
            "🌐 {} ({})",
            strings.typing_inherit_global,
            inherit_hk.display_text()
        );
        preset_rows.push(row([button(
            inherit_label,
            Variant::Secondary,
            HotkeyMsg::SetInherit,
        )
        .flex_1()]));
    }

    let can_submit = state.is_inherit
        || state.current_hotkey.is_some()
        || (!state.display_text.is_empty() && state.display_text != strings.hotkey_listening);

    modal_dialog()
        .header(dialog_callout(&state.prompt))
        .body(
            col([
                col([
                    crate::ui_framework::core::text(strings.hotkey_selected_title)
                        .secondary()
                        .bold(),
                    key_capture_box(
                        &box_text,
                        state.is_listening,
                        is_placeholder,
                        HotkeyMsg::ToggleListening,
                    ),
                    crate::ui_framework::core::text(strings.hotkey_click_hint)
                        .secondary()
                        .small(),
                ])
                .gap(1),
                if !preset_rows.is_empty() {
                    col([
                        crate::ui_framework::core::text(strings.hotkey_presets_title)
                            .secondary()
                            .bold(),
                        col(preset_rows).gap(2),
                    ])
                    .gap(1)
                } else {
                    col([])
                },
            ])
            .gap(3),
        )
        .footer(dialog_footer(
            &state.target_display,
            strings.btn_cancel,
            HotkeyMsg::Cancel,
            strings.hotkey_btn_save,
            HotkeyMsg::Submit,
            Variant::Primary,
            !can_submit,
        ))
        .build()
}

/// Displays the reusable modal Hotkey Capture Dialog with configurable presets and save callback.
pub fn show_hotkey_modal(parent: isize, config: HotkeyModalConfig) {
    let strings = crate::language::current();
    let is_inherit = config.current_hotkey.is_none() && config.inherit_hotkey.is_some();
    let display_text = if is_inherit {
        let global_str = config
            .inherit_hotkey
            .map(|h| h.display_text())
            .unwrap_or_default();
        format!("{} ({})", strings.typing_inherit_prefix, global_str)
    } else if let Some(hk) = config.current_hotkey {
        hk.display_text()
    } else {
        String::new()
    };

    set_captured_hotkey(config.current_hotkey, &display_text);

    let row_count = (config.presets.len() + 1) / 2
        + if config.inherit_hotkey.is_some() {
            1
        } else {
            0
        };
    let modal_height = (290 + (row_count as i32 * 36)).max(340);

    let on_save = Arc::new(config.on_save);
    let state = Arc::new(Mutex::new(HotkeyState {
        prompt: config.prompt,
        target_display: config.target_display,
        current_hotkey: config.current_hotkey,
        inherit_hotkey: config.inherit_hotkey,
        is_inherit,
        is_listening: false,
        display_text,
        presets: config.presets,
    }));

    let state_for_render = Arc::clone(&state);
    let state_for_msg = Arc::clone(&state);
    let state_for_raw = Arc::clone(&state);
    let on_save_for_msg = Arc::clone(&on_save);

    let initial_view = render_hotkey_view(&state_for_render.lock().unwrap());
    let modal_cfg = ModalConfig::new(&config.title, 480, modal_height);

    open_modal_with_wndproc(
        parent,
        modal_cfg,
        initial_view,
        move |handle, msg| {
            let mut st = state_for_msg.lock().unwrap();
            match msg {
                HotkeyMsg::ToggleListening => {
                    if st.is_listening {
                        st.is_listening = false;
                        stop_combo_capture();
                        st.display_text = if st.is_inherit {
                            let g = st
                                .inherit_hotkey
                                .map(|h| h.display_text())
                                .unwrap_or_default();
                            format!("{} ({})", strings.typing_inherit_prefix, g)
                        } else {
                            st.current_hotkey
                                .map(|h| h.display_text())
                                .unwrap_or_default()
                        };
                    } else {
                        st.is_listening = true;
                        st.is_inherit = false;
                        st.display_text = strings.hotkey_listening.to_string();
                        start_combo_capture(handle.hwnd, false);
                    }
                    handle.render(render_hotkey_view(&st));
                }
                HotkeyMsg::ComboCaptured(hk_opt, text) => {
                    if !text.is_empty() {
                        st.display_text = text;
                        if let Some(hk) = hk_opt {
                            st.current_hotkey = Some(hk);
                            st.is_listening = false;
                            st.is_inherit = false;
                            stop_combo_capture();
                        }
                    }
                    handle.render(render_hotkey_view(&st));
                }
                HotkeyMsg::ComboCancelled => {
                    stop_combo_capture();
                    st.is_listening = false;
                    st.display_text = if st.is_inherit {
                        let g = st
                            .inherit_hotkey
                            .map(|h| h.display_text())
                            .unwrap_or_default();
                        format!("{} ({})", strings.typing_inherit_prefix, g)
                    } else {
                        st.current_hotkey
                            .map(|h| h.display_text())
                            .unwrap_or_default()
                    };
                    handle.render(render_hotkey_view(&st));
                }
                HotkeyMsg::SetPreset(idx) => {
                    let preset = st.presets.get(idx).cloned();
                    if let Some(p) = preset {
                        st.current_hotkey = Some(p.hotkey);
                        st.display_text = p.label.clone();
                        st.is_listening = false;
                        st.is_inherit = false;
                        set_captured_hotkey(Some(p.hotkey), &p.label);
                        stop_combo_capture();
                        handle.render(render_hotkey_view(&st));
                    }
                }
                HotkeyMsg::SetInherit => {
                    st.current_hotkey = None;
                    st.is_inherit = true;
                    st.is_listening = false;
                    let g = st
                        .inherit_hotkey
                        .map(|h| h.display_text())
                        .unwrap_or_default();
                    st.display_text = format!("{} ({})", strings.typing_inherit_prefix, g);
                    set_captured_hotkey(None, &st.display_text);
                    stop_combo_capture();
                    handle.render(render_hotkey_view(&st));
                }
                HotkeyMsg::Submit => {
                    stop_combo_capture();
                    let final_hk = if st.is_inherit {
                        None
                    } else {
                        st.current_hotkey
                            .or_else(get_captured_hotkey)
                            .or_else(get_pending_modifier_hotkey)
                    };
                    if !st.is_inherit && final_hk.is_none() {
                        return;
                    }
                    if let Some(hk) = final_hk {
                        if !hk.is_valid() {
                            return;
                        }
                    }
                    handle.close();
                    on_save_for_msg(final_hk);
                }
                HotkeyMsg::Cancel => {
                    stop_combo_capture();
                    handle.close();
                }
                HotkeyMsg::FocusLost => {
                    if st.is_listening {
                        st.is_listening = false;
                        stop_combo_capture();
                        st.display_text = if st.is_inherit {
                            let g = st
                                .inherit_hotkey
                                .map(|h| h.display_text())
                                .unwrap_or_default();
                            format!("{} ({})", strings.typing_inherit_prefix, g)
                        } else {
                            st.current_hotkey
                                .map(|h| h.display_text())
                                .unwrap_or_default()
                        };
                        handle.render(render_hotkey_view(&st));
                    }
                }
                HotkeyMsg::EscapePressed => {
                    if st.is_listening {
                        st.is_listening = false;
                        stop_combo_capture();
                        st.display_text = if st.is_inherit {
                            let g = st
                                .inherit_hotkey
                                .map(|h| h.display_text())
                                .unwrap_or_default();
                            format!("{} ({})", strings.typing_inherit_prefix, g)
                        } else {
                            st.current_hotkey
                                .map(|h| h.display_text())
                                .unwrap_or_default()
                        };
                        handle.render(render_hotkey_view(&st));
                    } else {
                        stop_combo_capture();
                        handle.close();
                    }
                }
            }
        },
        move |handle, msg, wparam, _lparam| match msg {
            WM_COMBO_CAPTURED => {
                let text = get_captured_display_text();
                let hotkey = get_captured_hotkey();
                handle.dispatch(HotkeyMsg::ComboCaptured(hotkey, text));
                Some(0)
            }
            WM_COMBO_CANCELLED => {
                handle.dispatch(HotkeyMsg::ComboCancelled);
                Some(0)
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let should_dispatch = state_for_raw
                        .try_lock()
                        .map(|st| st.is_listening)
                        .unwrap_or(false);
                    if should_dispatch {
                        handle.dispatch(HotkeyMsg::FocusLost);
                    }
                }
                None
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if wparam == 0x1B /* VK_ESCAPE */ {
                    handle.dispatch(HotkeyMsg::EscapePressed);
                    return Some(0);
                }
                None
            }
            _ => None,
        },
    );
}

/// Displays the modal Hotkey Capture Dialog with caller-specified presets for switching typing mode
pub fn show_hotkey_dialog_with_presets(
    parent: isize,
    presets: impl IntoIterator<Item = HotkeyPreset>,
) {
    let strings = crate::language::current();
    let default_root_name = format!(
        "{} 🌐 {}",
        strings.hotkey_apply_prefix, strings.banner_root_title
    );
    let (is_root, target_profile_id, target_display, current_hotkey, global_hotkey) = {
        if let Ok(guard) = ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let global_hk = engine.config().switch_key;
            let sel_id = engine.profile_manager.selected_id.clone();
            if sel_id == "global" {
                (
                    true,
                    "global".to_string(),
                    default_root_name.clone(),
                    Some(global_hk),
                    global_hk,
                )
            } else if let Some(child) = engine.profile_manager.get_profile(&sel_id) {
                let name = format!(
                    "{} {} {}",
                    strings.hotkey_apply_prefix, child.icon, child.name
                );
                (false, sel_id, name, child.switch_key, global_hk)
            } else {
                (
                    true,
                    "global".to_string(),
                    default_root_name.clone(),
                    Some(global_hk),
                    global_hk,
                )
            }
        } else {
            (
                true,
                "global".to_string(),
                default_root_name.clone(),
                Some(Hotkey::default()),
                Hotkey::default(),
            )
        }
    };

    let inherit_hotkey = if is_root { None } else { Some(global_hotkey) };

    let config = HotkeyModalConfig::new(
        strings.hotkey_dialog_title,
        strings.hotkey_dialog_prompt,
        move |final_hotkey| {
            crate::ui::controller::save_switch_hotkey(&target_profile_id, is_root, final_hotkey);
        },
    )
    .with_target_display(target_display)
    .with_current_hotkey(current_hotkey)
    .with_inherit_hotkey(inherit_hotkey)
    .with_presets(presets);

    show_hotkey_modal(parent, config);
}

/// Displays the modal Hotkey Capture Dialog centered over the parent control panel for switching typing mode
pub fn show_hotkey_dialog(parent: isize) {
    let strings = crate::language::current();
    show_hotkey_dialog_with_presets(parent, switch_hotkey_presets(strings));
}

/// Displays the modal Hotkey Capture Dialog with caller-specified presets for the target field capture shortcut
pub fn show_target_capture_hotkey_dialog_with_presets(
    parent: isize,
    presets: impl IntoIterator<Item = HotkeyPreset>,
) {
    let strings = crate::language::current();
    let default_root_name = format!(
        "{} 🎯 {}",
        strings.hotkey_apply_prefix, strings.typing_group_target_hotkey
    );
    let global_hotkey = if let Ok(guard) = ENGINE_INSTANCE.lock()
        && let Some(ref engine) = *guard
    {
        engine.config().target_capture_hotkey
    } else {
        Hotkey::new(true, false, false, false, 0xC0)
    };

    let config = HotkeyModalConfig::new(
        strings.hotkey_target_dialog_title,
        strings.typing_target_hotkey_desc,
        move |final_hotkey| {
            if let Some(hk) = final_hotkey
                && hk.is_valid()
            {
                crate::ui::controller::save_target_capture_hotkey(hk);
            }
        },
    )
    .with_target_display(default_root_name)
    .with_current_hotkey(Some(global_hotkey))
    .with_presets(presets);

    show_hotkey_modal(parent, config);
}

/// Displays the modal Hotkey Capture Dialog specifically for the target field capture shortcut
pub fn show_target_capture_hotkey_dialog(parent: isize) {
    let strings = crate::language::current();
    show_target_capture_hotkey_dialog_with_presets(parent, target_capture_presets(strings));
}
