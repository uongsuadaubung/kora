//! Native Win32 Label (Static text, Hyperlink & Auto-Sizing Pill Badge) Component

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::tokens::{borders, radii};
use crate::ui_framework::win32::*;
use std::ptr::null_mut;

const SS_LEFT: u32 = 0x00000000;
const SS_CENTER: u32 = 0x00000001;
const SS_RIGHT: u32 = 0x0002;
const SS_NOPREFIX: u32 = 0x00000080;
const SS_NOTIFY: u32 = 0x00000100;
const GWLP_ID: i32 = -12;

const LABEL_PILL_SUBCLASS_ID: usize = 0x4C01;
const PILL_PAD_X: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LabelTone {
    Primary = 0,
    Secondary = 1,
    Accent = 2,
    Link = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SurfaceRole {
    Card = 0,
    Window = 1,
    Sidebar = 2,
    Input = 3,
}

const LABEL_MAGIC_MASK: usize = 0xFF00;
const LABEL_MAGIC: usize = 0x4C00;
const LABEL_LINK_BIT: usize = 0x10;

const fn pack_style(tone: LabelTone, surface: SurfaceRole, is_link: bool) -> isize {
    (LABEL_MAGIC
        | (tone as usize & 0x03)
        | ((surface as usize & 0x03) << 2)
        | if is_link { LABEL_LINK_BIT } else { 0 }) as isize
}

fn unpack_style(raw: isize) -> Option<(LabelTone, SurfaceRole, bool)> {
    let u = raw as usize;
    if (u & LABEL_MAGIC_MASK) != LABEL_MAGIC {
        return None;
    }
    let tone = match u & 0x03 {
        1 => LabelTone::Secondary,
        2 => LabelTone::Accent,
        3 => LabelTone::Link,
        _ => LabelTone::Primary,
    };
    let surface = match (u >> 2) & 0x03 {
        1 => SurfaceRole::Window,
        2 => SurfaceRole::Sidebar,
        3 => SurfaceRole::Input,
        _ => SurfaceRole::Card,
    };
    let is_link = (u & LABEL_LINK_BIT) != 0 || tone == LabelTone::Link;
    Some((tone, surface, is_link))
}

unsafe fn fit_pill_width(hwnd: isize, min_w: i32, max_w: i32) {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        let mut buf = vec![0u16; (len.max(0) + 1) as usize];
        let copied = if len > 0 {
            GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1).max(0) as usize
        } else {
            0
        };

        let hdc = GetDC(hwnd);
        let hfont = SendMessageW(hwnd, WM_GETFONT, 0, 0);
        let old_font = if hfont != 0 {
            SelectObject(hdc, hfont)
        } else {
            0
        };
        let mut sz = SIZE::default();
        if copied > 0 {
            GetTextExtentPoint32W(hdc, buf.as_ptr(), copied as i32, &mut sz);
        }
        if old_font != 0 {
            SelectObject(hdc, old_font);
        }
        ReleaseDC(hwnd, hdc);

        let mut rc = RECT::default();
        GetClientRect(hwnd, &mut rc);
        let h = rc.bottom - rc.top;
        let new_w = (sz.cx + PILL_PAD_X).clamp(min_w, max_w.max(min_w));

        let parent = GetParent(hwnd);
        if parent != 0 {
            let mut wrc = RECT::default();
            GetWindowRect(hwnd, &mut wrc);
            let mut tl = POINT {
                x: wrc.left,
                y: wrc.top,
            };
            ScreenToClient(parent, &mut tl);
            let dirty = RECT {
                left: tl.x - 2,
                top: tl.y - 2,
                right: tl.x + max_w.max(rc.right) + 4,
                bottom: tl.y + h + 2,
            };
            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                new_w,
                h,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            InvalidateRect(parent, &dirty, 0);
        } else {
            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                new_w,
                h,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        InvalidateRect(hwnd, null_mut(), 0);
    }
}

