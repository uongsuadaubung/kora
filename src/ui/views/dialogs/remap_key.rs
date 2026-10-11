//! Modal Dialog for Key Remapping (Đổi phím)
//! High-polish native Win32 GDI modal matching modern UI mockups.
//! Supports physical single-key capture, presets, and active profile assignment.

use crate::engine::config::hotkey::vk_to_name;
use crate::engine::profile::KeyRemapRule;
use crate::platform::win32::{
    start_single_key_capture, stop_single_key_capture, ENGINE_INSTANCE, WM_REMAP_KEY_CAPTURED,
};
use crate::ui::colors::ThemePalette;
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::tokens::id::*;
use crate::ui::tokens::{borders, radii};
use crate::ui::win32::*;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveCaptureTarget {
    None,
    From,
    To,
}

#[derive(Clone)]
struct RemapDialogState {
    active_target: ActiveCaptureTarget,
    original_from_vk: Option<u32>,
    from_vk: Option<u32>,
    to_vk: Option<u32>,
    target_display: String,
    hwnd_from_box: isize,
    hwnd_to_box: isize,
}

static DIALOG_PARENT_HWND: AtomicIsize = AtomicIsize::new(0);
static DISPLAY_FONT: AtomicIsize = AtomicIsize::new(0);
static STATE: Mutex<Option<RemapDialogState>> = Mutex::new(None);

/// Displays the modal Key Remap Dialog centered over the parent control panel
pub fn show_remap_dialog(parent: isize) {
    show_remap_dialog_with_keys(parent, None, None);
}

