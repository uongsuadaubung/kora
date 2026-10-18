//! Native Win32 PushButton Component

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

const BS_PUSHBUTTON: u32 = 0x00000000;
const BS_OWNERDRAW: u32 = 0x0000000B;

const BUTTON_SUBCLASS_ID: usize = 0x501;

pub type ButtonCustomDrawer = Arc<dyn Fn(&DRAWITEMSTRUCT, bool) -> bool + Send + Sync>;
pub type ButtonClickCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct ButtonData {
    pub variant: ButtonVariant,
    pub is_hover: bool,
    pub bg_erase: Option<u32>,
    pub corner_radius: i32,
    pub align_left: bool,
    pub custom_drawer: Option<ButtonCustomDrawer>,
    pub on_click: Option<ButtonClickCallback>,
}

impl Default for ButtonData {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::Secondary,
            is_hover: false,
            bg_erase: None,
            corner_radius: 6,
            align_left: false,
            custom_drawer: None,
            on_click: None,
        }
    }
}

unsafe extern "system" fn button_subclass_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    uid_subclass: usize,
    ref_data: usize,
) -> isize {
    const WM_NCDESTROY: u32 = 0x0082;
    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_MOUSELEAVE: u32 = 0x02A3;
    const WM_SETCURSOR: u32 = 0x0020;

    let state_ptr = ref_data as *const Mutex<ButtonData>;
    if state_ptr.is_null() {
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }
    let state_arc = unsafe { &*state_ptr };

    if msg == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, button_subclass_proc, uid_subclass);
            drop(Arc::from_raw(state_ptr));
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    if msg == WM_SETCURSOR {
        if unsafe { IsWindowEnabled(hwnd) } != 0 {
            unsafe {
                SetCursor(LoadCursorW(0, IDC_HAND));
            }
            return 1;
        }
    } else if msg == WM_MOUSEMOVE {
        let need_redraw = if let Ok(mut s) = state_arc.lock() {
            if !s.is_hover {
                s.is_hover = true;
                true
            } else {
                false
            }
        } else {
            false
        };
        if need_redraw {
            let mut tme = TRACKMOUSEEVENT {
                cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dw_flags: TME_LEAVE,
                hwnd_track: hwnd,
                dw_hover_time: 0,
            };
            unsafe {
                TrackMouseEvent(&mut tme);
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
    } else if msg == WM_MOUSELEAVE {
        let need_redraw = if let Ok(mut s) = state_arc.lock() {
            if s.is_hover {
                s.is_hover = false;
                true
            } else {
                false
            }
        } else {
            false
        };
        if need_redraw {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
    } else if msg == OCM_COMMAND {
        let notify_code = ((wparam >> 16) & 0xFFFF) as u32;
        if notify_code == 0 {
            let cb = state_arc.lock().ok().and_then(|s| s.on_click.clone());
            if let Some(cb) = cb {
                cb();
                return 1;
            }
        }
    } else if msg == OCM_DRAWITEM {
        let dis = unsafe { &*(lparam as *const DRAWITEMSTRUCT) };
        let (custom_drawer, variant, is_hover, bg_erase, corner_radius, align_left) =
            if let Ok(s) = state_arc.lock() {
                (
                    s.custom_drawer.clone(),
                    s.variant,
                    s.is_hover,
                    s.bg_erase,
                    s.corner_radius,
                    s.align_left,
                )
            } else {
                (None, ButtonVariant::Secondary, false, None, 6, false)
            };

        let is_dark = is_current_dark();
        if let Some(ref drawer) = custom_drawer
            && drawer(dis, is_dark)
        {
            return 1;
        }

        let palette = ThemePalette::get(is_dark);
        let bg_erase = bg_erase.unwrap_or(palette.bg_card);

        let mut buf = [0u16; 256];
        let len = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        let text = if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        };

        let is_pressed = (dis.item_state & 0x0001 /* ODS_SELECTED */) != 0;
        let is_disabled = (dis.item_state & 0x0004 /* ODS_DISABLED */) != 0;

        PushButton::render_ext_state(
            dis.hdc,
            &dis.rc_item,
            &text,
            variant,
            is_pressed,
            is_hover,
            is_disabled,
            is_dark,
            bg_erase,
            corner_radius,
            align_left,
        );
        return 1;
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub struct PushButton {
    hwnd: isize,
    id: u32,
    state: Arc<Mutex<ButtonData>>,
}

impl PushButton {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let id = super::resolve_id(id);
        let class_name = to_wide("BUTTON");
        let window_name = to_wide(text);

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
                x,
                y,
                width,
                height,
                parent,
                id as isize,
                0,
                null_mut(),
            )
        };

        if hwnd != 0 {
            let state = Arc::new(Mutex::new(ButtonData::default()));
            let state_raw = Arc::into_raw(state.clone()) as usize;
            unsafe {
                SetWindowSubclass(hwnd, button_subclass_proc, BUTTON_SUBCLASS_ID, state_raw);
            }
            if hfont != 0 {
                unsafe {
                    SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
                }
            }
            Some(Self { hwnd, id, state })
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_owner_draw(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        Self::create_with_variant(
            parent,
            id,
            text,
            x,
            y,
            width,
            height,
            hfont,
            ButtonVariant::Secondary,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_with_variant(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
        variant: ButtonVariant,
        bg_erase: Option<u32>,
    ) -> Option<Self> {
        let id = super::resolve_id(id);
        let class_name = to_wide("BUTTON");
        let window_name = to_wide(text);

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW,
                x,
                y,
                width,
                height,
                parent,
                id as isize,
                0,
                null_mut(),
            )
        };

        if hwnd != 0 {
            let state = Arc::new(Mutex::new(ButtonData {
                variant,
                is_hover: false,
                bg_erase,
                corner_radius: 6,
                align_left: false,
                custom_drawer: None,
                on_click: None,
            }));
            let state_raw = Arc::into_raw(state.clone()) as usize;
            unsafe {
                let empty = to_wide(" ");
                SetWindowTheme(hwnd, empty.as_ptr(), empty.as_ptr());
                SetWindowSubclass(hwnd, button_subclass_proc, BUTTON_SUBCLASS_ID, state_raw);
            }
            if hfont != 0 {
                unsafe {
                    SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
                }
            }
            Some(Self { hwnd, id, state })
        } else {
            None
        }
    }

    pub fn set_corner_radius(&self, corner_radius: i32) {
        if let Ok(mut s) = self.state.lock() {
            s.corner_radius = corner_radius;
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn set_align_left(&self, align_left: bool) {
        if let Ok(mut s) = self.state.lock() {
            s.align_left = align_left;
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn set_variant(&self, variant: ButtonVariant) {
        if let Ok(mut s) = self.state.lock() {
            s.variant = variant;
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn set_custom_drawer<F>(&self, drawer: F)
    where
        F: Fn(&DRAWITEMSTRUCT, bool) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut s) = self.state.lock() {
            s.custom_drawer = Some(Arc::new(drawer));
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    /// Builder: attaches a click callback invoked automatically on `OCM_COMMAND` (`BN_CLICKED`).
    pub fn on_click<F>(self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.set_on_click(callback);
        self
    }

    pub fn set_on_click<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut s) = self.state.lock() {
            s.on_click = Some(Arc::new(callback));
        }
    }

    /// Returns `true` when this button was clicked in `WM_COMMAND` (`BN_CLICKED = 0`).
    #[inline]
    pub fn is_clicked(&self, ctrl_id: u32, notify_code: u32) -> bool {
        ctrl_id == self.id && notify_code == 0
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn set_text(&self, text: &str) {
        let wide = to_wide(text);
        unsafe {
            SetWindowTextW(self.hwnd, wide.as_ptr());
        }
    }

    pub fn get_text(&self) -> String {
        let len = unsafe { GetWindowTextLengthW(self.hwnd) };
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let copied = unsafe { GetWindowTextW(self.hwnd, buf.as_mut_ptr(), len + 1) };
        if copied > 0 {
            String::from_utf16_lossy(&buf[..copied as usize])
        } else {
            String::new()
        }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { IsWindowEnabled(self.hwnd) != 0 }
    }

    pub fn set_enabled(&self, enabled: bool) {
        unsafe {
            EnableWindow(self.hwnd, enabled as i32);
            InvalidateRect(self.hwnd, null_mut(), 1);
        }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd) != 0 }
    }

    pub fn set_visible(&self, visible: bool) {
        unsafe {
            ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }

    pub fn is_mouse_hover(hwnd: isize) -> bool {
        let mut pt = POINT::default();
        unsafe {
            if GetCursorPos(&mut pt) != 0 {
                WindowFromPoint(pt) == hwnd
            } else {
                false
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_ext_state(
        hdc: isize,
        rc: &RECT,
        text: &str,
        variant: ButtonVariant,
        is_pressed: bool,
        is_hover: bool,
        is_disabled: bool,
        is_dark: bool,
        bg_erase: u32,
        radius: i32,
        align_left: bool,
    ) {
        unsafe {
            let palette = ThemePalette::get(is_dark);

            if bg_erase != 0 {
                let bg_br = CreateSolidBrush(bg_erase);
                FillRect(hdc, rc, bg_br);
                DeleteObject(bg_br);
            }

            let (btn_color, border_color, text_color, font) = match variant {
                ButtonVariant::Primary => {
                    if is_disabled {
                        (
                            palette.tab_active_bg,
                            palette.border,
                            palette.text_secondary,
                            get_app_font_bold(),
                        )
                    } else if is_pressed || is_hover {
                        (
                            palette.accent_hover,
                            palette.accent_hover,
                            palette.text_white,
                            get_app_font_bold(),
                        )
                    } else {
                        (
                            palette.accent,
                            palette.accent,
                            palette.text_white,
                            get_app_font_bold(),
                        )
                    }
                }
                ButtonVariant::Secondary => {
                    if is_disabled {
                        (
                            palette.tab_active_bg,
                            palette.border,
                            palette.text_secondary,
                            get_app_font_normal(),
                        )
                    } else if is_pressed {
                        (
                            palette.tab_hover_bg,
                            palette.border,
                            palette.text_primary,
                            get_app_font_normal(),
                        )
                    } else if is_hover {
                        (
                            palette.tab_hover_bg,
                            palette.border_hover,
                            palette.text_primary,
                            get_app_font_normal(),
                        )
                    } else {
                        (
                            palette.tab_active_bg,
                            palette.border,
                            palette.text_primary,
                            get_app_font_normal(),
                        )
                    }
                }
                ButtonVariant::Danger => {
                    if is_disabled {
                        (
                            palette.tab_active_bg,
                            palette.border,
                            palette.text_secondary,
                            get_app_font_normal(),
                        )
                    } else if is_pressed {
                        (
                            palette.danger_bg,
                            palette.danger_border,
                            palette.danger_text,
                            get_app_font_normal(),
                        )
                    } else if is_hover {
                        (
                            palette.danger_bg_hover,
                            palette.danger_border,
                            palette.danger_text_hover,
                            get_app_font_normal(),
                        )
                    } else {
                        (
                            palette.tab_active_bg,
                            palette.border,
                            palette.danger_text,
                            get_app_font_normal(),
                        )
                    }
                }
                ButtonVariant::DangerSolid => {
                    let (bg, border) = if is_pressed {
                        (0x002420DC, 0x002420DC)
                    } else if is_hover {
                        (0x004A50F8, 0x004A50F8)
                    } else {
                        (0x004444EF, 0x004444EF)
                    };
                    (bg, border, palette.text_white, get_app_font_bold())
                }
                ButtonVariant::Ghost => {
                    if is_pressed {
                        (
                            palette.tab_active_bg,
                            palette.border,
                            palette.text_primary,
                            get_app_font_normal(),
                        )
                    } else if is_hover {
                        (
                            palette.tab_hover_bg,
                            palette.border,
                            palette.text_primary,
                            get_app_font_normal(),
                        )
                    } else {
                        (
                            bg_erase,
                            bg_erase,
                            palette.text_secondary,
                            get_app_font_normal(),
                        )
                    }
                }
            };

            round_rect(hdc, rc, radius, btn_color, Some(border_color));

            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, text_color);
            SelectObject(hdc, font);

            let mut text_rc = *rc;
            let flags = if align_left {
                text_rc.left += 10;
                text_rc.right -= 6;
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX
            } else {
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX
            };

            DrawTextW(hdc, to_wide(text).as_ptr(), -1, &mut text_rc, flags);
        }
    }
}

/// Visual style variants for Fluent Buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    DangerSolid,
    Ghost,
}

