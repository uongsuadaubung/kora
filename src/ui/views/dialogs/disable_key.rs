//! Modal Dialog for Key Disabling (Khóa / Vô Hiệu Hóa Phím)
//! High-polish native Win32 GDI modal matching modern UI mockups.
//! Supports physical single-key capture, quick presets, and active profile assignment.

use super::{attach_state, close_modal, detach_state, open_client_modal, reflect_dialog_messages, state};
use crate::engine::config::hotkey::vk_to_name;
use crate::platform::win32::{
    start_single_key_capture, stop_single_key_capture, WM_SINGLE_KEY_CAPTURED,
};
use crate::ui::colors::ThemePalette;
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::components::{Label, LabelTone, SurfaceRole};
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::win32::*;
use std::ptr::null_mut;

struct DisableDialogState {
    parent: isize,
    display_font: isize,
    is_listening: bool,
    target_vk: Option<u32>,
    target_display: String,
    btn_box: Option<PushButton>,
}

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraDisableDialogClass";

/// Displays the modal Key Disabling Dialog centered over the parent control panel
pub fn show_disable_dialog(parent: isize) {
    let strings = crate::language::current();
    let target_display = super::base::active_profile_display_name();

    open_client_modal(
        parent,
        strings.dlg_disable_title,
        480,
        326,
        CLASS_NAME,
        disable_dialog_wnd_proc,
        &REGISTERED,
        Box::new(DisableDialogState {
            parent,
            display_font: 0,
            is_listening: false,
            target_vk: None,
            target_display,
            btn_box: None,
        }),
    );
}

