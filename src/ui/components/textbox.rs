use crate::ui::win32::*;
use crate::ui::tokens::spacing;
use std::ptr::null_mut;

const CONTAINER_SUBCLASS_ID: usize = 0x408;
const EDIT_SUBCLASS_ID: usize = 0x407;

#[link(name = "user32")]
unsafe extern "system" {
    fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: isize,
        h_menu: isize,
        h_instance: isize,
        lp_param: *mut std::ffi::c_void,
    ) -> isize;
    fn SendMessageW(h_wnd: isize, msg: u32, w_param: usize, l_param: isize) -> isize;
    fn GetWindowTextW(h_wnd: isize, lp_string: *mut u16, n_max_count: i32) -> i32;
    fn SetWindowTextW(h_wnd: isize, lp_string: *const u16) -> i32;
    fn GetWindowTextLengthW(h_wnd: isize) -> i32;
    fn GetParent(h_wnd: isize) -> isize;
    fn SetFocus(h_wnd: isize) -> isize;
}

const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_TABSTOP: u32 = 0x00010000;
const WS_CLIPCHILDREN: u32 = 0x02000000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const WM_SETFONT: u32 = 0x0030;

struct ContainerSubclassData {
    inner_edit: isize,
}

unsafe extern "system" fn container_subclass_proc(
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
    const WM_SIZE: u32 = 0x0005;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_SETCURSOR: u32 = 0x0020;
    const WM_SETFOCUS: u32 = 0x0007;
    const WM_CTLCOLOREDIT: u32 = 0x0133;
    const WM_GETTEXT: u32 = 0x000D;
    const WM_GETTEXTLENGTH: u32 = 0x000E;
    const WM_SETTEXT: u32 = 0x000C;
    const WM_SETFONT: u32 = 0x0030;
    const WM_COMMAND: u32 = 0x0111;
    const IDC_IBEAM: usize = 32513;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;

    if msg == WM_NCDESTROY {
        if ref_data != 0 {
            unsafe {
                RemoveWindowSubclass(hwnd, container_subclass_proc, uid_subclass);
                drop(Box::from_raw(ref_data as *mut ContainerSubclassData));
            }
        }
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }

    if ref_data == 0 {
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }

    let data = unsafe { &*(ref_data as *const ContainerSubclassData) };

    if msg == WM_ERASEBKGND {
        return 1;
    }

    if msg == WM_LBUTTONDOWN || msg == WM_SETFOCUS {
        unsafe {
            SetFocus(data.inner_edit);
        }
        return 0;
    }

    if msg == WM_SETCURSOR {
        unsafe {
            let cur = LoadCursorW(0, IDC_IBEAM);
            if cur != 0 {
                SetCursor(cur);
                return 1;
            }
        }
    }

    if msg == WM_GETTEXT || msg == WM_GETTEXTLENGTH || msg == WM_SETTEXT || msg == WM_SETFONT {
        return unsafe { SendMessageW(data.inner_edit, msg, wparam, lparam) };
    }

    if msg == WM_CTLCOLOREDIT {
        let hdc = wparam as isize;
        let is_dark = crate::ui::is_current_dark();
        let palette = crate::ui::colors::ThemePalette::get(is_dark);
        unsafe {
            SetTextColor(hdc, palette.text_input);
            SetBkColor(hdc, palette.bg_input);
            let br = crate::ui::theme::get_input_bg_brush();
            if br != 0 {
                return br;
            }
        }
    }

    if msg == WM_COMMAND {
        let parent = unsafe { GetParent(hwnd) };
        if parent != 0 {
            let code = (wparam >> 16) as u16;
            let id = unsafe { GetDlgCtrlID(hwnd) } as u16;
            let new_wparam = ((code as usize) << 16) | (id as usize);
            unsafe {
                SendMessageW(parent, WM_COMMAND, new_wparam, hwnd);
            }
        }
        return 0;
    }

    if msg == WM_SIZE {
        let res = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        let w = (lparam as i32) & 0xFFFF;
        let h = ((lparam as i32) >> 16) & 0xFFFF;
        if w > 0 && h > 0 {
            let inner_h = 18;
            let inner_y = (h - inner_h) / 2;
            let inner_x = spacing::SPACE_8;
            let inner_w = (w - spacing::SPACE_8 * 2).max(10);
            unsafe {
                SetWindowPos(
                    data.inner_edit,
                    0,
                    inner_x,
                    inner_y,
                    inner_w,
                    inner_h,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        return res;
    }

    if msg == WM_PAINT {
        let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        if hdc != 0 {
            let is_dark = crate::ui::is_current_dark();
            let palette = crate::ui::colors::ThemePalette::get(is_dark);
            let mut rc = RECT::default();
            unsafe {
                GetClientRect(hwnd, &mut rc);
            }

            // 1. Fill background
            let bg_brush = unsafe { CreateSolidBrush(palette.bg_input) };
            unsafe {
                FillRect(hdc, &rc, bg_brush);
                DeleteObject(bg_brush);
            }

            // 2. Draw 1px border around all 4 sides (highlighted with accent when focused)
            let is_focused = unsafe { GetFocus() == data.inner_edit };
            let border_color = if is_focused {
                palette.accent
            } else {
                palette.border
            };
            let border_brush = unsafe { CreateSolidBrush(border_color) };
            unsafe {
                FrameRect(hdc, &rc, border_brush);
                DeleteObject(border_brush);
            }

            unsafe {
                EndPaint(hwnd, &ps);
            }
        }
        return 0;
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

struct EditSubclassData {
    cue: Vec<u16>,
    font: isize,
    container: isize,
}

unsafe extern "system" fn edit_subclass_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    uid_subclass: usize,
    ref_data: usize,
) -> isize {
    const WM_PAINT: u32 = 0x000F;
    const WM_SETFOCUS: u32 = 0x0007;
    const WM_KILLFOCUS: u32 = 0x0008;
    const WM_NCDESTROY: u32 = 0x0082;
    const WM_SET_CUE_BANNER: u32 = 0x8000 + 10;

    if msg == WM_NCDESTROY {
        if ref_data != 0 {
            unsafe {
                RemoveWindowSubclass(hwnd, edit_subclass_proc, uid_subclass);
                drop(Box::from_raw(ref_data as *mut EditSubclassData));
            }
        }
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }

    if ref_data == 0 {
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }

    let data = unsafe { &mut *(ref_data as *mut EditSubclassData) };

    if msg == WM_SET_CUE_BANNER {
        let text_ptr = lparam as *const u16;
        if !text_ptr.is_null() {
            let mut len = 0;
            while unsafe { *text_ptr.add(len) } != 0 {
                len += 1;
            }
            data.cue = unsafe { std::slice::from_raw_parts(text_ptr, len + 1) }.to_vec();
            unsafe {
                InvalidateRect(hwnd, null_mut(), 1);
            }
        }
        return 1;
    }

    if msg == WM_SETFOCUS || msg == WM_KILLFOCUS {
        let res = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        unsafe {
            InvalidateRect(hwnd, null_mut(), 1);
            if data.container != 0 {
                InvalidateRect(data.container, null_mut(), 1);
            }
        }
        return res;
    }

    if msg == WM_PAINT {
        let res = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        if !data.cue.is_empty() {
            unsafe {
                if GetWindowTextLengthW(hwnd) == 0 && GetFocus() != hwnd {
                    let hdc = GetDC(hwnd);
                    if hdc != 0 {
                        let is_dark = crate::ui::is_current_dark();
                        let palette = crate::ui::colors::ThemePalette::get(is_dark);
                        let mut rc = RECT::default();
                        GetClientRect(hwnd, &mut rc);
                        SetBkMode(hdc, TRANSPARENT);
                        SetTextColor(hdc, palette.text_secondary);
                        if data.font != 0 {
                            SelectObject(hdc, data.font);
                        }
                        DrawTextW(
                            hdc,
                            data.cue.as_ptr(),
                            data.cue.len() as i32 - 1,
                            &mut rc,
                            DT_VCENTER | DT_SINGLELINE,
                        );
                        ReleaseDC(hwnd, hdc);
                    }
                }
            }
        }
        return res;
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub struct TextBox {
    container_hwnd: isize,
    inner_hwnd: isize,
    id: u32,
}

impl TextBox {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        parent: isize,
        id: u32,
        initial_text: &str,
        cue_banner: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        // 1. Container window (holds custom border, background, and pads child edit)
        let container = unsafe {
            CreateWindowExW(
                0,
                to_wide("STATIC").as_ptr(),
                null_mut(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_CLIPCHILDREN,
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

        if container == 0 {
            return None;
        }

        // 2. Borderless native EDIT control centered vertically inside container
        let inner_h = 18;
        let inner_y = (height - inner_h) / 2;
        let inner_x = spacing::SPACE_8;
        let inner_w = (width - spacing::SPACE_8 * 2).max(10);

        let inner_edit = unsafe {
            CreateWindowExW(
                0,
                to_wide("EDIT").as_ptr(),
                to_wide(initial_text).as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL,
                inner_x,
                inner_y,
                inner_w,
                inner_h,
                container,
                1001,
                0,
                null_mut(),
            )
        };

        if inner_edit == 0 {
            unsafe {
                DestroyWindow(container);
            }
            return None;
        }

        if hfont != 0 {
            unsafe {
                SendMessageW(inner_edit, WM_SETFONT, hfont as usize, 1);
            }
        }

        // 3. Subclass container
        let c_data = Box::into_raw(Box::new(ContainerSubclassData {
            inner_edit,
        }));
        unsafe {
            SetWindowSubclass(
                container,
                container_subclass_proc,
                CONTAINER_SUBCLASS_ID,
                c_data as usize,
            );
        }

        // 4. Subclass inner edit
        let e_data = Box::into_raw(Box::new(EditSubclassData {
            cue: to_wide(cue_banner),
            font: hfont,
            container,
        }));
        unsafe {
            SetWindowSubclass(
                inner_edit,
                edit_subclass_proc,
                EDIT_SUBCLASS_ID,
                e_data as usize,
            );
        }

        Some(Self {
            container_hwnd: container,
            inner_hwnd: inner_edit,
            id,
        })
    }

    /// Declarative helper: creates a TextBox with a placeholder in 1 line
    #[allow(clippy::too_many_arguments)]
    pub fn create_with_placeholder(
        parent: isize,
        id: u32,
        cue_banner: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        Self::create(parent, id, "", cue_banner, x, y, width, height, hfont)
    }

    pub fn hwnd(&self) -> isize {
        self.container_hwnd
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn get_text(&self) -> String {
        let len = unsafe { GetWindowTextLengthW(self.inner_hwnd) };
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let copied = unsafe { GetWindowTextW(self.inner_hwnd, buf.as_mut_ptr(), len + 1) };
        if copied > 0 {
            String::from_utf16_lossy(&buf[..copied as usize])
        } else {
            String::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        unsafe { GetWindowTextLengthW(self.inner_hwnd) == 0 }
    }

    pub fn clear(&self) {
        self.set_text("");
    }

    pub fn set_text(&self, text: &str) {
        let wide = to_wide(text);
        unsafe {
            SetWindowTextW(self.inner_hwnd, wide.as_ptr());
        }
    }

    pub fn set_cue_banner(&self, text: &str) {
        const WM_SET_CUE_BANNER: u32 = 0x8000 + 10;
        let wide = to_wide(text);
        unsafe {
            SendMessageW(self.inner_hwnd, WM_SET_CUE_BANNER, 0, wide.as_ptr() as isize);
        }
    }
}