unsafe extern "system" fn label_pill_subclass_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    uid_subclass: usize,
    ref_data: usize,
) -> isize {
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_SETTEXT: u32 = 0x000C;
    const WM_ENABLE: u32 = 0x000A;
    const WM_NCDESTROY: u32 = 0x0082;

    if msg == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, label_pill_subclass_proc, uid_subclass);
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    if msg == WM_ERASEBKGND {
        return 1;
    }

    if msg == WM_SETTEXT {
        let res = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        let min_w = (ref_data & 0xFFFF) as i32;
        let max_w = ((ref_data >> 16) & 0xFFFF) as i32;
        unsafe {
            fit_pill_width(hwnd, min_w, max_w);
        }
        return res;
    }

    if msg == WM_ENABLE {
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
                let (tone, surface, _) =
                    Label::style_of(hwnd).unwrap_or((LabelTone::Accent, SurfaceRole::Card, false));

                let outer_bg = match surface {
                    SurfaceRole::Card => palette.bg_card,
                    SurfaceRole::Window => palette.bg_window,
                    SurfaceRole::Sidebar => palette.bg_sidebar,
                    SurfaceRole::Input => palette.bg_card,
                };
                let br_outer = CreateSolidBrush(outer_bg);
                FillRect(hdc, &rc, br_outer);
                DeleteObject(br_outer);

                let br_pill = CreateSolidBrush(palette.bg_input);
                let pen_pill = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
                let old_b = SelectObject(hdc, br_pill);
                let old_p = SelectObject(hdc, pen_pill);
                RoundRect(
                    hdc,
                    rc.left,
                    rc.top,
                    rc.right,
                    rc.bottom,
                    radii::RADIUS_MD,
                    radii::RADIUS_MD,
                );
                SelectObject(hdc, old_b);
                SelectObject(hdc, old_p);
                DeleteObject(br_pill);
                DeleteObject(pen_pill);

                let mut buf = [0u16; 256];
                let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
                if len > 0 {
                    let is_disabled = IsWindowEnabled(hwnd) == 0;
                    let text_color = if is_disabled {
                        palette.text_secondary
                    } else {
                        match tone {
                            LabelTone::Primary => palette.text_primary,
                            LabelTone::Secondary => palette.text_secondary,
                            LabelTone::Accent => palette.accent,
                            LabelTone::Link => palette.text_link,
                        }
                    };
                    let hfont = SendMessageW(hwnd, WM_GETFONT, 0, 0);
                    let old_f = if hfont != 0 {
                        SelectObject(hdc, hfont)
                    } else {
                        0
                    };
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, text_color);
                    DrawTextW(
                        hdc,
                        buf.as_ptr(),
                        len,
                        &mut rc,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                }
                EndPaint(hwnd, &ps);
            }
        }
        return 0;
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

use std::marker::PhantomData;

/// Marker type for a standard read-only static text label.
#[derive(Debug, Clone, Copy)]
pub struct StaticText;

/// Marker type for an interactive hyperlink label (`SS_NOTIFY` + `id != 0` guaranteed).
#[derive(Debug, Clone, Copy)]
pub struct Link;

/// Marker type for a rounded auto-sizing pill badge label.
#[derive(Debug, Clone, Copy)]
pub struct PillBadge;

/// Trait associating each [`Label`] typestate with its kind-specific payload (`()` for static/pill, `u32` control ID for `Link`).
pub trait LabelKind {
    type Data: Copy;
}

impl LabelKind for StaticText {
    type Data = ();
}

impl LabelKind for PillBadge {
    type Data = ();
}

impl LabelKind for Link {
    type Data = u32;
}

pub struct Label<K: LabelKind = StaticText> {
    hwnd: isize,
    data: K::Data,
    _kind: PhantomData<K>,
}

