//! Modern Segmented Pill TabBar Control
//! Native Win32 Fluent Design Segmented Control with smooth pill animations and hover states.

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::tokens::{borders, radii};
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, Once};

const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_SETCURSOR: u32 = 0x0020;
const WM_COMMAND: u32 = 0x0111;

pub struct TabBarState {
    pub tabs: Vec<String>,
    pub cur_sel: usize,
    pub hover_idx: isize,
    pub font_normal: isize,
    pub font_bold: isize,
    pub parent: isize,
    pub control_id: u32,
}

static REGISTER_CLASS_ONCE: Once = Once::new();
const CLASS_NAME: &str = "KoraTabBarClass";

fn ensure_class_registered() {
    REGISTER_CLASS_ONCE.call_once(|| unsafe {
        let class_name_w = to_wide(CLASS_NAME);
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfn_wnd_proc = Some(tab_bar_wnd_proc);
        wc.lpsz_class_name = class_name_w.as_ptr();
        wc.h_cursor = LoadCursorW(0, IDC_ARROW);
        RegisterClassExW(&wc);
    });
}

unsafe extern "system" fn tab_bar_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<TabBarState> };

    if msg == WM_CREATE {
        return 0;
    }

    if msg == WM_DESTROY {
        if !state_ptr.is_null() {
            unsafe {
                drop(Arc::from_raw(state_ptr));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    if state_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    let state_mutex = unsafe { &*state_ptr };

    match msg {
        WM_ERASEBKGND => 1,
        WM_MOUSEMOVE => {
            let mut tme = TRACKMOUSEEVENT {
                cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dw_flags: TME_LEAVE,
                hwnd_track: hwnd,
                dw_hover_time: 0,
            };
            unsafe {
                TrackMouseEvent(&mut tme);
                let mut rc = RECT::default();
                GetClientRect(hwnd, &mut rc);
                let w = rc.right - rc.left;
                if w > 0 {
                    let x = (lparam as i16) as i32;
                    let changed = if let Ok(mut g) = state_mutex.lock() {
                        let num_tabs = g.tabs.len();
                        if num_tabs > 0 {
                            let track_padding = 3;
                            let available_w = (w - (track_padding * 2)).max(1);
                            let tab_w = (available_w / num_tabs as i32).max(1);
                            let idx =
                                ((x - track_padding) / tab_w).clamp(0, (num_tabs - 1) as i32) as isize;
                            let old = g.hover_idx;
                            g.hover_idx = idx;
                            old != idx
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if changed {
                        InvalidateRect(hwnd, null_mut(), 0);
                    }
                }
            }
            0
        }
        WM_MOUSELEAVE => {
            let changed = if let Ok(mut g) = state_mutex.lock() {
                let old = g.hover_idx;
                g.hover_idx = -1;
                old != -1
            } else {
                false
            };
            if changed {
                unsafe {
                    InvalidateRect(hwnd, null_mut(), 0);
                }
            }
            0
        }
        WM_SETCURSOR => {
            unsafe {
                SetCursor(LoadCursorW(0, IDC_ARROW));
            }
            1
        }
        WM_LBUTTONDOWN => {
            let mut rc = RECT::default();
            unsafe {
                GetClientRect(hwnd, &mut rc);
            }
            let w = rc.right - rc.left;
            if w > 0 {
                let x = (lparam as i16) as i32;
                let (changed, parent, id) = if let Ok(mut g) = state_mutex.lock() {
                    let num_tabs = g.tabs.len();
                    if num_tabs > 0 {
                        let track_padding = 3;
                        let available_w = (w - (track_padding * 2)).max(1);
                        let tab_w = (available_w / num_tabs as i32).max(1);
                        let idx =
                            ((x - track_padding) / tab_w).clamp(0, (num_tabs - 1) as i32) as usize;
                        let old = g.cur_sel;
                        if old != idx {
                            g.cur_sel = idx;
                            (true, g.parent, g.control_id)
                        } else {
                            (false, 0, 0)
                        }
                    } else {
                        (false, 0, 0)
                    }
                } else {
                    (false, 0, 0)
                };

                if changed {
                    unsafe {
                        InvalidateRect(hwnd, null_mut(), 0);
                    }
                    if parent != 0 {
                        unsafe {
                            SendMessageW(parent, WM_COMMAND, (id & 0xFFFF) as usize, hwnd);
                        }
                    }
                }
            }
            0
        }
        WM_PAINT => {
            unsafe {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let mut rc = RECT::default();
                    GetClientRect(hwnd, &mut rc);
                    let w = rc.right - rc.left;
                    let h = rc.bottom - rc.top;

                    if w > 0 && h > 0 {
                        if let Some(bb) = BackBuffer::new(hdc, w, h) {
                            render_tab_bar_to_dc(bb.hdc(), &rc, state_mutex);
                        } else {
                            render_tab_bar_to_dc(hdc, &rc, state_mutex);
                        }
                    }

                    EndPaint(hwnd, &ps);
                }
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn render_tab_bar_to_dc(hdc: isize, rc: &RECT, state_mutex: &Mutex<TabBarState>) {
    unsafe {
        let is_dark = is_current_dark();
        let palette = ThemePalette::get(is_dark);

        let (tabs, cur_sel, hover_idx, font_normal, font_bold) = if let Ok(g) = state_mutex.lock() {
            (
                g.tabs.clone(),
                g.cur_sel,
                g.hover_idx,
                g.font_normal,
                g.font_bold,
            )
        } else {
            (Vec::new(), 0, -1, 0, 0)
        };

        let brush_bg = CreateSolidBrush(palette.bg_window);
        FillRect(hdc, rc, brush_bg);
        DeleteObject(brush_bg);

        let brush_track = CreateSolidBrush(palette.tab_track_bg);
        let pen_track = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.tab_track_border);
        let old_brush = SelectObject(hdc, brush_track);
        let old_pen = SelectObject(hdc, pen_track);

        RoundRect(
            hdc,
            rc.left,
            rc.top,
            rc.right,
            rc.bottom,
            radii::RADIUS_PILL,
            radii::RADIUS_PILL,
        );

        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        DeleteObject(brush_track);
        DeleteObject(pen_track);

        let num_tabs = tabs.len() as i32;
        if num_tabs > 0 {
            let track_padding = 3;
            let available_w = (rc.right - rc.left) - (track_padding * 2);
            let tab_w = available_w / num_tabs;

            SetBkMode(hdc, TRANSPARENT);

            for (i, title) in tabs.iter().enumerate() {
                let i_i32 = i as i32;
                let item_left = track_padding + i_i32 * tab_w;
                let item_right = if i == tabs.len() - 1 {
                    rc.right - track_padding
                } else {
                    item_left + tab_w
                };
                let item_top = track_padding;
                let item_bottom = rc.bottom - track_padding;

                let mut item_rc = RECT {
                    left: item_left,
                    top: item_top,
                    right: item_right,
                    bottom: item_bottom,
                };

                if i == cur_sel {
                    let h_active_br = CreateSolidBrush(palette.tab_active_bg);
                    let h_active_pen =
                        CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.tab_active_border);
                    let ob = SelectObject(hdc, h_active_br);
                    let op = SelectObject(hdc, h_active_pen);

                    RoundRect(
                        hdc,
                        item_rc.left,
                        item_rc.top,
                        item_rc.right,
                        item_rc.bottom,
                        radii::RADIUS_CARD,
                        radii::RADIUS_CARD,
                    );

                    SelectObject(hdc, ob);
                    SelectObject(hdc, op);
                    DeleteObject(h_active_br);
                    DeleteObject(h_active_pen);

                    if font_bold != 0 {
                        SelectObject(hdc, font_bold);
                    }
                    SetTextColor(hdc, palette.accent);

                    let line_rc = RECT {
                        left: item_rc.left + 4,
                        top: item_rc.bottom - 2,
                        right: item_rc.right - 4,
                        bottom: item_rc.bottom,
                    };
                    let line_br = CreateSolidBrush(palette.accent);
                    FillRect(hdc, &line_rc, line_br);
                    DeleteObject(line_br);
                } else {
                    if hover_idx == i as isize {
                        let h_hvr_br = CreateSolidBrush(palette.tab_hover_bg);
                        let h_null_pen = CreatePen(PS_NULL, 0, 0);
                        let ob = SelectObject(hdc, h_hvr_br);
                        let op = SelectObject(hdc, h_null_pen);

                        RoundRect(
                            hdc,
                            item_rc.left,
                            item_rc.top,
                            item_rc.right,
                            item_rc.bottom,
                            radii::RADIUS_CARD,
                            radii::RADIUS_CARD,
                        );

                        SelectObject(hdc, ob);
                        SelectObject(hdc, op);
                        DeleteObject(h_hvr_br);
                        DeleteObject(h_null_pen);
                    }

                    if font_normal != 0 {
                        SelectObject(hdc, font_normal);
                    }
                    SetTextColor(hdc, palette.text_secondary);
                }

                let title_w = to_wide(title);
                DrawTextW(
                    hdc,
                    title_w.as_ptr(),
                    title.encode_utf16().count() as i32,
                    &mut item_rc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
            }
        }
    }
}

#[derive(Clone)]
pub struct TabBar {
    hwnd: isize,
    id: u32,
    state: Arc<Mutex<TabBarState>>,
}

impl TabBar {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont_normal: isize,
        hfont_bold: isize,
        tabs: &[impl AsRef<str>],
    ) -> Option<Self> {
        let id = super::resolve_id(id);
        ensure_class_registered();

        let state = Arc::new(Mutex::new(TabBarState {
            tabs: tabs.iter().map(|s| s.as_ref().to_string()).collect(),
            cur_sel: 0,
            hover_idx: -1,
            font_normal: hfont_normal,
            font_bold: hfont_bold,
            parent,
            control_id: id,
        }));

        let raw_state = Arc::into_raw(state.clone());
        let class_name_w = to_wide(CLASS_NAME);

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name_w.as_ptr(),
                to_wide("TabBar").as_ptr(),
                WS_CHILD | WS_VISIBLE,
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

        if hwnd == 0 {
            unsafe {
                drop(Arc::from_raw(raw_state));
            }
            return None;
        }

        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw_state as isize);
        }

        Some(Self { hwnd, id, state })
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn is_changed(&self, ctrl_id: u32, notify_code: u32) -> bool {
        ctrl_id == self.id && notify_code == 0
    }

    pub fn get_cur_sel(&self) -> usize {
        if let Ok(g) = self.state.lock() {
            g.cur_sel
        } else {
            0
        }
    }

    pub fn set_cur_sel(&self, idx: usize) {
        if let Ok(mut g) = self.state.lock()
            && idx < g.tabs.len()
        {
            g.cur_sel = idx;
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn set_tabs(&self, tabs: &[impl AsRef<str>]) {
        if let Ok(mut g) = self.state.lock() {
            g.tabs = tabs.iter().map(|s| s.as_ref().to_string()).collect();
            if g.cur_sel >= g.tabs.len() {
                g.cur_sel = 0;
            }
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn set_visible(&self, visible: bool) {
        let cmd = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.hwnd, cmd);
        }
    }
}

