//! Modal Confirmation Dialog for Kora UI
//! Generic, reusable Yes/No dialog with customizable title, message, button labels, and danger styling.

use super::{attach_state, close_modal, detach_state, open_modal, reflect_dialog_messages, state, ModalSpec};
use crate::ui::colors::ThemePalette;
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::win32::*;
use std::sync::Once;

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

/// Per-window state (owned by the dialog through `GWLP_USERDATA`).
struct ConfirmState {
    parent: isize,
    options: ConfirmDialogOptions,
    /// UTF-16 message, converted once instead of on every paint.
    message_w: Vec<u16>,
    callback: Option<ConfirmCallback>,
    btn_no: Option<PushButton>,
    btn_yes: Option<PushButton>,
}

const CLASS_NAME: &str = "KoraConfirmDialogClass";
static CLASS_REGISTERED: Once = Once::new();

const DIALOG_W: i32 = 430;
const DIALOG_H: i32 = 195;
const FOOTER_Y: i32 = 100;

/// Displays a modal confirmation dialog centered over the parent window.
/// The `on_confirm` closure is executed when the user clicks Yes or presses Enter.
pub fn show_confirm_dialog<F>(parent: isize, options: ConfirmDialogOptions, on_confirm: F)
where
    F: FnOnce() + Send + 'static,
{
    register_class_once(&CLASS_REGISTERED, CLASS_NAME, confirm_dialog_wnd_proc, 0);
    let title = options.title.clone();
    let st = Box::new(ConfirmState {
        parent,
        message_w: to_wide(&options.message),
        options,
        callback: Some(Box::new(on_confirm)),
        btn_no: None,
        btn_yes: None,
    });
    open_modal(
        parent,
        &ModalSpec {
            class_name: CLASS_NAME,
            title: &title,
            width: DIALOG_W,
            height: DIALOG_H,
            ex_style: WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME,
            // Pure modal dialog: no minimize, no maximize, only Close [X]
            style: WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
        },
        st,
    );
}

/// Displays a danger-styled confirmation dialog (`btn_yes` and `btn_no` automatically localized).
pub fn show_danger_confirm<F>(
    parent: isize,
    title: impl Into<String>,
    message: impl Into<String>,
    on_confirm: F,
) where
    F: FnOnce() + Send + 'static,
{
    show_confirm_dialog(
        parent,
        ConfirmDialogOptions::new(title, message).danger(true),
        on_confirm,
    );
}

/// Displays a standard delete confirmation dialog (`strings.confirm_delete_title`, danger styling).
pub fn show_delete_confirm<F>(parent: isize, message: impl Into<String>, on_confirm: F)
where
    F: FnOnce() + Send + 'static,
{
    let strings = crate::language::current();
    show_danger_confirm(parent, strings.confirm_delete_title, message, on_confirm);
}

/// Runs the confirm callback (taken out of the state first, so a callback that
/// opens another dialog cannot observe this one) and closes the dialog.
unsafe fn confirm_and_close(hwnd: isize) {
    let cb = unsafe { state::<ConfirmState>(hwnd) }.and_then(|s| s.callback.take());
    if let Some(cb) = cb {
        cb();
    }
    unsafe {
        PostMessageW(hwnd, 0x0010 /* WM_CLOSE */, 0, 0);
    }
}

unsafe fn cancel_and_close(hwnd: isize) {
    unsafe {
        if let Some(s) = state::<ConfirmState>(hwnd) {
            s.callback = None;
        }
        PostMessageW(hwnd, 0x0010 /* WM_CLOSE */, 0, 0);
    }
}

