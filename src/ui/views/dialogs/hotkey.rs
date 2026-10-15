//! Modal Dialog for capturing and customizing typing mode switch hotkey
//! Pixel-perfect native Win32 GDI implementation matching modern mockup.

use super::{attach_state, close_modal, detach_state, open_client_modal, reflect_dialog_messages, state};
use crate::engine::config::Hotkey;
use crate::platform::win32::{
    get_captured_display_text, get_captured_hotkey, get_pending_modifier_hotkey,
    set_captured_hotkey, start_combo_capture, stop_combo_capture, ENGINE_INSTANCE,
    WM_COMBO_CANCELLED, WM_COMBO_CAPTURED,
};
use crate::ui::colors::ThemePalette;
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::components::{Label, LabelTone, SurfaceRole};
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::win32::*;
use std::ptr::null_mut;

struct HotkeyDialogState {
    parent: isize,
    display_font: isize,
    is_listening: bool,
    is_inherit: bool,
    is_root: bool,
    target_profile_id: String,
    target_display: String,
    global_hotkey: Hotkey,
    current_hotkey: Option<Hotkey>,
    display_text: String,
    btn_display: Option<PushButton>,
}

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraHotkeyDialogClass";

/// Displays the modal Hotkey Capture Dialog centered over the parent control panel
pub fn show_hotkey_dialog(parent: isize) {
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
                (true, "global".to_string(), default_root_name.clone(), Some(global_hk), global_hk)
            } else if let Some(child) = engine.profile_manager.get_profile(&sel_id) {
                let name = format!("{} {} {}", strings.hotkey_apply_prefix, child.icon, child.name);
                (false, sel_id, name, child.switch_key, global_hk)
            } else {
                (true, "global".to_string(), default_root_name.clone(), Some(global_hk), global_hk)
            }
        } else {
            (true, "global".to_string(), default_root_name.clone(), Some(Hotkey::default()), Hotkey::default())
        }
    };

    let is_inherit = !is_root && current_hotkey.is_none();
    let display_text = if is_inherit {
        format!("{} ({})", strings.typing_inherit_prefix, global_hotkey.display_text())
    } else {
        current_hotkey.unwrap_or(global_hotkey).display_text()
    };

    set_captured_hotkey(current_hotkey, &display_text);

    open_client_modal(
        parent,
        strings.hotkey_dialog_title,
        440,
        336,
        CLASS_NAME,
        hotkey_dialog_wnd_proc,
        &REGISTERED,
        Box::new(HotkeyDialogState {
            parent,
            display_font: 0,
            is_listening: false,
            is_inherit,
            is_root,
            target_profile_id,
            target_display,
            global_hotkey,
            current_hotkey,
            display_text,
            btn_display: None,
        }),
    );
}

unsafe fn apply_hotkey_preset(hwnd: isize, preset: Option<Hotkey>, label: &str) {
    let Some(st) = (unsafe { state::<HotkeyDialogState>(hwnd) }) else {
        return;
    };
    let disp_hwnd = st.btn_display.as_ref().map_or(0, |b| b.hwnd());
    st.is_listening = false;
    st.is_inherit = preset.is_none();
    st.current_hotkey = preset;
    st.display_text = if preset.is_some() {
        label.to_string()
    } else {
        format!(
            "{} ({})",
            crate::language::current().typing_inherit_prefix,
            st.global_hotkey.display_text()
        )
    };
    set_captured_hotkey(st.current_hotkey, &st.display_text);
    stop_combo_capture();
    if disp_hwnd != 0 {
        unsafe {
            InvalidateRect(disp_hwnd, null_mut(), 1);
        }
    }
}

