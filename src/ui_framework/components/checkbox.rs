//! Native Win32 CheckBox Component
//! Custom-subclassed for perfect Dark and Light theme integration,
//! crisp primary text color, custom Fluent 18x18 checkbox, and exact vertical centering.

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::tokens::{borders, radii};
use crate::ui_framework::win32::*;
use std::ptr::null_mut;

const CHECKBOX_SUBCLASS_ID: usize = 0x408;

const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_TABSTOP: u32 = 0x00010000;
const BS_AUTOCHECKBOX: u32 = 0x00000003;
const BS_3STATE: u32 = 0x00000005;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BM_SETSTYLE: u32 = 0x00F4;
const BST_CHECKED: usize = 0x0001;
const BST_UNCHECKED: usize = 0x0000;
const BST_INDETERMINATE: usize = 0x0002;
const WM_SETFONT: u32 = 0x0030;
const CBM_SET_TRI: u32 = 0x0400 + 0x60;
const CBM_GET_INHERITED: u32 = 0x0400 + 0x61;
const CBM_SET_ALIGN_RIGHT: u32 = 0x0400 + 0x62;

struct CheckBoxData {
    is_hover: bool,
    bg_color: Option<u32>,
    tri: bool,
    inherited: bool,
    inherit_note: &'static str,
    align_right: bool,
}