/// Displays the modal Key Remap Dialog with optional initial keys for editing
pub fn show_remap_dialog_with_keys(parent: isize, initial_from: Option<u32>, initial_to: Option<u32>) {
    let class_name = to_wide("KoraRemapDialogClass");

    unsafe {
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfn_wnd_proc = Some(remap_dialog_wnd_proc);
        wc.h_instance = 0;
        wc.h_cursor = LoadCursorW(0, IDC_ARROW);
        wc.lpsz_class_name = class_name.as_ptr();
        RegisterClassExW(&wc);

        DIALOG_PARENT_HWND.store(parent, Ordering::SeqCst);

        let strings = crate::language::current();
        let default_root_name = format!("🌐 {}", strings.banner_root_title);
        let target_display = {
            let guard = ENGINE_INSTANCE.lock().unwrap();
            if let Some(ref engine) = *guard {
                let sel_id = engine.profile_manager.selected_id.clone();
                if sel_id == "global" {
                    default_root_name
                } else if let Some(child) = engine.profile_manager.get_profile(&sel_id) {
                    format!("{} {}", child.icon, child.name)
                } else {
                    default_root_name
                }
            } else {
                default_root_name
            }
        };

        let from_vk = initial_from;
        let to_vk = initial_to;

        if let Ok(mut guard) = STATE.lock() {
            *guard = Some(RemapDialogState {
                active_target: ActiveCaptureTarget::None,
                original_from_vk: initial_from,
                from_vk,
                to_vk,
                target_display,
                hwnd_from_box: 0,
                hwnd_to_box: 0,
            });
        }

        // Client dimensions: 500 x 336
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: 500,
            bottom: 336,
        };
        let style = 0x00080000 | 0x10000000 | 0x00C00000; // WS_SYSMENU | WS_VISIBLE | WS_CAPTION
        let ex_style = 0x00010000 | 0x00000001; // WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME
        AdjustWindowRect(&mut rc, style, 0);

        let width = rc.right - rc.left;
        let height = rc.bottom - rc.top;

        let mut parent_rc: RECT = std::mem::zeroed();
        GetWindowRect(parent, &mut parent_rc);
        let x = parent_rc.left + ((parent_rc.right - parent_rc.left) - width) / 2;
        let y = parent_rc.top + ((parent_rc.bottom - parent_rc.top) - height) / 2;

        let title_str = if initial_from.is_some() {
            format!("{} ({})", strings.action_edit, strings.dlg_remap_title)
        } else {
            strings.dlg_remap_title.to_string()
        };
        let title = to_wide(&title_str);

        EnableWindow(parent, 0);

        let hwnd = CreateWindowExW(
            ex_style,
            class_name.as_ptr(),
            title.as_ptr(),
            style,
            x,
            y,
            width,
            height,
            parent,
            0,
            0,
            null_mut(),
        );

        if hwnd != 0 {
            apply_modern_window_styling(hwnd, is_current_dark());
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        } else {
            EnableWindow(parent, 1);
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
    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_COMMAND: u32 = 0x0111;
    const WM_DRAWITEM: u32 = 0x002B;
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_CLOSE: u32 = 0x0010;
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_SYSKEYDOWN: u32 = 0x0104;

    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct DRAWITEMSTRUCT {
        ctl_type: u32,
        ctl_id: u32,
        item_id: u32,
        item_action: u32,
        item_state: u32,
        hwnd_item: isize,
        hdc: isize,
        rc_item: RECT,
        item_data: usize,
    }

    unsafe {
        match msg {
            WM_CREATE => {
                let hfont_large = CreateFontW(
                    20,
                    0,
                    0,
                    0,
                    700,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    5,
                    0,
                    to_wide("Segoe UI").as_ptr(),
                );
                DISPLAY_FONT.store(hfont_large, Ordering::SeqCst);

                let strings = crate::language::current();

                // Interactive Box 1: From Key
                let btn_from = PushButton::create_owner_draw(
                    hwnd,
                    IDC_BTN_REMAP_FROM,
                    "",
                    20,
                    96,
                    205,
                    48,
                    hfont_large,
                );
                if let Some(ref btn) = btn_from {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let palette = ThemePalette::get(is_dark);
                        let (is_listening, vk_opt) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| {
                                g.as_ref().map(|s| (s.active_target == ActiveCaptureTarget::From, s.from_vk))
                            })
                            .unwrap_or((false, None));

                        // 0. Erase background of the button with dialog background to eliminate white corners
                        let br_bg = CreateSolidBrush(palette.bg_window);
                        FillRect(dis.hdc, &dis.rc_item, br_bg);
                        DeleteObject(br_bg);

                        let bg_color = if is_listening {
                            if is_dark { 0x003A2810 } else { 0x00F8EFEA }
                        } else {
                            palette.bg_input
                        };

                        let brush = CreateSolidBrush(bg_color);
                        let pen = CreatePen(PS_SOLID, borders::WIDTH_FOCUS, palette.accent);
                        let prev_brush = SelectObject(dis.hdc, brush);
                        let prev_pen = SelectObject(dis.hdc, pen);

                        RoundRect(
                            dis.hdc,
                            dis.rc_item.left,
                            dis.rc_item.top,
                            dis.rc_item.right,
                            dis.rc_item.bottom,
                            radii::RADIUS_MD,
                            radii::RADIUS_MD,
                        );

                        SelectObject(dis.hdc, prev_brush);
                        SelectObject(dis.hdc, prev_pen);
                        DeleteObject(brush);
                        DeleteObject(pen);

                        SetBkMode(dis.hdc, TRANSPARENT);
                        let strings = crate::language::current();
                        let (text_color, text) = if is_listening {
                            (palette.accent, strings.dlg_remap_listening)
                        } else if let Some(vk) = vk_opt {
                            let name = vk_to_name(vk);
                            let t = if name.is_empty() { "Key" } else { name };
                            (palette.accent, t)
                        } else {
                            (palette.text_secondary, strings.dlg_remap_unassigned)
                        };
                        SetTextColor(dis.hdc, text_color);

                        let font = DISPLAY_FONT.load(Ordering::SeqCst);
                        let prev_font = if font != 0 {
                            SelectObject(dis.hdc, font)
                        } else {
                            0
                        };

                        let wide_text = to_wide(&text);
                        let mut rc = dis.rc_item;
                        DrawTextW(
                            dis.hdc,
                            wide_text.as_ptr(),
                            -1,
                            &mut rc,
                            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                        );

                        if prev_font != 0 {
                            SelectObject(dis.hdc, prev_font);
                        }
                        true
                    });
                }

                // Interactive Box 2: To Key
                let btn_to = PushButton::create_owner_draw(
                    hwnd,
                    IDC_BTN_REMAP_TO,
                    "",
                    275,
                    96,
                    205,
                    48,
                    hfont_large,
                );
                if let Some(ref btn) = btn_to {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let palette = ThemePalette::get(is_dark);
                        let (is_listening, vk_opt) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| {
                                g.as_ref().map(|s| (s.active_target == ActiveCaptureTarget::To, s.to_vk))
                            })
                            .unwrap_or((false, None));

                        // 0. Erase background of the button with dialog background to eliminate white corners
                        let br_bg = CreateSolidBrush(palette.bg_window);
                        FillRect(dis.hdc, &dis.rc_item, br_bg);
                        DeleteObject(br_bg);

                        let bg_color = if is_listening {
                            if is_dark { 0x003A2810 } else { 0x00F8EFEA }
                        } else {
                            palette.bg_input
                        };

                        let brush = CreateSolidBrush(bg_color);
                        let pen = CreatePen(PS_SOLID, borders::WIDTH_FOCUS, palette.accent);
                        let prev_brush = SelectObject(dis.hdc, brush);
                        let prev_pen = SelectObject(dis.hdc, pen);

                        RoundRect(
                            dis.hdc,
                            dis.rc_item.left,
                            dis.rc_item.top,
                            dis.rc_item.right,
                            dis.rc_item.bottom,
                            radii::RADIUS_MD,
                            radii::RADIUS_MD,
                        );

                        SelectObject(dis.hdc, prev_brush);
                        SelectObject(dis.hdc, prev_pen);
                        DeleteObject(brush);
                        DeleteObject(pen);

                        SetBkMode(dis.hdc, TRANSPARENT);
                        let strings = crate::language::current();
                        let (text_color, text) = if is_listening {
                            (palette.accent, strings.dlg_remap_listening)
                        } else if let Some(vk) = vk_opt {
                            let name = vk_to_name(vk);
                            let t = if name.is_empty() { "Key" } else { name };
                            (palette.accent, t)
                        } else {
                            (palette.text_secondary, strings.dlg_remap_unassigned)
                        };
                        SetTextColor(dis.hdc, text_color);

                        let font = DISPLAY_FONT.load(Ordering::SeqCst);
                        let prev_font = if font != 0 {
                            SelectObject(dis.hdc, font)
                        } else {
                            0
                        };

                        let wide_text = to_wide(&text);
                        let mut rc = dis.rc_item;
                        DrawTextW(
                            dis.hdc,
                            wide_text.as_ptr(),
                            -1,
                            &mut rc,
                            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                        );

                        if prev_font != 0 {
                            SelectObject(dis.hdc, prev_font);
                        }
                        true
                    });
                }

                let is_dark_init = is_current_dark();
                let palette_init = ThemePalette::get(is_dark_init);

                // Row of 4 Presets
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_REMAP_PRESET_CAPS_CTRL,
                    "Caps Lock ➔ Ctrl",
                    20,
                    208,
                    110,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_REMAP_PRESET_CAPS_ESC,
                    "Caps Lock ➔ Escape",
                    138,
                    208,
                    122,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_REMAP_PRESET_INS_DEL,
                    "Insert ➔ Delete",
                    268,
                    208,
                    104,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_REMAP_PRESET_F1_ESC,
                    "F1 ➔ Escape",
                    380,
                    208,
                    100,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                // Footer Buttons
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_REMAP_CANCEL,
                    strings.dlg_remap_btn_cancel,
                    254,
                    280,
                    76,
                    32,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_REMAP_APPLY,
                    strings.dlg_remap_btn_apply,
                    340,
                    280,
                    140,
                    32,
                    0,
                    ButtonVariant::Primary,
                    Some(palette_init.bg_window),
                );

                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut st) = *guard
                {
                    if let Some(ref b) = btn_from {
                        st.hwnd_from_box = b.hwnd();
                    }
                    if let Some(ref b) = btn_to {
                        st.hwnd_to_box = b.hwnd();
                    }
                }

                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let is_dark = is_current_dark();
                    let palette = ThemePalette::get(is_dark);
                    let strings = crate::language::current();
                    let hfont_normal = get_app_font_normal();
                    let hfont_bold = get_app_font_bold();

                    // Window background
                    let brush_bg = CreateSolidBrush(palette.bg_window);
                    let mut rc_client: RECT = std::mem::zeroed();
                    GetClientRect(hwnd, &mut rc_client);
                    FillRect(hdc, &rc_client, brush_bg);
                    DeleteObject(brush_bg);

                    // Callout tip box
                    let brush_card = CreateSolidBrush(palette.bg_card);
                    let pen_card = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
                    let prev_brush = SelectObject(hdc, brush_card);
                    let prev_pen = SelectObject(hdc, pen_card);
                    RoundRect(hdc, 20, 14, 480, 58, radii::RADIUS_MD, radii::RADIUS_MD);
                    SelectObject(hdc, prev_brush);
                    SelectObject(hdc, prev_pen);
                    DeleteObject(brush_card);
                    DeleteObject(pen_card);

                    // Text inside callout tip box
                    let mut rc_tip_text = RECT {
                        left: 30,
                        top: 16,
                        right: 470,
                        bottom: 56,
                    };
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, palette.text_secondary);
                    let prev_font = SelectObject(hdc, hfont_normal);
                    let tip_w = to_wide(strings.dlg_remap_prompt);
                    DrawTextW(
                        hdc,
                        tip_w.as_ptr(),
                        -1,
                        &mut rc_tip_text,
                        DT_WORDBREAK | DT_VCENTER,
                    );

                    // Label Left: KHI BẤM PHÍM:
                    let mut rc_from_lbl = RECT {
                        left: 20,
                        top: 74,
                        right: 225,
                        bottom: 92,
                    };
                    SelectObject(hdc, hfont_bold);
                    SetTextColor(hdc, palette.text_secondary);
                    let from_lbl_w = to_wide(strings.dlg_remap_from);
                    DrawTextW(
                        hdc,
                        from_lbl_w.as_ptr(),
                        -1,
                        &mut rc_from_lbl,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Label Right: SẼ HOẠT ĐỘNG NHƯ:
                    let mut rc_to_lbl = RECT {
                        left: 275,
                        top: 74,
                        right: 480,
                        bottom: 92,
                    };
                    let to_lbl_w = to_wide(strings.dlg_remap_to);
                    DrawTextW(
                        hdc,
                        to_lbl_w.as_ptr(),
                        -1,
                        &mut rc_to_lbl,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Center Arrow: ➜
                    let mut rc_arrow = RECT {
                        left: 225,
                        top: 96,
                        right: 275,
                        bottom: 144,
                    };
                    let font_large = DISPLAY_FONT.load(Ordering::SeqCst);
                    let prev_large = if font_large != 0 {
                        SelectObject(hdc, font_large)
                    } else {
                        0
                    };
                    SetTextColor(hdc, palette.accent);
                    let arrow_w = to_wide("➜");
                    DrawTextW(
                        hdc,
                        arrow_w.as_ptr(),
                        -1,
                        &mut rc_arrow,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                    if prev_large != 0 {
                        SelectObject(hdc, prev_large);
                    }

                    // Subtitle Hint Left: Click vào ô để bấm phím
                    let mut rc_from_hint = RECT {
                        left: 20,
                        top: 150,
                        right: 225,
                        bottom: 168,
                    };
                    SelectObject(hdc, hfont_normal);
                    SetTextColor(hdc, palette.text_secondary);
                    let hint_w = to_wide(strings.dlg_remap_click_hint);
                    DrawTextW(
                        hdc,
                        hint_w.as_ptr(),
                        -1,
                        &mut rc_from_hint,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Subtitle Hint Right: Click vào ô để bấm phím
                    let mut rc_to_hint = RECT {
                        left: 275,
                        top: 150,
                        right: 480,
                        bottom: 168,
                    };
                    DrawTextW(
                        hdc,
                        hint_w.as_ptr(),
                        -1,
                        &mut rc_to_hint,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Label: Các cặp đổi phím phổ biến:
                    let mut rc_presets = RECT {
                        left: 20,
                        top: 182,
                        right: 480,
                        bottom: 202,
                    };
                    SelectObject(hdc, hfont_bold);
                    SetTextColor(hdc, palette.text_secondary);
                    let presets_w = to_wide(strings.dlg_remap_presets_title);
                    DrawTextW(
                        hdc,
                        presets_w.as_ptr(),
                        -1,
                        &mut rc_presets,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Footer bar
                    let rc_footer = RECT {
                        left: 0,
                        top: 264,
                        right: 500,
                        bottom: 336,
                    };
                    let brush_footer = CreateSolidBrush(palette.bg_sidebar);
                    FillRect(hdc, &rc_footer, brush_footer);
                    DeleteObject(brush_footer);

                    // Separator line
                    let pen_sep = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border_separator);
                    let prev_pen = SelectObject(hdc, pen_sep);
                    MoveToEx(hdc, 0, 264, null_mut());
                    LineTo(hdc, 500, 264);
                    SelectObject(hdc, prev_pen);
                    DeleteObject(pen_sep);

                    // Target context text
                    let default_root = format!("🌐 {}", strings.banner_root_title);
                    let target_display = STATE
                        .lock()
                        .ok()
                        .and_then(|g| g.as_ref().map(|s| s.target_display.clone()))
                        .unwrap_or(default_root);
                    let target_str = format!("{} {}", strings.hotkey_apply_prefix, target_display);
                    let mut rc_target = RECT {
                        left: 20,
                        top: 280,
                        right: 250,
                        bottom: 312,
                    };
                    SelectObject(hdc, hfont_normal);
                    SetTextColor(hdc, palette.text_secondary);
                    let target_w = to_wide(&target_str);
                    DrawTextW(
                        hdc,
                        target_w.as_ptr(),
                        -1,
                        &mut rc_target,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    SelectObject(hdc, prev_font);
                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_DRAWITEM => {
                let dis = &*(lparam as *const DRAWITEMSTRUCT);
                if dis.hwnd_item != 0 {
                    let res = SendMessageW(dis.hwnd_item, OCM_DRAWITEM, wparam, lparam);
                    if res != 0 {
                        return res;
                    }
                }
                0
            }
            WM_REMAP_KEY_CAPTURED => {
                let vk = wparam as u32;
                let mut h_from = 0;
                let mut h_to = 0;

                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut st) = *guard
                {
                    h_from = st.hwnd_from_box;
                    h_to = st.hwnd_to_box;

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
                    InvalidateRect(h_from, null_mut(), 1);
                }
                if h_to != 0 {
                    InvalidateRect(h_to, null_mut(), 1);
                }
                0
            }
            WM_COMMAND => {
                let control_id = (wparam & 0xFFFF) as u32;
                match control_id {
                    IDC_BTN_REMAP_FROM => {
                        let mut h_from = 0;
                        let mut h_to = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            if st.active_target == ActiveCaptureTarget::From {
                                st.active_target = ActiveCaptureTarget::None;
                                stop_single_key_capture();
                            } else {
                                st.active_target = ActiveCaptureTarget::From;
                                start_single_key_capture(hwnd);
                            }
                            h_from = st.hwnd_from_box;
                            h_to = st.hwnd_to_box;
                        }
                        if h_from != 0 {
                            InvalidateRect(h_from, null_mut(), 1);
                        }
                        if h_to != 0 {
                            InvalidateRect(h_to, null_mut(), 1);
                        }
                    }
                    IDC_BTN_REMAP_TO => {
                        let mut h_from = 0;
                        let mut h_to = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            if st.active_target == ActiveCaptureTarget::To {
                                st.active_target = ActiveCaptureTarget::None;
                                stop_single_key_capture();
                            } else {
                                st.active_target = ActiveCaptureTarget::To;
                                start_single_key_capture(hwnd);
                            }
                            h_from = st.hwnd_from_box;
                            h_to = st.hwnd_to_box;
                        }
                        if h_from != 0 {
                            InvalidateRect(h_from, null_mut(), 1);
                        }
                        if h_to != 0 {
                            InvalidateRect(h_to, null_mut(), 1);
                        }
                    }
                    IDC_BTN_REMAP_PRESET_CAPS_CTRL => {
                        apply_preset(0x14, 0x11);
                    }
                    IDC_BTN_REMAP_PRESET_CAPS_ESC => {
                        apply_preset(0x14, 0x1B);
                    }
                    IDC_BTN_REMAP_PRESET_INS_DEL => {
                        apply_preset(0x2D, 0x2E);
                    }
                    IDC_BTN_REMAP_PRESET_F1_ESC => {
                        apply_preset(0x70, 0x1B);
                    }
                    IDC_BTN_REMAP_CANCEL => {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    IDC_BTN_REMAP_APPLY => {
                        let (orig_from, from_opt, to_opt) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| g.as_ref().map(|s| (s.original_from_vk, s.from_vk, s.to_vk)))
                            .unwrap_or((None, None, None));

                        if let (Some(from), Some(to)) = (from_opt, to_opt) {
                            if from != 0 && to != 0 && from != to {
                                let mut changed = false;
                            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                                && let Some(ref mut engine) = *guard
                            {
                                let sel_id = engine.profile_manager.selected_id.clone();
                                if sel_id == "global" {
                                    if let Some(orig) = orig_from {
                                        engine.profile_manager.global_profile.app_remaps.retain(|r| r.from_vk != orig);
                                    }
                                    let rule = KeyRemapRule::new(from, to);
                                    engine.profile_manager.global_profile.app_remaps.retain(|r| r.from_vk != from);
                                    engine.profile_manager.global_profile.app_remaps.push(rule);
                                    changed = true;
                                } else {
                                    let is_global = engine.profile_manager.global_profile.app_remaps.iter().any(|r| r.from_vk == from);
                                    if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                                        if let Some(orig) = orig_from {
                                            child.overridden_remaps.remove(&orig);
                                            child.app_remaps.retain(|r| r.from_vk != orig);
                                        }
                                        if is_global {
                                            child.muted_remaps.retain(|&v| v != from);
                                            child.overridden_remaps.insert(from, to);
                                        } else {
                                            child.app_remaps.retain(|r| r.from_vk != from);
                                            child.app_remaps.push(KeyRemapRule::new(from, to));
                                        }
                                        changed = true;
                                    }
                                }

                                if changed {
                                    crate::engine::config_store::save_config_all_debounced(
                                        engine.config(),
                                        &engine.macro_table,
                                        &engine.profile_manager,
                                    );
                                }
                            }

                            if changed {
                                if let Ok(guard) = ENGINE_INSTANCE.lock()
                                    && let Some(ref engine) = *guard
                                    && let Ok(ui_guard) = crate::ui::UI_MANAGER.try_lock()
                                    && let Some(ref ui) = *ui_guard
                                {
                                    let sel_id = engine.profile_manager.selected_id.clone();
                                    let remap_items = engine.profile_manager.resolve_remap_items_for_ui(&sel_id);
                                    ui.controls.tab_remap.populate_remaps(&remap_items);
                                }
                            }

                            PostMessageW(hwnd, WM_CLOSE, 0, 0);
                        }
                    }
                    }
                    _ => {}
                }
                0
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let vk = wparam as u32;
                if vk == 0x1B {
                    // Escape closes dialog when not actively capturing
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                }
                0
            }
            WM_ERASEBKGND => 1,
            WM_CLOSE => {
                stop_single_key_capture();
                let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                    SetForegroundWindow(parent);
                }
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                stop_single_key_capture();
                let parent = DIALOG_PARENT_HWND.swap(0, Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                }
                let font = DISPLAY_FONT.swap(0, Ordering::SeqCst);
                if font != 0 {
                    DeleteObject(font);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn apply_preset(from: u32, to: u32) {
    stop_single_key_capture();
    let mut h_from = 0;
    let mut h_to = 0;
    if let Ok(mut guard) = STATE.lock()
        && let Some(ref mut st) = *guard
    {
        st.from_vk = Some(from);
        st.to_vk = Some(to);
        st.active_target = ActiveCaptureTarget::None;
        h_from = st.hwnd_from_box;
        h_to = st.hwnd_to_box;
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
