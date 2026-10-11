//! Modal Confirmation Dialog for Kora UI
//! Generic, reusable Yes/No dialog with customizable title, message, button labels, and danger styling.

use crate::ui::colors::ThemePalette;
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::tokens::borders;
use crate::ui::tokens::id::{IDC_CONFIRM_BTN_NO, IDC_CONFIRM_BTN_YES};
use crate::ui::win32::*;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;

/// Options for configuring a confirmation dialog
#[derive(Debug, Clone)]
pub struct ConfirmDialogOptions {
    pub title: String,
    pub message: String,
    pub yes_text: String,
    pub no_text: String,
    pub is_danger: bool,
}

impl ConfirmDialogOptions {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let strings = crate::language::current();
        Self {
            title: title.into(),
            message: message.into(),
            yes_text: strings.btn_yes.to_string(),
            no_text: strings.btn_no.to_string(),
            is_danger: false,
        }
    }

    pub fn danger(mut self, is_danger: bool) -> Self {
        self.is_danger = is_danger;
        self
    }

    pub fn yes_text(mut self, text: impl Into<String>) -> Self {
        self.yes_text = text.into();
        self
    }

    pub fn no_text(mut self, text: impl Into<String>) -> Self {
        self.no_text = text.into();
        self
    }
}

impl Default for ConfirmDialogOptions {
    fn default() -> Self {
        let strings = crate::language::current();
        Self {
            title: strings.confirm_delete_title.to_string(),
            message: String::new(),
            yes_text: strings.btn_yes.to_string(),
            no_text: strings.btn_no.to_string(),
            is_danger: false,
        }
    }
}

type ConfirmCallback = Box<dyn FnOnce() + Send + 'static>;

static DIALOG_PARENT_HWND: AtomicIsize = AtomicIsize::new(0);
static CURRENT_OPTIONS: Mutex<Option<ConfirmDialogOptions>> = Mutex::new(None);
static CURRENT_CALLBACK: Mutex<Option<ConfirmCallback>> = Mutex::new(None);

const CONFIRM_BTN_SUBCLASS_ID: usize = 9911;