unsafe extern "system" fn checkbox_subclass_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    uid_subclass: usize,
    ref_data: usize,
) -> isize {
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_NCDESTROY: u32 = 0x0082;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_KEYUP: u32 = 0x0101;
    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_MOUSELEAVE: u32 = 0x02A3;
    const DT_LEFT: u32 = 0x00000000;
    const DT_VCENTER: u32 = 0x00000004;
    const DT_SINGLELINE: u32 = 0x00000020;
    const PS_SOLID: i32 = 0;
    const TRANSPARENT: i32 = 1;

    if msg == WM_NCDESTROY {
        if ref_data != 0 {
            unsafe {
                RemoveWindowSubclass(hwnd, checkbox_subclass_proc, uid_subclass);
                drop(Box::from_raw(ref_data as *mut CheckBoxData));
            }
        }
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }

    if msg == WM_ERASEBKGND {
        return 1;
    }

    let data_ptr = ref_data as *mut CheckBoxData;

    if msg == CBM_SET_TRI && !data_ptr.is_null() {
        let data = unsafe { &mut *data_ptr };
        data.tri = wparam != 0;
        data.inherited = wparam == 2;
        data.inherit_note = if lparam != 0 {
            unsafe { *(lparam as *const &'static str) }
        } else {
            ""
        };
        let style = if data.tri { BS_3STATE } else { BS_AUTOCHECKBOX };
        unsafe {
            DefSubclassProc(hwnd, BM_SETSTYLE, style as usize, 0);
            InvalidateRect(hwnd, null_mut(), 0);
        }
        return 1;
    }
    if msg == CBM_GET_INHERITED {
        return (!data_ptr.is_null() && unsafe { (*data_ptr).tri && (*data_ptr).inherited })
            as isize;
    }
    if msg == CBM_SET_ALIGN_RIGHT && !data_ptr.is_null() {
        unsafe {
            (*data_ptr).align_right = wparam != 0;
            InvalidateRect(hwnd, null_mut(), 0);
        }
        return 1;
    }
    if msg == OCM_COMMAND
        && (wparam >> 16) & 0xFFFF == 0
        && !data_ptr.is_null()
        && unsafe { (*data_ptr).tri }
    {
        let inherited = unsafe { (*data_ptr).inherited };
        let cur = unsafe { SendMessageW(hwnd, BM_GETCHECK, 0, 0) } as usize;
        let as_state = |v: bool| if v { BST_CHECKED } else { BST_UNCHECKED };
        let next = if cur == BST_INDETERMINATE {
            as_state(!inherited)
        } else if cur == as_state(inherited) {
            BST_INDETERMINATE
        } else {
            as_state(inherited)
        };
        unsafe { SendMessageW(hwnd, BM_SETCHECK, next, 0) };
        return 0;
    }

    if msg == WM_MOUSEMOVE && !data_ptr.is_null() {
        let data = unsafe { &mut *data_ptr };
        if !data.is_hover {
            data.is_hover = true;
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
    } else if msg == WM_MOUSELEAVE && !data_ptr.is_null() {
        let data = unsafe { &mut *data_ptr };
        if data.is_hover {
            data.is_hover = false;
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
    } else if msg == WM_LBUTTONUP || msg == WM_KEYUP || msg == BM_SETCHECK {
        let res = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }
        return res;
    }

    if msg == WM_PAINT {
        unsafe {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            if hdc != 0 {
                let mut rc = RECT::default();
                GetClientRect(hwnd, &mut rc);

                let is_dark = is_current_dark();
                let palette = ThemePalette::get(is_dark);
                let is_hover = if !data_ptr.is_null() {
                    (*data_ptr).is_hover
                } else {
                    false
                };

                let bg_color = if !data_ptr.is_null() {
                    (*data_ptr).bg_color.unwrap_or(palette.bg_card)
                } else {
                    palette.bg_card
                };
                let bg_brush = CreateSolidBrush(bg_color);
                FillRect(hdc, &rc, bg_brush);
                DeleteObject(bg_brush);

                let check_state = SendMessageW(hwnd, BM_GETCHECK, 0, 0) as usize;
                let is_checked = check_state == BST_CHECKED;
                let is_disabled = IsWindowEnabled(hwnd) == 0;
                let inherited_on = !data_ptr.is_null() && (*data_ptr).inherited;
                let inherit_note = (check_state == BST_INDETERMINATE && !data_ptr.is_null())
                    .then(|| (*data_ptr).inherit_note)
                    .filter(|s| !s.is_empty());

                let hfont = SendMessageW(hwnd, WM_GETFONT, 0, 0);
                let old_font = if hfont != 0 {
                    SelectObject(hdc, hfont)
                } else {
                    0
                };
                let mut text_buf = [0u16; 256];
                let text_len = GetWindowTextW(hwnd, text_buf.as_mut_ptr(), 256);
                let measure = |s: &[u16]| {
                    let mut sz = SIZE::default();
                    GetTextExtentPoint32W(hdc, s.as_ptr(), s.len() as i32, &mut sz);
                    sz.cx
                };
                let label_w = measure(&text_buf[..text_len.max(0) as usize]);
                let space_w = measure(&[b' ' as u16]);
                let note_w: Vec<u16> = inherit_note
                    .map(|n| n.encode_utf16().collect())
                    .unwrap_or_default();
                let note_px = if note_w.is_empty() {
                    0
                } else {
                    space_w + measure(&note_w)
                };
                let gap = 5;
                let box_size = 13;
                let content_w = 2 + box_size + gap + label_w + note_px;
                let offset = if !data_ptr.is_null() && (*data_ptr).align_right {
                    (rc.right - content_w).max(0)
                } else {
                    0
                };

                let h = rc.bottom - rc.top;
                let box_top = (h - box_size) / 2;
                let box_rc = RECT {
                    left: offset + 2,
                    top: box_top,
                    right: offset + 2 + box_size,
                    bottom: box_top + box_size,
                };

                let (glyph_bg, glyph_border) = if is_checked {
                    if is_hover && !is_disabled {
                        (palette.accent_hover, palette.accent_hover)
                    } else if is_disabled {
                        (palette.text_secondary, palette.text_secondary)
                    } else {
                        (palette.accent, palette.accent)
                    }
                } else if is_disabled {
                    (palette.bg_input, palette.border)
                } else if is_hover {
                    (palette.bg_input, palette.border_hover)
                } else {
                    (palette.bg_input, palette.border)
                };

                let br = CreateSolidBrush(glyph_bg);
                let pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, glyph_border);
                let ob = SelectObject(hdc, br);
                let op = SelectObject(hdc, pen);

                RoundRect(
                    hdc,
                    box_rc.left,
                    box_rc.top,
                    box_rc.right,
                    box_rc.bottom,
                    radii::RADIUS_TINY,
                    radii::RADIUS_TINY,
                );

                SelectObject(hdc, ob);
                SelectObject(hdc, op);
                DeleteObject(br);
                DeleteObject(pen);

                if is_checked {
                    let pen_check = CreatePen(PS_SOLID, borders::WIDTH_FOCUS, palette.text_white);
                    let op_c = SelectObject(hdc, pen_check);
                    MoveToEx(hdc, box_rc.left + 2, box_rc.top + 6, null_mut());
                    LineTo(hdc, box_rc.left + 5, box_rc.top + 9);
                    LineTo(hdc, box_rc.left + 10, box_rc.top + 3);
                    SelectObject(hdc, op_c);
                    DeleteObject(pen_check);
                } else if check_state == BST_INDETERMINATE {
                    let dash_color = if inherited_on && !is_disabled {
                        palette.accent
                    } else {
                        palette.text_secondary
                    };
                    let pen_dash = CreatePen(PS_SOLID, borders::WIDTH_FOCUS, dash_color);
                    let op_d = SelectObject(hdc, pen_dash);
                    MoveToEx(hdc, box_rc.left + 3, box_rc.top + 6, null_mut());
                    LineTo(hdc, box_rc.left + 10, box_rc.top + 6);
                    SelectObject(hdc, op_d);
                    DeleteObject(pen_dash);
                }

                if text_len > 0 {
                    let mut text_rc = RECT {
                        left: box_rc.right + gap,
                        top: 0,
                        right: rc.right,
                        bottom: rc.bottom,
                    };
                    SetBkMode(hdc, TRANSPARENT);
                    let text_color = if is_disabled {
                        palette.text_secondary
                    } else {
                        palette.text_primary
                    };
                    SetTextColor(hdc, text_color);

                    DrawTextW(
                        hdc,
                        text_buf.as_ptr(),
                        text_len,
                        &mut text_rc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );

                    if !note_w.is_empty() {
                        let mut note_rc = RECT {
                            left: text_rc.left + label_w + space_w,
                            ..text_rc
                        };
                        SetTextColor(hdc, palette.text_secondary);
                        DrawTextW(
                            hdc,
                            note_w.as_ptr(),
                            note_w.len() as i32,
                            &mut note_rc,
                            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                        );
                    }
                }
                if old_font != 0 {
                    SelectObject(hdc, old_font);
                }

                EndPaint(hwnd, &ps);
            }
        }
        return 0;
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

use crate::ui_framework::I18nSource;
use std::marker::PhantomData;

/// Marker type for a standard 2-state (`bool`) checkbox.
#[derive(Debug, Clone, Copy)]
pub struct TwoState;

/// Marker type for a 3-state (`Option<bool>`) inheritable checkbox used in profile tabs.
#[derive(Debug, Clone, Copy)]
pub struct Inheritable<L: I18nSource = ()>(PhantomData<L>);

pub struct CheckBox<M = TwoState> {
    hwnd: isize,
    id: u32,
    _mode: PhantomData<M>,
}

impl<M> CheckBox<M> {
    #[allow(clippy::too_many_arguments)]
    fn create_raw(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
        bg_color: Option<u32>,
    ) -> Option<Self> {
        let id = super::resolve_id(id);
        let class_name = to_wide("BUTTON");
        let window_name = to_wide(text);

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX,
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
            if hfont != 0 {
                unsafe {
                    SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
                }
            }
            let data = Box::into_raw(Box::new(CheckBoxData {
                is_hover: false,
                bg_color,
                tri: false,
                inherited: false,
                inherit_note: "",
                align_right: false,
            }));
            unsafe {
                SetWindowSubclass(
                    hwnd,
                    checkbox_subclass_proc,
                    CHECKBOX_SUBCLASS_ID,
                    data as usize,
                );
            }
            Some(Self {
                hwnd,
                id,
                _mode: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns `true` when this checkbox was toggled in `WM_COMMAND` (`BN_CLICKED = 0`).
    #[inline]
    pub fn is_toggled(&self, ctrl_id: u32, notify_code: u32) -> bool {
        ctrl_id == self.id && notify_code == 0
    }

    pub fn is_checked(&self) -> bool {
        let res = unsafe { SendMessageW(self.hwnd, BM_GETCHECK, 0, 0) } as usize;
        res == BST_CHECKED
            || (res == BST_INDETERMINATE
                && unsafe { SendMessageW(self.hwnd, CBM_GET_INHERITED, 0, 0) } != 0)
    }

    pub fn set_align_right(&self, right: bool) {
        unsafe { SendMessageW(self.hwnd, CBM_SET_ALIGN_RIGHT, right as usize, 0) };
    }

    pub fn set_text(&self, text: &str) {
        let wide = to_wide(text);
        unsafe {
            SetWindowTextW(self.hwnd, wide.as_ptr());
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        unsafe {
            EnableWindow(self.hwnd, enabled as i32);
            InvalidateRect(self.hwnd, null_mut(), 1);
        }
    }

    pub fn set_visible(&self, visible: bool) {
        unsafe {
            ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }
}

impl CheckBox<TwoState> {
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
        Self::create_raw(parent, id, text, x, y, width, height, hfont, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_ext(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
        bg_color: Option<u32>,
    ) -> Option<Self> {
        Self::create_raw(parent, id, text, x, y, width, height, hfont, bg_color)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_with_state(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
        checked: bool,
    ) -> Option<Self> {
        let cb = Self::create(parent, id, text, x, y, width, height, hfont)?;
        if checked {
            cb.set_checked(true);
        }
        Some(cb)
    }

    pub fn set_checked(&self, checked: bool) {
        let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
        unsafe {
            SendMessageW(self.hwnd, CBM_SET_TRI, 0, 0);
            SendMessageW(self.hwnd, BM_SETCHECK, state, 0);
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    /// Transitions a 2-state checkbox into a 3-state inheritable checkbox.
    #[inline]
    pub fn into_inheritable<L: I18nSource>(self) -> CheckBox<Inheritable<L>> {
        CheckBox {
            hwnd: self.hwnd,
            id: self.id,
            _mode: PhantomData,
        }
    }
}

impl<L: I18nSource> CheckBox<Inheritable<L>> {
    #[allow(clippy::too_many_arguments)]
    pub fn create_inheritable(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        Self::create_raw(parent, id, text, x, y, width, height, hfont, None)
    }

    pub fn set_tri_state_with_note(
        &self,
        value: Option<bool>,
        inherited: bool,
        note: &'static str,
    ) {
        let state = match value {
            None => BST_INDETERMINATE,
            Some(true) => BST_CHECKED,
            Some(false) => BST_UNCHECKED,
        };
        unsafe {
            SendMessageW(
                self.hwnd,
                CBM_SET_TRI,
                1 + inherited as usize,
                &note as *const &'static str as isize,
            );
            SendMessageW(self.hwnd, BM_SETCHECK, state, 0);
        }
    }

    pub fn set_tri_state(&self, value: Option<bool>, inherited: bool) {
        let note = L::current().inherit_note(inherited);
        self.set_tri_state_with_note(value, inherited, note);
    }

    /// Sets 2-state when `is_root` is true, or 3-state (`set_tri_state`) for child profiles.
    pub fn set_inherited(&self, is_root: bool, child_value: Option<bool>, global_value: bool) {
        if is_root {
            let state = if global_value {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            };
            unsafe {
                SendMessageW(self.hwnd, CBM_SET_TRI, 0, 0);
                SendMessageW(self.hwnd, BM_SETCHECK, state, 0);
                InvalidateRect(self.hwnd, null_mut(), 0);
            }
        } else {
            self.set_tri_state(child_value, global_value);
        }
    }

    pub fn tri_state(&self) -> Option<bool> {
        match unsafe { SendMessageW(self.hwnd, BM_GETCHECK, 0, 0) } as usize {
            BST_INDETERMINATE => None,
            s => Some(s == BST_CHECKED),
        }
    }
}