unsafe fn submit_hotkey(hwnd: isize) {
    let Some(st) = (unsafe { state::<HotkeyDialogState>(hwnd) }) else {
        return;
    };
    stop_combo_capture();
    st.is_listening = false;
    let strings = crate::language::current();
    if st.display_text.is_empty() || st.display_text == strings.hotkey_listening {
        return;
    }

    let final_hotkey = if st.is_inherit {
        None
    } else {
        let valid_hk = st
            .current_hotkey
            .or_else(get_captured_hotkey)
            .or_else(get_pending_modifier_hotkey);
        match valid_hk {
            Some(hk) if hk.is_valid() => Some(hk),
            _ => return,
        }
    };

    let is_root = st.is_root;
    let target_profile_id = st.target_profile_id.clone();
    crate::ui::controller::save_switch_hotkey(&target_profile_id, is_root, final_hotkey);
    unsafe {
        PostMessageW(hwnd, WM_CLOSE, 0, 0);
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn hotkey_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_CREATE: u32 = 0x0001;
    const WM_COMMAND: u32 = 0x0111;
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_CLOSE: u32 = 0x0010;
    const WM_NCDESTROY: u32 = 0x0082;

    unsafe {
        if let Some(res) = reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                attach_state::<HotkeyDialogState>(hwnd, lparam);
                let Some(st) = state::<HotkeyDialogState>(hwnd) else {
                    return -1;
                };
                let hfont_large = super::base::create_display_font();
                let hfont_normal = get_app_font_normal();
                let hfont_bold = get_app_font_bold();
                st.display_font = hfont_large;

                let strings = crate::language::current();

                Label::create(hwnd, strings.hotkey_selected_title, 20, 66, 400, 18, hfont_bold)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));

                // Interactive Hotkey Display Box
                let btn_display = PushButton::create_owner_draw(
                    hwnd,
                    0,
                    "",
                    20,
                    86,
                    400,
                    46,
                    hfont_large,
                );
                if let Some(ref btn) = btn_display {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let (is_listening, text, font) = state::<HotkeyDialogState>(hwnd)
                            .map(|s| (s.is_listening, s.display_text.clone(), s.display_font))
                            .unwrap_or((false, "Ctrl + Shift".to_string(), 0));

                        super::base::draw_key_box(dis, is_dark, is_listening, &text, false, false, font);
                        true
                    });
                    btn.set_on_click(move || {
                        let Some(st) = state::<HotkeyDialogState>(hwnd) else {
                            return;
                        };
                        let disp_hwnd = st.btn_display.as_ref().map_or(0, |b| b.hwnd());
                        st.is_listening = true;
                        st.is_inherit = false;
                        st.display_text = crate::language::current().hotkey_listening.to_string();
                        start_combo_capture(hwnd, false);
                        if disp_hwnd != 0 {
                            InvalidateRect(disp_hwnd, null_mut(), 1);
                        }
                    });
                }

                Label::create(hwnd, strings.hotkey_click_hint, 20, 136, 400, 18, hfont_normal)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
                Label::create(hwnd, strings.hotkey_presets_title, 20, 160, 400, 18, hfont_bold)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));

                // Preset buttons (Row 1 + Row 2)
                let presets = [
                    (
                        strings.hotkey_preset_ctrl_shift,
                        20,
                        182,
                        175,
                        Hotkey::new(true, true, false, false, 0),
                        "Ctrl + Shift",
                    ),
                    (
                        strings.hotkey_preset_alt_z,
                        203,
                        182,
                        155,
                        Hotkey::new(false, false, true, false, 0x5A),
                        "Alt + Z",
                    ),
                    (
                        strings.hotkey_preset_win_space,
                        20,
                        216,
                        100,
                        Hotkey::new(false, false, false, true, 0x20),
                        "Win + Space",
                    ),
                    (
                        strings.hotkey_preset_shift_space,
                        128,
                        216,
                        105,
                        Hotkey::new(false, true, false, false, 0x20),
                        "Shift + Space",
                    ),
                ];
                for (label, x, y, w, hk, text) in presets {
                    if let Some(btn) = PushButton::create_with_variant(
                        hwnd,
                        0,
                        label,
                        x,
                        y,
                        w,
                        28,
                        0,
                        ButtonVariant::Secondary,
                        None,
                    ) {
                        btn.on_click(move || apply_hotkey_preset(hwnd, Some(hk), text));
                    }
                }

                if !st.is_root
                    && let Some(btn) = PushButton::create_with_variant(
                        hwnd,
                        0,
                        strings.typing_inherit_global,
                        241,
                        216,
                        179,
                        28,
                        0,
                        ButtonVariant::Secondary,
                        None,
                    )
                {
                    btn.on_click(move || apply_hotkey_preset(hwnd, None, ""));
                }

                // Footer Buttons
                let palette_init = ThemePalette::get(is_current_dark());
                super::create_dialog_footer_buttons(
                    hwnd,
                    (strings.btn_cancel, 236, 84),
                    (strings.hotkey_btn_save, 328, 92, ButtonVariant::Primary),
                    280,
                    32,
                    Some(palette_init.bg_sidebar),
                    move || submit_hotkey(hwnd),
                );

                st.btn_display = btn_display;

                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let palette = ThemePalette::get(is_current_dark());
                    let strings = crate::language::current();

                    super::base::fill_dialog_bg(hwnd, hdc, &palette);
                    super::base::draw_dialog_callout(
                        hdc,
                        &palette,
                        RECT { left: 20, top: 12, right: 420, bottom: 58 },
                        strings.hotkey_dialog_prompt,
                    );

                    let default_root = format!("🌐 {}", strings.banner_root_title);
                    let target_display = state::<HotkeyDialogState>(hwnd)
                        .map(|s| s.target_display.clone())
                        .unwrap_or(default_root);
                    super::base::draw_dialog_footer(hdc, &palette, 440, 264, 336, Some(&target_display));

                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_COMBO_CAPTURED => {
                let text = get_captured_display_text();
                let hotkey = get_captured_hotkey();
                let mut disp_hwnd = 0;

                if let Some(st) = state::<HotkeyDialogState>(hwnd) {
                    disp_hwnd = st.btn_display.as_ref().map_or(0, |b| b.hwnd());
                    if !text.is_empty() {
                        st.display_text = text;
                        if hotkey.is_some() {
                            st.current_hotkey = hotkey;
                            st.is_listening = false;
                            stop_combo_capture();
                        }
                    }
                }

                if disp_hwnd != 0 {
                    InvalidateRect(disp_hwnd, null_mut(), 0);
                    UpdateWindow(disp_hwnd);
                }
                0
            }
            WM_COMBO_CANCELLED => {
                let mut disp_hwnd = 0;
                if let Some(st) = state::<HotkeyDialogState>(hwnd) {
                    st.is_listening = false;
                    st.display_text = if st.is_inherit {
                        format!("{} ({})", crate::language::current().typing_inherit_prefix, st.global_hotkey.display_text())
                    } else {
                        st.current_hotkey.map(|h| h.display_text()).unwrap_or_default()
                    };
                    disp_hwnd = st.btn_display.as_ref().map_or(0, |b| b.hwnd());
                }
                stop_combo_capture();
                if disp_hwnd != 0 {
                    InvalidateRect(disp_hwnd, null_mut(), 1);
                }
                0
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let mut disp_hwnd = 0;
                    if let Some(st) = state::<HotkeyDialogState>(hwnd)
                        && st.is_listening
                    {
                        st.is_listening = false;
                        st.display_text = if st.is_inherit {
                            format!("{} ({})", crate::language::current().typing_inherit_prefix, st.global_hotkey.display_text())
                        } else {
                            st.current_hotkey.map(|h| h.display_text()).unwrap_or_default()
                        };
                        disp_hwnd = st.btn_display.as_ref().map_or(0, |b| b.hwnd());
                    }
                    stop_combo_capture();
                    if disp_hwnd != 0 {
                        InvalidateRect(disp_hwnd, null_mut(), 1);
                    }
                }
                0
            }
            WM_COMMAND => {
                match (wparam & 0xFFFF) as u32 {
                    IDOK => submit_hotkey(hwnd),
                    IDCANCEL => {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    _ => {}
                }
                0
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let vk = wparam as u32;
                if vk == 0x1B {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    return 0;
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ERASEBKGND => 1,
            WM_CLOSE => {
                stop_combo_capture();
                let parent = state::<HotkeyDialogState>(hwnd).map_or(0, |s| s.parent);
                close_modal(hwnd, parent);
                0
            }
            WM_NCDESTROY => {
                stop_combo_capture();
                if let Some(st) = detach_state::<HotkeyDialogState>(hwnd) {
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
