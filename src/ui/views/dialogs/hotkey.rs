//! Modal Dialog for capturing and customizing typing mode switch hotkey
//! Pixel-perfect native Win32 GDI implementation matching modern mockup.

use crate::engine::config::Hotkey;
use crate::engine::config_store;
use crate::platform::win32::{
    get_captured_display_text, get_captured_hotkey, set_captured_hotkey, start_hotkey_capture,
    stop_hotkey_capture, ENGINE_INSTANCE, WM_HOTKEY_CAPTURED, WM_HOTKEY_CANCELLED,
};
use crate::ui::colors::ThemePalette;
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::win32::*;
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::tokens::id::*;
use crate::ui::tokens::{borders, radii};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;

static DIALOG_PARENT_HWND: AtomicIsize = AtomicIsize::new(0);
static DISPLAY_FONT: AtomicIsize = AtomicIsize::new(0);

#[derive(Clone)]
struct HotkeyDialogState {
    is_listening: bool,
    is_inherit: bool,
    is_root: bool,
    target_profile_id: String,
    target_display: String,
    global_hotkey: Hotkey,
    current_hotkey: Option<Hotkey>,
    display_text: String,
    hwnd_display_box: isize,
}

static STATE: Mutex<Option<HotkeyDialogState>> = Mutex::new(None);

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraHotkeyDialogClass";

