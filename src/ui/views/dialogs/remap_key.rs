//! Modal Dialog for Key Remapping (Đổi phím)
//! High-polish native Win32 GDI modal matching modern UI mockups.
//! Supports physical single-key capture, presets, and active profile assignment.

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveCaptureTarget {
    None,
    From,
    To,
}

struct RemapDialogState {
    parent: isize,
    display_font: isize,
    active_target: ActiveCaptureTarget,
    original_from_vk: Option<u32>,
    from_vk: Option<u32>,
    to_vk: Option<u32>,
    target_display: String,
    btn_from: Option<PushButton>,
    btn_to: Option<PushButton>,
}

/// Displays the modal Key Remap Dialog centered over the parent control panel
pub fn show_remap_dialog(parent: isize) {
    show_remap_dialog_with_keys(parent, None, None);
}

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraRemapDialogClass";

/// Displays the modal Key Remap Dialog with optional initial keys for editing
pub fn show_remap_dialog_with_keys(
    parent: isize,
    initial_from: Option<u32>,
    initial_to: Option<u32>,
) {
    let strings = crate::language::current();
    let target_display = super::base::active_profile_display_name();

    let title_str = if initial_from.is_some() {
        format!("{} ({})", strings.action_edit, strings.dlg_remap_title)
    } else {
        strings.dlg_remap_title.to_string()
    };

    open_client_modal(
        parent,
        &title_str,
        500,
        336,
        CLASS_NAME,
        remap_dialog_wnd_proc,
        &REGISTERED,
        Box::new(RemapDialogState {
            parent,
            display_font: 0,
            active_target: ActiveCaptureTarget::None,
            original_from_vk: initial_from,
            from_vk: initial_from,
            to_vk: initial_to,
            target_display,
            btn_from: None,
            btn_to: None,
        }),
    );
}

unsafe fn draw_remap_box(hwnd: isize, target: ActiveCaptureTarget, dis: &DRAWITEMSTRUCT, is_dark: bool) {
    let (is_listening, vk_opt, font) = (unsafe { state::<RemapDialogState>(hwnd) })
        .map(|s| {
            let vk = match target {
                ActiveCaptureTarget::From => s.from_vk,
                ActiveCaptureTarget::To => s.to_vk,
                ActiveCaptureTarget::None => None,
            };
            (s.active_target == target, vk, s.display_font)
        })
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

    super::base::draw_key_box(dis, is_dark, is_listening, text, is_placeholder, false, font);
}

unsafe fn toggle_capture(hwnd: isize, target: ActiveCaptureTarget) {
    let Some(st) = (unsafe { state::<RemapDialogState>(hwnd) }) else {
        return;
    };
    let h_from = st.btn_from.as_ref().map_or(0, |b| b.hwnd());
    let h_to = st.btn_to.as_ref().map_or(0, |b| b.hwnd());
    if st.active_target == target {
        st.active_target = ActiveCaptureTarget::None;
        stop_single_key_capture();
    } else {
        st.active_target = target;
        start_single_key_capture(hwnd);
    }
    unsafe {
        if h_from != 0 {
            InvalidateRect(h_from, null_mut(), 1);
        }
        if h_to != 0 {
            InvalidateRect(h_to, null_mut(), 1);
        }
    }
}