impl Label<StaticText> {
    pub fn style_of(hwnd: isize) -> Option<(LabelTone, SurfaceRole, bool)> {
        if hwnd == 0 {
            return None;
        }
        unpack_style(unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) })
    }

    pub fn is_link_hwnd(hwnd: isize) -> bool {
        Self::style_of(hwnd).is_some_and(|(_, _, is_link)| is_link)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_raw(
        parent: isize,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
        align_flags: u32,
    ) -> Option<Self> {
        let class_name = to_wide("STATIC");
        let window_name = to_wide(text);
        let style = WS_CHILD | WS_VISIBLE | align_flags | SS_NOPREFIX;

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_name.as_ptr(),
                style,
                x,
                y,
                width,
                height,
                parent,
                0,
                0,
                null_mut(),
            )
        };

        if hwnd != 0 {
            unsafe {
                if hfont != 0 {
                    SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
                }
                SetWindowLongPtrW(
                    hwnd,
                    GWLP_USERDATA,
                    pack_style(LabelTone::Primary, SurfaceRole::Card, false),
                );
            }
            Some(Self {
                hwnd,
                data: (),
                _kind: PhantomData,
            })
        } else {
            None
        }
    }

    /// Creates a label for auto-layout engines without requiring manual (x, y, w, h) coordinates.
    pub fn new(parent: isize, text: &str, hfont: isize) -> Option<Self> {
        Self::create_raw(parent, text, 0, 0, 0, 0, hfont, SS_LEFT)
    }

    /// Creates a centered label for auto-layout engines without requiring manual coordinates.
    pub fn new_centered(parent: isize, text: &str, hfont: isize) -> Option<Self> {
        Self::create_raw(parent, text, 0, 0, 0, 0, hfont, SS_CENTER)
    }

    /// Creates a right-aligned label for auto-layout engines without requiring manual coordinates.
    pub fn new_right(parent: isize, text: &str, hfont: isize) -> Option<Self> {
        Self::create_raw(parent, text, 0, 0, 0, 0, hfont, SS_RIGHT)
    }

    pub fn create(
        parent: isize,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        Self::create_raw(parent, text, x, y, width, height, hfont, SS_LEFT)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_centered(
        parent: isize,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        Self::create_raw(parent, text, x, y, width, height, hfont, SS_CENTER)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_right(
        parent: isize,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        Self::create_raw(parent, text, x, y, width, height, hfont, SS_RIGHT)
    }

    /// Transitions a static label into an interactive hyperlink (`Label<Link>`),
    /// automatically assigning a non-zero control ID and enabling `SS_NOTIFY`.
    pub fn as_link(self) -> Label<Link> {
        let (_, surface, _) =
            Label::style_of(self.hwnd).unwrap_or((LabelTone::Primary, SurfaceRole::Card, false));
        let new_id = super::next_control_id();
        unsafe {
            let style = GetWindowLongW(self.hwnd, GWL_STYLE) as u32;
            SetWindowLongPtrW(self.hwnd, GWL_STYLE, (style | SS_NOTIFY) as isize);
            SetWindowLongPtrW(self.hwnd, GWLP_ID, new_id as isize);
            SetWindowLongPtrW(
                self.hwnd,
                GWLP_USERDATA,
                pack_style(LabelTone::Link, surface, true),
            );
        }
        Label {
            hwnd: self.hwnd,
            data: new_id,
            _kind: PhantomData,
        }
    }

    /// Transitions a static label into an auto-sizing rounded pill badge (`Label<PillBadge>`).
    pub fn as_pill_badge(self, min_w: i32, max_w: i32) -> Label<PillBadge> {
        let (_, surface, is_link) =
            Label::style_of(self.hwnd).unwrap_or((LabelTone::Accent, SurfaceRole::Card, false));
        let bounds =
            ((min_w.max(0) as usize) & 0xFFFF) | (((max_w.max(min_w) as usize) & 0xFFFF) << 16);
        unsafe {
            SetWindowLongPtrW(
                self.hwnd,
                GWLP_USERDATA,
                pack_style(LabelTone::Accent, surface, is_link),
            );
            SetWindowSubclass(
                self.hwnd,
                label_pill_subclass_proc,
                LABEL_PILL_SUBCLASS_ID,
                bounds,
            );
            fit_pill_width(self.hwnd, min_w, max_w);
        }
        Label {
            hwnd: self.hwnd,
            data: (),
            _kind: PhantomData,
        }
    }
}

impl Label<Link> {
    #[inline]
    pub fn id(&self) -> u32 {
        self.data
    }

    /// Only available on `Label<Link>`. No runtime `self.id != 0` check needed
    /// because [`Label::as_link`] guarantees a non-zero control ID and `SS_NOTIFY`.
    #[inline]
    pub fn is_clicked(&self, ctrl_id: u32, notify_code: u32) -> bool {
        ctrl_id == self.data && notify_code == 0
    }
}

impl<K: LabelKind> Label<K> {
    pub fn with_tone(self, tone: LabelTone) -> Self {
        self.set_tone(tone);
        self
    }

    pub fn on_surface(self, surface: SurfaceRole) -> Self {
        let (tone, _, is_link) =
            Label::style_of(self.hwnd).unwrap_or((LabelTone::Primary, SurfaceRole::Card, false));
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, pack_style(tone, surface, is_link));
        }
        self
    }

    pub fn set_tone(&self, tone: LabelTone) {
        let (_, surface, is_link) =
            Label::style_of(self.hwnd).unwrap_or((LabelTone::Primary, SurfaceRole::Card, false));
        unsafe {
            SetWindowLongPtrW(
                self.hwnd,
                GWLP_USERDATA,
                pack_style(tone, surface, is_link || tone == LabelTone::Link),
            );
        }
    }

    #[inline]
    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    pub fn set_text(&self, text: &str) {
        let wide = to_wide(text);
        unsafe {
            SetWindowTextW(self.hwnd, wide.as_ptr());
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

    pub fn is_enabled(&self) -> bool {
        unsafe { IsWindowEnabled(self.hwnd) != 0 }
    }

    pub fn set_enabled(&self, enabled: bool) {
        unsafe {
            EnableWindow(self.hwnd, enabled as i32);
        }
    }

    pub fn show(&self) {
        self.set_visible(true);
    }

    pub fn hide(&self) {
        self.set_visible(false);
    }
}