unsafe fn submit_disable(hwnd: isize) {
    let Some(st) = (unsafe { state::<DisableDialogState>(hwnd) }) else {
        return;
    };
    stop_single_key_capture();
    if let Some(vk) = st.target_vk
        && vk != 0
    {
        crate::ui::controller::save_disable(vk);
        unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn disable_dialog_wnd_proc(
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
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_SYSKEYDOWN: u32 = 0x0104;
    const WM_NCDESTROY: u32 = 0x0082;

    unsafe {
        if let Some(res) = reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                attach_state::<DisableDialogState>(hwnd, lparam);
                let Some(st) = state::<DisableDialogState>(hwnd) else {
                    return -1;
                };
                let hfont_large = super::base::create_display_font();
                let hfont_normal = get_app_font_normal();
                let hfont_bold = get_app_font_bold();
                st.display_font = hfont_large;

                let palette_init = ThemePalette::get(is_current_dark());
                let strings = crate::language::current();

                Label::create(hwnd, strings.dlg_disable_key, 20, 74, 440, 18, hfont_bold)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));

                // Interactive Key Capture Box
                let btn_box = PushButton::create_owner_draw(
                    hwnd,
                    0,
                    "",
                    20,
                    96,
                    440,
                    44,
                    hfont_large,
                );
                if let Some(ref btn) = btn_box {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let (is_listening, vk_opt, font) = state::<DisableDialogState>(hwnd)
                            .map(|s| (s.is_listening, s.target_vk, s.display_font))
                            .unwrap_or((false, None, 0));

                        let strings = crate::language::current();
                        let (text, is_placeholder) = if is_listening {
                            (strings.dlg_remap_listening, false)
                        } else if let Some(vk) = vk_opt {
                            let name = vk_to_name(vk);
                            (if name.is_empty() { "Key" } else { name }, false)
                        } else {
                            (strings.dlg_remap_unassigned, true)
                        };

                        super::base::draw_key_box(
                            dis,
                            is_dark,
                            is_listening,
                            text,
                            is_placeholder,
                            true,
                            font,
                        );
                        true
                    });
                    btn.set_on_click(move || {
                        let Some(st) = state::<DisableDialogState>(hwnd) else {
                            return;
                        };
                        let h_box = st.btn_box.as_ref().map_or(0, |b| b.hwnd());
                        if st.is_listening {
                            st.is_listening = false;
                            stop_single_key_capture();
                        } else {
                            st.is_listening = true;
                            start_single_key_capture(hwnd);
                        }
                        if h_box != 0 {
                            InvalidateRect(h_box, null_mut(), 1);
                        }
                    });
                }

                Label::create(hwnd, strings.dlg_disable_click_hint, 20, 145, 440, 18, hfont_normal)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
                Label::create(hwnd, strings.dlg_disable_presets_title, 20, 172, 440, 18, hfont_bold)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));

                // 6 Presets (Row 1 + Row 2)
                let presets = [
                    ("🚫 Left Windows", 20, 196, 98, 0x5B),
                    ("🚫 Right Windows", 124, 196, 104, 0x5C),
                    ("🚫 Insert (Ghi đè chữ)", 234, 196, 126, 0x2D),
                    ("🚫 Caps Lock", 366, 196, 94, 0x14),
                    ("🚫 F1 (Trợ giúp)", 20, 230, 100, 0x70),
                    ("🚫 Scroll Lock", 126, 230, 100, 0x91),
                ];
                for (label, x, y, w, vk) in presets {
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
                        Some(palette_init.bg_window),
                    ) {
                        btn.on_click(move || set_preset_vk(hwnd, vk));
                    }
                }

                // Footer Buttons
                super::create_dialog_footer_buttons(
                    hwnd,
                    (strings.dlg_disable_btn_cancel, 264, 82),
                    (
                        strings.dlg_disable_btn_apply,
                        354,
                        106,
                        ButtonVariant::DangerSolid,
                    ),
                    280,
                    32,
                    Some(palette_init.bg_sidebar),
                    move || submit_disable(hwnd),
                );

                st.btn_box = btn_box;

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
                        RECT { left: 20, top: 14, right: 460, bottom: 62 },
                        strings.dlg_disable_prompt,
                    );

                    let default_root = format!("🌐 {}", strings.banner_root_title);
                    let target_display = state::<DisableDialogState>(hwnd)
                        .map(|s| s.target_display.clone())
                        .unwrap_or(default_root);
                    super::base::draw_dialog_footer(hdc, &palette, 480, 264, 326, Some(&target_display));

                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_SINGLE_KEY_CAPTURED => {
                let vk = wparam as u32;
                let mut h_box = 0;

                if let Some(st) = state::<DisableDialogState>(hwnd) {
                    st.target_vk = Some(vk);
                    st.is_listening = false;
                    stop_single_key_capture();
                    h_box = st.btn_box.as_ref().map_or(0, |b| b.hwnd());
                }

                if h_box != 0 {
                    InvalidateRect(h_box, null_mut(), 0);
                    UpdateWindow(h_box);
                }
                0
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let mut h_box = 0;
                    if let Some(st) = state::<DisableDialogState>(hwnd)
                        && st.is_listening
                    {
                        st.is_listening = false;
                        h_box = st.btn_box.as_ref().map_or(0, |b| b.hwnd());
                    }
                    stop_single_key_capture();
                    if h_box != 0 {
                        InvalidateRect(h_box, null_mut(), 1);
                    }
                }
                0
            }
            WM_COMMAND => {
                match (wparam & 0xFFFF) as u32 {
                    IDOK => submit_disable(hwnd),
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
                    let mut was_listening = false;
                    let mut h_box = 0;
                    if let Some(st) = state::<DisableDialogState>(hwnd)
                        && st.is_listening
                    {
                        st.is_listening = false;
                        was_listening = true;
                        h_box = st.btn_box.as_ref().map_or(0, |b| b.hwnd());
                    }
                    stop_single_key_capture();
                    if was_listening {
                        if h_box != 0 {
                            InvalidateRect(h_box, null_mut(), 1);
                        }
                    } else {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    return 0;
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ERASEBKGND => 1,
            WM_CLOSE => {
                stop_single_key_capture();
                let parent = state::<DisableDialogState>(hwnd).map_or(0, |s| s.parent);
                close_modal(hwnd, parent);
                0
            }
            WM_NCDESTROY => {
                stop_single_key_capture();
                if let Some(st) = detach_state::<DisableDialogState>(hwnd) {
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn set_preset_vk(hwnd: isize, vk: u32) {
    stop_single_key_capture();
    let mut h_box = 0;
    if let Some(st) = unsafe { state::<DisableDialogState>(hwnd) } {
        st.target_vk = Some(vk);
        st.is_listening = false;
        h_box = st.btn_box.as_ref().map_or(0, |b| b.hwnd());
    }
    if h_box != 0 {
        unsafe {
            InvalidateRect(h_box, null_mut(), 1);
        }
    }
}