fn paint(hwnd: isize, hdc: isize, st: &ConfirmState) {
    let palette = ThemePalette::get(is_current_dark());
    let is_danger = st.options.is_danger;

    let mut rc_client = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rc_client) };

    // 1. Window main body background
    fill_rect_color(hdc, &rc_client, palette.bg_window);

    // 2. Footer background and separator line
    let rc_footer = RECT { top: FOOTER_Y, ..rc_client };
    fill_rect_color(hdc, &rc_footer, palette.bg_sidebar);
    draw_line(hdc, 0, FOOTER_Y, rc_client.right, FOOTER_Y, palette.border);

    // 3. Vibrant Colored Icon Badge (Warning or Info)
    let badge_size = 40;
    let rc_badge = RECT { left: 20, top: 24, right: 20 + badge_size, bottom: 24 + badge_size };
    let badge_color = if is_danger { palette.danger_border } else { palette.accent };
    fill_ellipse(hdc, &rc_badge, badge_color);

    unsafe {
        SetBkMode(hdc, TRANSPARENT);

        // Centered glyph: bold pure white "!" for danger, "i" for info
        let mut rc_glyph = rc_badge;
        let glyph: &[u16] = if is_danger { &[b'!' as u16, 0] } else { &[b'i' as u16, 0] };
        let font_glyph = crate::ui::theme::cached_font(if is_danger { 18 } else { 16 }, FW_BOLD);
        let prev_font = SelectObject(hdc, font_glyph);
        SetTextColor(hdc, palette.text_white);
        DrawTextW(hdc, glyph.as_ptr(), -1, &mut rc_glyph, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        // 4. Message text (word wrapped)
        let mut rc_msg = RECT { left: 74, top: 22, right: rc_client.right - 20, bottom: FOOTER_Y - 8 };
        SelectObject(hdc, get_app_font_normal());
        SetTextColor(hdc, palette.text_primary);
        DrawTextW(hdc, st.message_w.as_ptr(), -1, &mut rc_msg, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
        SelectObject(hdc, prev_font);
    }
}

/// # Safety
///
/// Win32 window procedure callback called by Windows OS message dispatch.
pub unsafe extern "system" fn confirm_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_CREATE: u32 = 0x0001;
    const WM_COMMAND: u32 = 0x0111;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_CLOSE: u32 = 0x0010;
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_NCDESTROY: u32 = 0x0082;

    unsafe {
        if let Some(res) = reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                attach_state::<ConfirmState>(hwnd, lparam);
                let Some(st) = state::<ConfirmState>(hwnd) else { return -1 };
                let is_danger = st.options.is_danger;
                let palette = ThemePalette::get(is_current_dark());

                let yes_variant = if is_danger {
                    ButtonVariant::Danger
                } else {
                    ButtonVariant::Primary
                };

                // Cancel / No Button (Auto-ID = 0)
                let btn_no = PushButton::create_with_variant(
                    hwnd,
                    0,
                    &st.options.no_text,
                    210,
                    110,
                    92,
                    32,
                    get_app_font_normal(),
                    ButtonVariant::Secondary,
                    Some(palette.bg_sidebar),
                );

                // Confirm / Yes Button (Auto-ID = 0)
                let btn_yes = PushButton::create_with_variant(
                    hwnd,
                    0,
                    &st.options.yes_text,
                    310,
                    110,
                    94,
                    32,
                    get_app_font_bold(),
                    yes_variant,
                    Some(palette.bg_sidebar),
                );

                // Default focus: focus No on danger actions to prevent accidental deletion
                let focus_hwnd = if is_danger {
                    btn_no.as_ref().map_or(0, |b| b.hwnd())
                } else {
                    btn_yes.as_ref().map_or(0, |b| b.hwnd())
                };
                st.btn_no = btn_no;
                st.btn_yes = btn_yes;

                if focus_hwnd != 0 {
                    SetFocus(focus_hwnd);
                }
                0
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    if let Some(st) = state::<ConfirmState>(hwnd) {
                        match BackBuffer::new(hdc, DIALOG_W, DIALOG_H) {
                            Some(buf) => paint(hwnd, buf.hdc(), st),
                            None => paint(hwnd, hdc, st),
                        }
                    }
                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_ERASEBKGND => 1,
            WM_COMMAND => {
                let (yes_id, no_id) = state::<ConfirmState>(hwnd)
                    .map(|s| {
                        (
                            s.btn_yes.as_ref().map_or(0, |b| b.id()),
                            s.btn_no.as_ref().map_or(0, |b| b.id()),
                        )
                    })
                    .unwrap_or((0, 0));

                let mut id = (wparam & 0xFFFF) as u32;
                if id == IDOK {
                    // Enter: activates the focused button (No stays No on danger dialogs)
                    id = super::resolve_enter_command(hwnd, yes_id);
                }
                if yes_id != 0 && id == yes_id {
                    confirm_and_close(hwnd);
                } else if (no_id != 0 && id == no_id) || id == IDCANCEL {
                    cancel_and_close(hwnd);
                }
                0
            }
            WM_KEYDOWN => {
                // Only reached when the dialog itself has focus (no child focused).
                match wparam as u32 {
                    0x1B => cancel_and_close(hwnd),  // Escape cancels
                    0x0D => confirm_and_close(hwnd), // Enter confirms
                    _ => {}
                }
                0
            }
            WM_CLOSE => {
                let parent = state::<ConfirmState>(hwnd).map_or(0, |s| s.parent);
                close_modal(hwnd, parent);
                0
            }
            WM_NCDESTROY => {
                if let Some(st) = detach_state::<ConfirmState>(hwnd) {
                    // Safety net for abnormal destruction (e.g. owner destroyed first).
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