/// Displays a modal confirmation dialog centered over the parent window.
/// The `on_confirm` closure is executed when the user clicks Yes or presses Enter.
pub fn show_confirm_dialog<F>(parent: isize, options: ConfirmDialogOptions, on_confirm: F)
where
    F: FnOnce() + Send + 'static,
{
    let class_name = to_wide("KoraConfirmDialogClass");

    unsafe {
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfn_wnd_proc = Some(confirm_dialog_wnd_proc);
        wc.h_instance = 0;
        wc.h_cursor = LoadCursorW(0, IDC_ARROW);
        wc.lpsz_class_name = class_name.as_ptr();
        RegisterClassExW(&wc);

        DIALOG_PARENT_HWND.store(parent, Ordering::SeqCst);
        let title = options.title.clone();
        if let Ok(mut g) = CURRENT_OPTIONS.lock() {
            *g = Some(options);
        }
        if let Ok(mut g) = CURRENT_CALLBACK.lock() {
            *g = Some(Box::new(on_confirm));
        }

        let width = 430;
        let height = 195;

        let mut rc: RECT = std::mem::zeroed();
        GetWindowRect(parent, &mut rc);
        let x = rc.left + ((rc.right - rc.left) - width) / 2;
        let y = rc.top + ((rc.bottom - rc.top) - height) / 2;

        EnableWindow(parent, 0);

        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME,
            class_name.as_ptr(),
            to_wide(&title).as_ptr(),
            WS_CAPTION | WS_SYSMENU | WS_VISIBLE, // Pure modal dialog: no minimize, no maximize, only Close [X]
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

unsafe extern "system" fn confirm_button_subclass_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    _uid: usize,
    ref_data: usize,
) -> isize {
    const WM_KEYDOWN: u32 = 0x0100;
    if msg == WM_KEYDOWN {
        let vk = wparam as u32;
        let dlg_hwnd = ref_data as isize;
        if vk == 0x1B {
            // VK_ESCAPE cancels dialog
            unsafe {
                PostMessageW(dlg_hwnd, 0x0010 /* WM_CLOSE */, 0, 0);
            }
            return 0;
        }
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub unsafe extern "system" fn confirm_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_COMMAND: u32 = 0x0111;
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_CLOSE: u32 = 0x0010;
    const WM_DRAWITEM: u32 = 0x002B;
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
                let (is_danger, yes_text, no_text) = {
                    if let Ok(guard) = CURRENT_OPTIONS.lock()
                        && let Some(ref opts) = *guard
                    {
                        (opts.is_danger, opts.yes_text.clone(), opts.no_text.clone())
                    } else {
                        let strings = crate::language::current();
                        (false, strings.btn_yes.to_string(), strings.btn_no.to_string())
                    }
                };

                let font_bold = get_app_font_bold();
                let font_normal = get_app_font_normal();
                let is_dark = is_current_dark();
                let palette = ThemePalette::get(is_dark);

                let yes_variant = if is_danger {
                    ButtonVariant::Danger
                } else {
                    ButtonVariant::Primary
                };

                // Cancel / No Button
                let btn_no = PushButton::create_with_variant(
                    hwnd,
                    IDC_CONFIRM_BTN_NO,
                    &no_text,
                    210,
                    110,
                    92,
                    32,
                    font_normal,
                    ButtonVariant::Secondary,
                    Some(palette.bg_sidebar),
                );

                // Confirm / Yes Button
                let btn_yes = PushButton::create_with_variant(
                    hwnd,
                    IDC_CONFIRM_BTN_YES,
                    &yes_text,
                    310,
                    110,
                    94,
                    32,
                    font_bold,
                    yes_variant,
                    Some(palette.bg_sidebar),
                );

                // Subclass buttons to catch VK_ESCAPE while focused
                if let Some(ref b) = btn_no {
                    SetWindowSubclass(
                        b.hwnd(),
                        confirm_button_subclass_proc,
                        CONFIRM_BTN_SUBCLASS_ID,
                        hwnd as usize,
                    );
                }
                if let Some(ref b) = btn_yes {
                    SetWindowSubclass(
                        b.hwnd(),
                        confirm_button_subclass_proc,
                        CONFIRM_BTN_SUBCLASS_ID,
                        hwnd as usize,
                    );
                }

                // Default focus: focus No on danger actions to prevent accidental deletion
                if is_danger {
                    if let Some(ref b) = btn_no {
                        SetFocus(b.hwnd());
                    }
                } else if let Some(ref b) = btn_yes {
                    SetFocus(b.hwnd());
                }

                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let is_dark = is_current_dark();
                    let palette = ThemePalette::get(is_dark);

                    let (is_danger, message) = {
                        if let Ok(guard) = CURRENT_OPTIONS.lock()
                            && let Some(ref opts) = *guard
                        {
                            (opts.is_danger, opts.message.clone())
                        } else {
                            (false, String::new())
                        }
                    };

                    let mut rc_client = RECT::default();
                    GetClientRect(hwnd, &mut rc_client);

                    // 1. Window main body background
                    let brush_bg = CreateSolidBrush(palette.bg_window);
                    FillRect(hdc, &rc_client, brush_bg);
                    DeleteObject(brush_bg);

                    // 2. Footer background and separator line
                    let footer_y = 100;
                    let rc_footer = RECT {
                        left: 0,
                        top: footer_y,
                        right: rc_client.right,
                        bottom: rc_client.bottom,
                    };
                    let brush_footer = CreateSolidBrush(palette.bg_sidebar);
                    FillRect(hdc, &rc_footer, brush_footer);
                    DeleteObject(brush_footer);

                    let pen_sep = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
                    let prev_pen = SelectObject(hdc, pen_sep);
                    MoveToEx(hdc, 0, footer_y, null_mut());
                    LineTo(hdc, rc_client.right, footer_y);
                    SelectObject(hdc, prev_pen);
                    DeleteObject(pen_sep);

                    // 3. Vibrant Colored Icon Badge (Warning or Info)
                    let badge_size = 40;
                    let badge_x = 20;
                    let badge_y = 24;
                    let rc_badge = RECT {
                        left: badge_x,
                        top: badge_y,
                        right: badge_x + badge_size,
                        bottom: badge_y + badge_size,
                    };

                    let badge_color = if is_danger {
                        palette.danger_border
                    } else {
                        palette.accent
                    };

                    let brush_badge = CreateSolidBrush(badge_color);
                    let pen_badge = CreatePen(PS_SOLID, 1, badge_color);
                    let prev_brush = SelectObject(hdc, brush_badge);
                    let prev_pen = SelectObject(hdc, pen_badge);
                    Ellipse(
                        hdc,
                        rc_badge.left,
                        rc_badge.top,
                        rc_badge.right,
                        rc_badge.bottom,
                    );
                    SelectObject(hdc, prev_brush);
                    SelectObject(hdc, prev_pen);
                    DeleteObject(brush_badge);
                    DeleteObject(pen_badge);

                    // Centered glyph: bold pure white "!" for danger, "i" for info
                    let mut rc_glyph = rc_badge;
                    let glyph = if is_danger { "!" } else { "i" };
                    let glyph_w = to_wide(glyph);
                    let font_glyph = create_app_font(if is_danger { 18 } else { 16 }, FW_BOLD);
                    let prev_font = SelectObject(hdc, font_glyph);
                    SetBkMode(hdc, 1);
                    SetTextColor(hdc, palette.text_white);
                    DrawTextW(
                        hdc,
                        glyph_w.as_ptr(),
                        -1,
                        &mut rc_glyph,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                    SelectObject(hdc, prev_font);
                    DeleteObject(font_glyph);

                    // 4. Message text (word wrapped)
                    let mut rc_msg = RECT {
                        left: 74,
                        top: 22,
                        right: rc_client.right - 20,
                        bottom: footer_y - 8,
                    };
                    let msg_w = to_wide(&message);
                    let font_msg = get_app_font_normal();
                    let prev_font = SelectObject(hdc, font_msg);
                    SetBkMode(hdc, 1);
                    SetTextColor(hdc, palette.text_primary);
                    DrawTextW(
                        hdc,
                        msg_w.as_ptr(),
                        -1,
                        &mut rc_msg,
                        DT_LEFT | DT_WORDBREAK | DT_NOPREFIX,
                    );
                    SelectObject(hdc, prev_font);

                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_ERASEBKGND => 1,
            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as u32;
                if id == IDC_CONFIRM_BTN_YES {
                    if let Ok(mut guard) = CURRENT_CALLBACK.lock()
                        && let Some(cb) = guard.take()
                    {
                        cb();
                    }
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                } else if id == IDC_CONFIRM_BTN_NO {
                    if let Ok(mut guard) = CURRENT_CALLBACK.lock() {
                        guard.take();
                    }
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                }
                0
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let vk = wparam as u32;
                if vk == 0x1B {
                    // Escape cancels
                    if let Ok(mut guard) = CURRENT_CALLBACK.lock() {
                        guard.take();
                    }
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    return 0;
                } else if vk == 0x0D {
                    // Enter confirms
                    if let Ok(mut guard) = CURRENT_CALLBACK.lock()
                        && let Some(cb) = guard.take()
                    {
                        cb();
                    }
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    return 0;
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
            WM_CLOSE => {
                let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                    SetForegroundWindow(parent);
                }
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                let parent = DIALOG_PARENT_HWND.swap(0, Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                }
                if let Ok(mut g) = CURRENT_OPTIONS.lock() {
                    *g = None;
                }
                if let Ok(mut g) = CURRENT_CALLBACK.lock() {
                    *g = None;
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