unsafe fn submit_remap(hwnd: isize) {
    let Some(st) = (unsafe { state::<RemapDialogState>(hwnd) }) else {
        return;
    };
    stop_single_key_capture();
    let (orig_from, from_opt, to_opt) = (st.original_from_vk, st.from_vk, st.to_vk);
    if let (Some(from), Some(to)) = (from_opt, to_opt)
        && from != 0
        && to != 0
        && from != to
    {
        crate::ui::controller::save_remap(orig_from, from, to);
        unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn remap_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_NCDESTROY: u32 = 0x0082;
    unsafe {
        if let Some(res) = reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                attach_state::<RemapDialogState>(hwnd, lparam);
                let Some(st) = state::<RemapDialogState>(hwnd) else {
                    return -1;
                };
                let hfont_large = super::base::create_display_font();
                let hfont_normal = get_app_font_normal();
                let hfont_bold = get_app_font_bold();
                st.display_font = hfont_large;

                let strings = crate::language::current();

                Label::create(hwnd, strings.dlg_remap_from, 20, 74, 205, 18, hfont_bold)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
                Label::create(hwnd, strings.dlg_remap_to, 275, 74, 205, 18, hfont_bold)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));

                // Interactive Box 1: From Key
                let btn_from = PushButton::create_owner_draw(
                    hwnd,
                    0,
                    "",
                    20,
                    96,
                    205,
                    48,
                    hfont_large,
                );
                if let Some(ref btn) = btn_from {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        draw_remap_box(hwnd, ActiveCaptureTarget::From, dis, is_dark);
                        true
                    });
                    btn.set_on_click(move || toggle_capture(hwnd, ActiveCaptureTarget::From));
                }

                // Center Arrow: ➜
                Label::create_centered(hwnd, "➜", 225, 108, 50, 24, hfont_large)
                    .map(|l| l.with_tone(LabelTone::Accent).on_surface(SurfaceRole::Window));

                // Interactive Box 2: To Key
                let btn_to = PushButton::create_owner_draw(
                    hwnd,
                    0,
                    "",
                    275,
                    96,
                    205,
                    48,
                    hfont_large,
                );
                if let Some(ref btn) = btn_to {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        draw_remap_box(hwnd, ActiveCaptureTarget::To, dis, is_dark);
                        true
                    });
                    btn.set_on_click(move || toggle_capture(hwnd, ActiveCaptureTarget::To));
                }

                Label::create(hwnd, strings.dlg_remap_click_hint, 20, 150, 205, 18, hfont_normal)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
                Label::create(hwnd, strings.dlg_remap_click_hint, 275, 150, 205, 18, hfont_normal)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
                Label::create(hwnd, strings.dlg_remap_presets_title, 20, 182, 460, 20, hfont_bold)
                    .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));

                let palette_init = ThemePalette::get(is_current_dark());

                // Row of 4 Presets
                let presets = [
                    ("Caps Lock ➔ Ctrl", 20, 110, 0x14, 0x11),
                    ("Caps Lock ➔ Escape", 138, 122, 0x14, 0x1B),
                    ("Insert ➔ Delete", 268, 104, 0x2D, 0x2E),
                    ("F1 ➔ Escape", 380, 100, 0x70, 0x1B),
                ];
                for (label, x, w, from_vk, to_vk) in presets {
                    if let Some(btn) = PushButton::create_with_variant(
                        hwnd,
                        0,
                        label,
                        x,
                        208,
                        w,
                        28,
                        0,
                        ButtonVariant::Secondary,
                        Some(palette_init.bg_window),
                    ) {
                        btn.on_click(move || apply_preset(hwnd, from_vk, to_vk));
                    }
                }

                // Footer Buttons
                super::create_dialog_footer_buttons(
                    hwnd,
                    (strings.dlg_remap_btn_cancel, 254, 76),
                    (strings.dlg_remap_btn_apply, 340, 140, ButtonVariant::Primary),
                    280,
                    32,
                    Some(palette_init.bg_window),
                    move || submit_remap(hwnd),
                );

                st.btn_from = btn_from;
                st.btn_to = btn_to;

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
                        RECT { left: 20, top: 14, right: 480, bottom: 58 },
                        strings.dlg_remap_prompt,
                    );

                    let default_root = format!("🌐 {}", strings.banner_root_title);
                    let target_display = state::<RemapDialogState>(hwnd)
                        .map(|s| s.target_display.clone())
                        .unwrap_or(default_root);
                    super::base::draw_dialog_footer(hdc, &palette, 500, 264, 336, Some(&target_display));

                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_SINGLE_KEY_CAPTURED => {
                let vk = wparam as u32;
                let mut h_from = 0;
                let mut h_to = 0;

                if let Some(st) = state::<RemapDialogState>(hwnd) {
                    h_from = st.btn_from.as_ref().map_or(0, |b| b.hwnd());
                    h_to = st.btn_to.as_ref().map_or(0, |b| b.hwnd());

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
                }

                if h_from != 0 {
                    InvalidateRect(h_from, null_mut(), 0);
                    UpdateWindow(h_from);
                }
                if h_to != 0 {
                    InvalidateRect(h_to, null_mut(), 0);
                    UpdateWindow(h_to);
                }
                0
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let mut h_from = 0;
                    let mut h_to = 0;
                    if let Some(st) = state::<RemapDialogState>(hwnd)
                        && st.active_target != ActiveCaptureTarget::None
                    {
                        st.active_target = ActiveCaptureTarget::None;
                        h_from = st.btn_from.as_ref().map_or(0, |b| b.hwnd());
                        h_to = st.btn_to.as_ref().map_or(0, |b| b.hwnd());
                    }
                    stop_single_key_capture();
                    if h_from != 0 {
                        InvalidateRect(h_from, null_mut(), 1);
                    }
                    if h_to != 0 {
                        InvalidateRect(h_to, null_mut(), 1);
                    }
                }
                0
            }
            WM_COMMAND => {
                match (wparam & 0xFFFF) as u32 {
                    IDOK => submit_remap(hwnd),
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
                    let mut was_capturing = false;
                    let mut h_from = 0;
                    let mut h_to = 0;
                    if let Some(st) = state::<RemapDialogState>(hwnd)
                        && st.active_target != ActiveCaptureTarget::None
                    {
                        st.active_target = ActiveCaptureTarget::None;
                        was_capturing = true;
                        h_from = st.btn_from.as_ref().map_or(0, |b| b.hwnd());
                        h_to = st.btn_to.as_ref().map_or(0, |b| b.hwnd());
                    }
                    stop_single_key_capture();
                    if was_capturing {
                        if h_from != 0 {
                            InvalidateRect(h_from, null_mut(), 1);
                        }
                        if h_to != 0 {
                            InvalidateRect(h_to, null_mut(), 1);
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
                let parent = state::<RemapDialogState>(hwnd).map_or(0, |s| s.parent);
                close_modal(hwnd, parent);
                0
            }
            WM_NCDESTROY => {
                stop_single_key_capture();
                if let Some(st) = detach_state::<RemapDialogState>(hwnd) {
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn apply_preset(hwnd: isize, from: u32, to: u32) {
    stop_single_key_capture();
    let mut h_from = 0;
    let mut h_to = 0;
    if let Some(st) = unsafe { state::<RemapDialogState>(hwnd) } {
        st.from_vk = Some(from);
        st.to_vk = Some(to);
        st.active_target = ActiveCaptureTarget::None;
        h_from = st.btn_from.as_ref().map_or(0, |b| b.hwnd());
        h_to = st.btn_to.as_ref().map_or(0, |b| b.hwnd());
    }
    unsafe {
        if h_from != 0 {
            InvalidateRect(h_from, null_mut(), 1);
        }
        if h_to != 0 {
            InvalidateRect(h_to, null_mut(), 1);
        }
    }
}