/// Displays the modal Hotkey Capture Dialog centered over the parent control panel
pub fn show_hotkey_dialog(parent: isize) {
    register_class_once(&REGISTERED, CLASS_NAME, hotkey_dialog_wnd_proc, 0);

    unsafe {
        let class_name = to_wide(CLASS_NAME);
        DIALOG_PARENT_HWND.store(parent, Ordering::SeqCst);

        let strings = crate::language::current();
        let default_root_name = format!("🌐 {}", strings.banner_root_title);
        let (is_root, target_profile_id, target_display, current_hotkey, global_hotkey) = {
            if let Ok(guard) = ENGINE_INSTANCE.lock()
                && let Some(ref engine) = *guard
            {
                let global_hk = engine.config().switch_key;
                let sel_id = engine.profile_manager.selected_id.clone();
                if sel_id == "global" {
                    (true, "global".to_string(), default_root_name.clone(), Some(global_hk), global_hk)
                } else if let Some(child) = engine.profile_manager.get_profile(&sel_id) {
                    let name = format!("{} {}", child.icon, child.name);
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

        if let Ok(mut guard) = STATE.lock() {
            *guard = Some(HotkeyDialogState {
                is_listening: false,
                is_inherit,
                is_root,
                target_profile_id,
                target_display,
                global_hotkey,
                current_hotkey,
                display_text: display_text.clone(),
                hwnd_display_box: 0,
            });
        }

        set_captured_hotkey(current_hotkey, &display_text);

        // Client dimensions: 440 x 336
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: 440,
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

        let title = to_wide(strings.hotkey_dialog_title);

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
pub unsafe extern "system" fn hotkey_dialog_wnd_proc(
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

                // Interactive Hotkey Display Box
                let btn_display = PushButton::create_owner_draw(
                    hwnd,
                    IDC_LABEL_HOTKEY_DISPLAY,
                    "",
                    20,
                    86,
                    400,
                    46,
                    hfont_large,
                );
                if let Some(ref btn) = btn_display {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let palette = ThemePalette::get(is_dark);
                        let (is_listening, text) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| {
                                g.as_ref().map(|s| (s.is_listening, s.display_text.clone()))
                            })
                            .unwrap_or((false, "Ctrl + Shift".to_string()));

                        let bg_color = if is_listening {
                            if is_dark {
                                0x003A2810 // Subtle dark cyan/blue tint
                            } else {
                                0x00F8EFEA // Light soft blue tint
                            }
                        } else {
                            palette.bg_input
                        };

                        let br_bg = CreateSolidBrush(palette.bg_window);
                        FillRect(dis.hdc, &dis.rc_item, br_bg);
                        DeleteObject(br_bg);

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
                        SetTextColor(dis.hdc, palette.accent);

                        let font = DISPLAY_FONT.load(Ordering::SeqCst);
                        let prev_font = if font != 0 {
                            SelectObject(dis.hdc, font)
                        } else {
                            0
                        };

                        let mut rc = dis.rc_item;
                        let wide_text = to_wide(&text);
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

                // Row 1 presets
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_HOTKEY_PRESET_CTRL_SHIFT,
                    strings.hotkey_preset_ctrl_shift,
                    20,
                    182,
                    175,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    None,
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_HOTKEY_PRESET_ALT_Z,
                    strings.hotkey_preset_alt_z,
                    203,
                    182,
                    155,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    None,
                );

                // Row 2 presets
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_HOTKEY_PRESET_WIN_SPACE,
                    strings.hotkey_preset_win_space,
                    20,
                    216,
                    100,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    None,
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_HOTKEY_PRESET_SHIFT_SPACE,
                    strings.hotkey_preset_shift_space,
                    128,
                    216,
                    105,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    None,
                );

                let btn_inherit = PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_HOTKEY_PRESET_INHERIT,
                    strings.typing_inherit_global,
                    241,
                    216,
                    179,
                    28,
                    0,
                    ButtonVariant::Secondary,
                    None,
                );

                // Footer Buttons
                let is_dark_init = is_current_dark();
                let palette_init = ThemePalette::get(is_dark_init);
                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_HOTKEY_CANCEL,
                    strings.btn_cancel,
                    236,
                    280,
                    84,
                    32,
                    0,
                    ButtonVariant::Secondary,
                    Some(palette_init.bg_sidebar),
                );

                PushButton::create_with_variant(
                    hwnd,
                    IDC_BTN_HOTKEY_SAVE,
                    strings.hotkey_btn_save,
                    328,
                    280,
                    92,
                    32,
                    0,
                    ButtonVariant::Primary,
                    Some(palette_init.bg_sidebar),
                );

                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut st) = *guard
                {
                    if let Some(ref b) = btn_display {
                        st.hwnd_display_box = b.hwnd();
                    }
                    if let Some(ref b) = btn_inherit
                        && st.is_root
                    {
                        ShowWindow(b.hwnd(), SW_HIDE);
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
                    RoundRect(hdc, 20, 12, 420, 58, radii::RADIUS_MD, radii::RADIUS_MD);
                    SelectObject(hdc, prev_brush);
                    SelectObject(hdc, prev_pen);
                    DeleteObject(brush_card);
                    DeleteObject(pen_card);

                    // Text inside callout tip box
                    let mut rc_tip_text = RECT {
                        left: 30,
                        top: 14,
                        right: 410,
                        bottom: 56,
                    };
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, palette.text_secondary);
                    let prev_font = SelectObject(hdc, hfont_normal);
                    let tip_w = to_wide(strings.hotkey_dialog_prompt);
                    DrawTextW(
                        hdc,
                        tip_w.as_ptr(),
                        -1,
                        &mut rc_tip_text,
                        DT_WORDBREAK | DT_VCENTER,
                    );

                    // Label: TỔ HỢP PHÍM ĐƯỢC CHỌN:
                    let mut rc_sel = RECT {
                        left: 20,
                        top: 66,
                        right: 420,
                        bottom: 84,
                    };
                    SelectObject(hdc, hfont_bold);
                    SetTextColor(hdc, palette.text_secondary);
                    let sel_w = to_wide(strings.hotkey_selected_title);
                    DrawTextW(
                        hdc,
                        sel_w.as_ptr(),
                        -1,
                        &mut rc_sel,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Label: Click vào khung phía trên...
                    let mut rc_hint = RECT {
                        left: 20,
                        top: 136,
                        right: 420,
                        bottom: 154,
                    };
                    SelectObject(hdc, hfont_normal);
                    SetTextColor(hdc, palette.text_secondary);
                    let hint_w = to_wide(strings.hotkey_click_hint);
                    DrawTextW(
                        hdc,
                        hint_w.as_ptr(),
                        -1,
                        &mut rc_hint,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    // Label: Các tổ hợp phím thông dụng:
                    let mut rc_presets = RECT {
                        left: 20,
                        top: 160,
                        right: 420,
                        bottom: 178,
                    };
                    SelectObject(hdc, hfont_bold);
                    SetTextColor(hdc, palette.text_secondary);
                    let presets_w = to_wide(strings.hotkey_presets_title);
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
                        right: 440,
                        bottom: 336,
                    };
                    let brush_footer = CreateSolidBrush(palette.bg_sidebar);
                    FillRect(hdc, &rc_footer, brush_footer);
                    DeleteObject(brush_footer);

                    // Separator line
                    let pen_sep = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border_separator);
                    let prev_pen = SelectObject(hdc, pen_sep);
                    MoveToEx(hdc, 0, 264, null_mut());
                    LineTo(hdc, 440, 264);
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
                        right: 230,
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
            WM_HOTKEY_CAPTURED => {
                let text = get_captured_display_text();
                let hotkey = get_captured_hotkey();
                let mut disp_hwnd = 0;

                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut st) = *guard
                {
                    disp_hwnd = st.hwnd_display_box;
                    if !text.is_empty() {
                        st.display_text = text;
                        if hotkey.is_some() {
                            st.current_hotkey = hotkey;
                            st.is_listening = false;
                            stop_hotkey_capture();
                        }
                    }
                }

                if disp_hwnd != 0 {
                    InvalidateRect(disp_hwnd, null_mut(), 1);
                }
                0
            }
            WM_HOTKEY_CANCELLED => {
                let mut disp_hwnd = 0;
                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut st) = *guard
                {
                    st.is_listening = false;
                    st.display_text = if st.is_inherit {
                        format!("{} ({})", crate::language::current().typing_inherit_prefix, st.global_hotkey.display_text())
                    } else {
                        st.current_hotkey.map(|h| h.display_text()).unwrap_or_default()
                    };
                    disp_hwnd = st.hwnd_display_box;
                }
                stop_hotkey_capture();
                if disp_hwnd != 0 {
                    InvalidateRect(disp_hwnd, null_mut(), 1);
                }
                0
            }
            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    let mut disp_hwnd = 0;
                    if let Ok(mut guard) = STATE.lock()
                        && let Some(ref mut st) = *guard
                        && st.is_listening
                    {
                        st.is_listening = false;
                        st.display_text = if st.is_inherit {
                            format!("{} ({})", crate::language::current().typing_inherit_prefix, st.global_hotkey.display_text())
                        } else {
                            st.current_hotkey.map(|h| h.display_text()).unwrap_or_default()
                        };
                        disp_hwnd = st.hwnd_display_box;
                    }
                    stop_hotkey_capture();
                    if disp_hwnd != 0 {
                        InvalidateRect(disp_hwnd, null_mut(), 1);
                    }
                }
                0
            }
            WM_COMMAND => {
                let control_id = (wparam & 0xFFFF) as u32;
                match control_id {
                    IDC_LABEL_HOTKEY_DISPLAY => {
                        let strings = crate::language::current();
                        let mut disp_hwnd = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            st.is_listening = true;
                            st.is_inherit = false;
                            st.display_text = strings.hotkey_listening.to_string();
                            disp_hwnd = st.hwnd_display_box;
                        }
                        start_hotkey_capture(hwnd);
                        if disp_hwnd != 0 {
                            InvalidateRect(disp_hwnd, null_mut(), 1);
                        }
                    }
                    IDC_BTN_HOTKEY_PRESET_CTRL_SHIFT => {
                        let mut disp_hwnd = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            st.is_listening = false;
                            st.is_inherit = false;
                            st.current_hotkey = Some(Hotkey::new(true, true, false, false, 0));
                            st.display_text = "Ctrl + Shift".to_string();
                            disp_hwnd = st.hwnd_display_box;
                            set_captured_hotkey(st.current_hotkey, &st.display_text);
                        }
                        stop_hotkey_capture();
                        if disp_hwnd != 0 {
                            InvalidateRect(disp_hwnd, null_mut(), 1);
                        }
                    }
                    IDC_BTN_HOTKEY_PRESET_ALT_Z => {
                        let mut disp_hwnd = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            st.is_listening = false;
                            st.is_inherit = false;
                            st.current_hotkey = Some(Hotkey::new(false, false, true, false, 0x5A));
                            st.display_text = "Alt + Z".to_string();
                            disp_hwnd = st.hwnd_display_box;
                            set_captured_hotkey(st.current_hotkey, &st.display_text);
                        }
                        stop_hotkey_capture();
                        if disp_hwnd != 0 {
                            InvalidateRect(disp_hwnd, null_mut(), 1);
                        }
                    }
                    IDC_BTN_HOTKEY_PRESET_WIN_SPACE => {
                        let mut disp_hwnd = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            st.is_listening = false;
                            st.is_inherit = false;
                            st.current_hotkey = Some(Hotkey::new(false, false, false, true, 0x20));
                            st.display_text = "Win + Space".to_string();
                            disp_hwnd = st.hwnd_display_box;
                            set_captured_hotkey(st.current_hotkey, &st.display_text);
                        }
                        stop_hotkey_capture();
                        if disp_hwnd != 0 {
                            InvalidateRect(disp_hwnd, null_mut(), 1);
                        }
                    }
                    IDC_BTN_HOTKEY_PRESET_SHIFT_SPACE => {
                        let mut disp_hwnd = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            st.is_listening = false;
                            st.is_inherit = false;
                            st.current_hotkey = Some(Hotkey::new(false, true, false, false, 0x20));
                            st.display_text = "Shift + Space".to_string();
                            disp_hwnd = st.hwnd_display_box;
                            set_captured_hotkey(st.current_hotkey, &st.display_text);
                        }
                        stop_hotkey_capture();
                        if disp_hwnd != 0 {
                            InvalidateRect(disp_hwnd, null_mut(), 1);
                        }
                    }
                    IDC_BTN_HOTKEY_PRESET_INHERIT => {
                        let strings = crate::language::current();
                        let mut disp_hwnd = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut st) = *guard
                        {
                            st.is_listening = false;
                            st.is_inherit = true;
                            st.current_hotkey = None;
                            st.display_text = format!("{} ({})", strings.typing_inherit_prefix, st.global_hotkey.display_text());
                            disp_hwnd = st.hwnd_display_box;
                            set_captured_hotkey(None, &st.display_text);
                        }
                        stop_hotkey_capture();
                        if disp_hwnd != 0 {
                            InvalidateRect(disp_hwnd, null_mut(), 1);
                        }
                    }
                    IDC_BTN_HOTKEY_SAVE => {
                        stop_hotkey_capture();
                        let state_opt = STATE.lock().ok().and_then(|g| g.clone());
                        if let Some(mut st) = state_opt {
                            if st.is_listening {
                                st.is_listening = false;
                            }
                            let strings = crate::language::current();
                            if st.display_text.is_empty()
                                || st.display_text == strings.hotkey_listening
                            {
                                return 0;
                            }

                            if !st.is_inherit {
                                let valid_hk = st.current_hotkey.or_else(get_captured_hotkey);
                                match valid_hk {
                                    Some(hk) if hk.is_valid() => {}
                                    _ => return 0,
                                }
                            }

                            if let Ok(mut guard) = ENGINE_INSTANCE.lock()
                                && let Some(ref mut engine) = *guard
                            {
                                if st.is_inherit {
                                    if !st.is_root
                                        && let Some(child) = engine
                                            .profile_manager
                                            .get_profile_mut(&st.target_profile_id)
                                    {
                                        child.switch_key = None;
                                    }
                                } else {
                                    let final_hotkey = st
                                        .current_hotkey
                                        .or_else(get_captured_hotkey)
                                        .unwrap_or_default();

                                    if st.is_root {
                                        engine.config_mut().switch_key = final_hotkey;
                                        engine.config_mut().switch_with_ctrl_shift =
                                            final_hotkey.is_ctrl_shift();
                                    } else if let Some(child) = engine
                                        .profile_manager
                                        .get_profile_mut(&st.target_profile_id)
                                    {
                                        child.switch_key = Some(final_hotkey);
                                    }
                                }

                                config_store::save_config_all_debounced(
                                    engine.config(),
                                    &engine.macro_table,
                                    &engine.profile_manager,
                                );

                                if let Some(c) = crate::ui::controls() {
                                    if st.is_root {
                                        c.tab_typing.load_config(engine.config());
                                    } else if let Some(child) = engine
                                        .profile_manager
                                        .get_profile(&st.target_profile_id)
                                    {
                                        c.tab_typing.load_profile(child, engine.config());
                                    }
                                }
                            }
                        }

                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    IDC_BTN_HOTKEY_CANCEL => {
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
                stop_hotkey_capture();
                let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                    SetForegroundWindow(parent);
                }
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                stop_hotkey_capture();
                let font = DISPLAY_FONT.swap(0, Ordering::SeqCst);
                if font != 0 {
                    DeleteObject(font);
                }
                let parent = DIALOG_PARENT_HWND.swap(0, Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                    SetForegroundWindow(parent);
                }
                if let Ok(mut guard) = STATE.lock() {
                    *guard = None;
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
