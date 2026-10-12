//! Modal Dialog for Key Disabling (Khóa / Vô Hiệu Hóa Phím)
//! High-polish native Win32 GDI modal matching modern UI mockups.
//! Supports physical single-key capture, quick presets, and active profile assignment.

use crate::engine::config::hotkey::vk_to_name;
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

#[derive(Clone)]
struct DisableDialogState {
    is_listening: bool,
    target_vk: Option<u32>,
    target_display: String,
    hwnd_box: isize,
}

static DIALOG_PARENT_HWND: AtomicIsize = AtomicIsize::new(0);
static DISPLAY_FONT: AtomicIsize = AtomicIsize::new(0);
static STATE: Mutex<Option<DisableDialogState>> = Mutex::new(None);

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraDisableDialogClass";

/// Displays the modal Key Disabling Dialog centered over the parent control panel
pub fn show_disable_dialog(parent: isize) {
    register_class_once(&REGISTERED, CLASS_NAME, disable_dialog_wnd_proc, 0);

    unsafe {
        let class_name = to_wide(CLASS_NAME);
        DIALOG_PARENT_HWND.store(parent, Ordering::SeqCst);

        let strings = crate::language::current();
        let default_root_name = format!("🌐 {}", strings.banner_root_title);
        let target_display = {
            if let Ok(guard) = ENGINE_INSTANCE.lock()
                && let Some(ref engine) = *guard
            {
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

        if let Ok(mut guard) = STATE.lock() {
            *guard = Some(DisableDialogState {
                is_listening: false,
                target_vk: None,
                target_display,
                hwnd_box: 0,
            });
        }

        // Client dimensions: 480 x 326
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: 480,
            bottom: 326,
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

        let title = to_wide(strings.dlg_disable_title);

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
pub unsafe extern "system" fn disable_dialog_wnd_proc(
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

    unsafe {
        match msg {
            WM_CREATE => {
                let hfont_large = CreateFontW(
                    19,
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
                    to_wide("Consolas").as_ptr(),
                );
                DISPLAY_FONT.store(hfont_large, Ordering::SeqCst);

                let is_dark_init = is_current_dark();
                let palette_init = ThemePalette::get(is_dark_init);
                let strings = crate::language::current();

                // Interactive Key Capture Box
                let btn_box = PushButton::create_owner_draw(
                    hwnd,
                    IDC_BTN_DISABLE_KEY,
                    "",
                    20,
                    96,
                    440,
                    44,
                    hfont_large,
                );
                if let Some(ref btn) = btn_box {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let palette = ThemePalette::get(is_dark);
                        let (is_listening, vk_opt) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| g.as_ref().map(|s| (s.is_listening, s.target_vk)))
                            .unwrap_or((false, None));

                        // 0. Erase background of the button with dialog background to eliminate white corners
                        let br_bg = CreateSolidBrush(palette.bg_window);
                        FillRect(dis.hdc, &dis.rc_item, br_bg);
                        DeleteObject(br_bg);

                        let bg_color = if is_listening {
                            palette.danger_bg
                        } else {
                            palette.bg_input
                        };

                        let brush = CreateSolidBrush(bg_color);
                        let pen_style = if is_listening { PS_SOLID } else { PS_DASH };
                        let pen_width = if is_listening { 2 } else { 1 };
                        let pen = CreatePen(pen_style, pen_width, palette.danger_border);
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
                            (palette.danger_border, strings.dlg_remap_listening.to_string())
                        } else if let Some(vk) = vk_opt {
                            let name = vk_to_name(vk);
                            let t = if name.is_empty() { "Key".to_string() } else { name.to_string() };
                            (palette.danger_border, t)
                        } else {
                            (palette.text_secondary, strings.dlg_remap_unassigned.to_string())
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

                // Row 1 of Presets (4 buttons)
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_PRESET_LWIN,
                    "🚫 Left Windows",
                    20,
                    196,
                    98,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_PRESET_RWIN,
                    "🚫 Right Windows",
                    124,
                    196,
                    104,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_PRESET_INS,
                    "🚫 Insert (Ghi đè chữ)",
                    234,
                    196,
                    126,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_PRESET_CAPS,
                    "🚫 Caps Lock",
                    366,
                    196,
                    94,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                // Row 2 of Presets (2 buttons)
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_PRESET_F1,
                    "🚫 F1 (Trợ giúp)",
                    20,
                    230,
                    100,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_PRESET_SCROLL,
                    "🚫 Scroll Lock",
                    126,
                    230,
                    100,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_window),
                );

                // Footer Buttons
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_CANCEL,
                    strings.dlg_disable_btn_cancel,
                    264,
                    280,
                    82,
                    32,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_sidebar),
                );

                let btn_apply = PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_DISABLE_APPLY,
                    strings.dlg_disable_btn_apply,
                    354,
                    280,
                    106,
                    32,
                    0,
                    ButtonVariant::Primary,
                    Some(palette_init.bg_sidebar),
                );

                if let Some(ref btn) = btn_apply {
                    btn.set_custom_drawer(move |dis, _is_dark| {
                        let is_pressed = (dis.item_state & 0x0001) != 0;
                        let is_hover = PushButton::is_mouse_hover(dis.hwnd_item);
                        let palette = ThemePalette::get(is_current_dark());

                        // Erase background with footer background to keep clean corners
                        let br_bg = CreateSolidBrush(palette.bg_sidebar);
                        FillRect(dis.hdc, &dis.rc_item, br_bg);
                        DeleteObject(br_bg);

                        let (btn_color, border_color) = if is_pressed {
                            (0x002420DC, 0x002420DC) // Darker danger red
                        } else if is_hover {
                            (0x004A50F8, 0x004A50F8) // Brighter hover danger red
                        } else {
                            (0x004444EF, 0x004444EF) // Solid vibrant red (#EF4444)
                        };

                        let brush = CreateSolidBrush(btn_color);
                        let pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, border_color);
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
                        SetTextColor(dis.hdc, 0x00FFFFFF);
                        let font_bold = get_app_font_bold();
                        let prev_font = SelectObject(dis.hdc, font_bold);

                        let strings = crate::language::current();
                        let wide_text = to_wide(strings.dlg_disable_btn_apply);
                        let mut rc = dis.rc_item;
                        DrawTextW(
                            dis.hdc,
                            wide_text.as_ptr(),
                            -1,
                            &mut rc,
                            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                        );

                        SelectObject(dis.hdc, prev_font);
                        true
                    });
                }

                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut st) = *guard
                    && let Some(ref b) = btn_box
                {
                    st.hwnd_box = b.hwnd();
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

                    // Callout warning box
                    let brush_card = CreateSolidBrush(palette.bg_card_header);
                    let pen_card = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
                    let prev_brush = SelectObject(hdc, brush_card);
                    let prev_pen = SelectObject(hdc, pen_card);
                    RoundRect(hdc, 20, 14, 460, 62, radii::RADIUS_MD, radii::RADIUS_MD);
                    SelectObject(hdc, prev_brush);
                    SelectObject(hdc, prev_pen);
                    DeleteObject(brush_card);
                    DeleteObject(pen_card);

                    // Text inside callout warning box
                    let mut rc_tip_text = RECT {
                        left: 28,
                        top: 18,
                        right: 452,
                        bottom: 58,
                    };
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, palette.text_secondary);
                    let prev_font = SelectObject(hdc, hfont_normal);
                    let tip_w = to_wide(strings.dlg_disable_prompt);
                    DrawTextW(
                        hdc,
                        tip_w.as_ptr(),
                        -1,
                        &mut rc_tip_text,
                        DT_WORDBREAK | DT_VCENTER,
                    );

                    // Label: PHÍM CẦN KHÓA:
                    let mut rc_key_lbl = RECT {
                        left: 20,
                        top: 74,
                        right: 460,
                        bottom: 92,
                    };
                    SelectObject(hdc, hfont_bold);
                    SetTextColor(hdc, palette.text_secondary);
                    let key_lbl_w = to_wide(strings.dlg_disable_key);
                    DrawTextW(
                        hdc,
                        key_lbl_w.as_ptr(),
                        -1,
                        &mut rc_key_lbl,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Subtitle Hint: Click vào khung phía trên rồi bấm phím bạn muốn khóa
                    let mut rc_hint = RECT {
                        left: 20,
                        top: 145,
                        right: 460,
                        bottom: 163,
                    };
                    SelectObject(hdc, hfont_normal);
                    SetTextColor(hdc, palette.text_secondary);
                    let hint_w = to_wide(strings.dlg_disable_click_hint);
                    DrawTextW(
                        hdc,
                        hint_w.as_ptr(),
                        -1,
                        &mut rc_hint,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Label: Hoặc click chọn nhanh các phím hay bị bấm nhầm:
                    let mut rc_presets = RECT {
                        left: 20,
                        top: 172,
                        right: 460,
                        bottom: 190,
                    };
                    SelectObject(hdc, hfont_bold);
                    SetTextColor(hdc, palette.text_secondary);
                    let presets_w = to_wide(strings.dlg_disable_presets_title);
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
                        right: 480,
                        bottom: 326,
                    };
                    let brush_footer = CreateSolidBrush(palette.bg_sidebar);
                    FillRect(hdc, &rc_footer, brush_footer);
                    DeleteObject(brush_footer);

                    // Separator line
                    let pen_sep = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border_separator);
                    let prev_pen = SelectObject(hdc, pen_sep);
                    MoveToEx(hdc, 0, 264, null_mut());
                    LineTo(hdc, 480, 264);
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
                let mut h_box = 0;

                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut st) = *guard
                {
                    st.target_vk = Some(vk);
                    st.is_listening = false;
                    stop_single_key_capture();
                    h_box = st.hwnd_box;
                }

                if h_box != 0 {
                    InvalidateRect(h_box, null_mut(), 1);
                }
                0
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let mut h_box = 0;
                    if let Ok(mut guard) = STATE.lock()
                        && let Some(ref mut st) = *guard
                        && st.is_listening
                    {
                        st.is_listening = false;
                        h_box = st.hwnd_box;
                    }
                    stop_single_key_capture();
                    if h_box != 0 {
                        InvalidateRect(h_box, null_mut(), 1);
                    }
                }
                0
            }
            WM_COMMAND => {
                let control_id = (wparam & 0xFFFF) as u32;
                match control_id {
                    IDC_BTN_DISABLE_KEY => {
                        let mut h_box = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            if st.is_listening {
                                st.is_listening = false;
                                stop_single_key_capture();
                            } else {
                                st.is_listening = true;
                                start_single_key_capture(hwnd);
                            }
                            h_box = st.hwnd_box;
                        }
                        if h_box != 0 {
                            InvalidateRect(h_box, null_mut(), 1);
                        }
                    }
                    IDC_BTN_DISABLE_PRESET_LWIN => set_preset_vk(0x5B),
                    IDC_BTN_DISABLE_PRESET_RWIN => set_preset_vk(0x5C),
                    IDC_BTN_DISABLE_PRESET_INS => set_preset_vk(0x2D),
                    IDC_BTN_DISABLE_PRESET_CAPS => set_preset_vk(0x14),
                    IDC_BTN_DISABLE_PRESET_F1 => set_preset_vk(0x70),
                    IDC_BTN_DISABLE_PRESET_SCROLL => set_preset_vk(0x91),
                    IDC_BTN_DISABLE_CANCEL => {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    IDC_BTN_DISABLE_APPLY => {
                        stop_single_key_capture();
                        let vk_opt = STATE
                            .lock()
                            .ok()
                            .and_then(|g| g.as_ref().and_then(|s| s.target_vk));

                        if let Some(vk) = vk_opt
                            && vk != 0
                        {
                            let mut changed = false;
                            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                                && let Some(ref mut engine) = *guard
                            {
                                let sel_id = engine.profile_manager.selected_id.clone();
                                let is_global = engine.profile_manager.global_profile.app_disabled.contains(&vk);
                                if sel_id == "global" {
                                    if !is_global {
                                        engine.profile_manager.global_profile.app_disabled.push(vk);
                                        changed = true;
                                    }
                                } else if let Some(child) = engine.profile_manager.get_profile_mut(&sel_id) {
                                    if is_global {
                                        if child.unmuted_disabled.contains(&vk) {
                                            child.unmuted_disabled.retain(|&v| v != vk);
                                            changed = true;
                                        }
                                    } else if !child.app_disabled.contains(&vk) {
                                        child.app_disabled.push(vk);
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
                                crate::ui::controller::refresh_disable_list();
                            }

                            PostMessageW(hwnd, WM_CLOSE, 0, 0);
                        }
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
                    if let Ok(mut guard) = STATE.lock()
                        && let Some(ref mut st) = *guard
                        && st.is_listening
                    {
                        st.is_listening = false;
                        was_listening = true;
                        h_box = st.hwnd_box;
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

fn set_preset_vk(vk: u32) {
    stop_single_key_capture();
    let mut h_box = 0;
    if let Ok(mut guard) = STATE.lock()
        && let Some(ref mut st) = *guard
    {
        st.target_vk = Some(vk);
        st.is_listening = false;
        h_box = st.hwnd_box;
    }
    if h_box != 0 {
        unsafe {
            InvalidateRect(h_box, null_mut(), 1);
        }
    }
}
