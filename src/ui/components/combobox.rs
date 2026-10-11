//! Native Win32 ComboBox Component (Owner-Drawn with Smooth Padding & Modern Theming)

use crate::ui::win32::*;
use crate::ui::colors::ThemePalette;
use crate::ui::tokens::spacing;
use std::ptr::null_mut;

const CBS_DROPDOWNLIST: u32 = 0x0003;
const CBS_OWNERDRAWFIXED: u32 = 0x0010;
const CBS_HASSTRINGS: u32 = 0x0200;

const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;
const CB_RESETCONTENT: u32 = 0x014B;
const CB_GETLBTEXTLEN: u32 = 0x0149;
const CB_GETLBTEXT: u32 = 0x0148;
const CB_SETITEMHEIGHT: u32 = 0x0153;
const CB_ERR: isize = -1;
const WM_SETFONT: u32 = 0x0030;

unsafe extern "system" fn combobox_self_subclass(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    uid_subclass: usize,
    _ref_data: usize,
) -> isize {
    const WM_NCDESTROY: u32 = 0x0082;
    if msg == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, combobox_self_subclass, uid_subclass);
        }
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }

    // Message Reflection: Self-contained item rendering
    if msg == OCM_DRAWITEM {
        let dis = unsafe { &*(lparam as *const DRAWITEMSTRUCT) };
        let is_dark = crate::ui::is_current_dark();
        if ComboBox::render_item(dis, is_dark) {
            return 1;
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub struct ComboBox {
    hwnd: isize,
    id: u32,
}

impl ComboBox {
    pub fn create(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let class_name = to_wide("COMBOBOX");

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                null_mut(),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_TABSTOP
                    | WS_VSCROLL
                    | CBS_DROPDOWNLIST
                    | CBS_HASSTRINGS
                    | CBS_OWNERDRAWFIXED,
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
            unsafe {
                if hfont != 0 {
                    SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
                }
                const COMBO_SELECTION_HEIGHT: isize = 22;
                const COMBO_LIST_ITEM_HEIGHT: isize = 24;
                SendMessageW(hwnd, CB_SETITEMHEIGHT, usize::MAX, COMBO_SELECTION_HEIGHT);
                SendMessageW(hwnd, CB_SETITEMHEIGHT, 0, COMBO_LIST_ITEM_HEIGHT);

                SetWindowSubclass(hwnd, combobox_self_subclass, id as usize, 0);
            }
            Some(Self { hwnd, id })
        } else {
            None
        }
    }

    /// Unified owner-drawn renderer for Win32 ComboBox with smooth padding and modern theme colors
    pub fn render_item(dis: &DRAWITEMSTRUCT, is_dark: bool) -> bool {
        if dis.ctl_type != 3 /* ODT_COMBOBOX */ {
            return false;
        }

        unsafe {
            let palette = ThemePalette::get(is_dark);
            let is_selected = (dis.item_state & 0x0001 /* ODS_SELECTED */) != 0;
            let is_disabled = (dis.item_state & 0x0004 /* ODS_DISABLED */) != 0;
            let is_edit = (dis.item_state & 0x1000 /* ODS_COMBOBOXEDIT */) != 0;

            // Background color:
            // - Closed selection box: always use clean input background (bg_input)
            // - Dropdown list item: highlight with accent if selected/hovered, otherwise bg_input
            let bg_color = if is_edit {
                palette.bg_input
            } else if is_selected {
                palette.accent
            } else {
                palette.bg_input
            };

            // Text color:
            let text_color = if is_disabled {
                palette.text_secondary
            } else if !is_edit && is_selected {
                palette.text_white
            } else {
                palette.text_primary
            };

            // 1. Fill Item Background
            let brush = CreateSolidBrush(bg_color);
            FillRect(dis.hdc, &dis.rc_item, brush);
            DeleteObject(brush);

            // 2. Draw Item Text with comfortable padding
            if (dis.item_id as i32) >= 0 {
                let mut text_buf = [0u16; 256];
                let len = SendMessageW(
                    dis.hwnd_item,
                    CB_GETLBTEXT,
                    dis.item_id as usize,
                    text_buf.as_mut_ptr() as isize,
                );

                if len > 0 && len != CB_ERR {
                    SetBkMode(dis.hdc, TRANSPARENT);
                    SetTextColor(dis.hdc, text_color);

                    let font = crate::ui::theme::get_app_font_normal();
                    let old_font = if font != 0 {
                        SelectObject(dis.hdc, font)
                    } else {
                        0
                    };

                    let mut text_rc = dis.rc_item;
                    text_rc.left += spacing::SPACE_8;
                    text_rc.right -= spacing::SPACE_4;

                    DrawTextW(
                        dis.hdc,
                        text_buf.as_ptr(),
                        len as i32,
                        &mut text_rc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );

                    if old_font != 0 {
                        SelectObject(dis.hdc, old_font);
                    }
                }
            }

            true
        }
    }

    /// Creates a ComboBox with items and an initial selected index in a single declarative call
    #[allow(clippy::too_many_arguments)]
    pub fn create_with_items(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
        items: &[&str],
        default_selected: usize,
    ) -> Option<Self> {
        let combo = Self::create(parent, id, x, y, width, height, hfont)?;
        for item in items {
            combo.add_item(item);
        }
        if !items.is_empty() {
            combo.set_selected(default_selected);
        }
        Some(combo)
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn set_enabled(&self, enabled: bool) {
        unsafe {
            EnableWindow(self.hwnd, enabled as i32);
        }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { IsWindowEnabled(self.hwnd) != 0 }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd) != 0 }
    }

    pub fn set_visible(&self, visible: bool) {
        unsafe {
            ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }

    pub fn add_item(&self, text: &str) {
        let wide = to_wide(text);
        unsafe {
            SendMessageW(self.hwnd, CB_ADDSTRING, 0, wide.as_ptr() as isize);
        }
    }

    pub fn get_selected(&self) -> Option<usize> {
        let sel = unsafe { SendMessageW(self.hwnd, CB_GETCURSEL, 0, 0) };
        if sel != CB_ERR && sel >= 0 {
            Some(sel as usize)
        } else {
            None
        }
    }

    pub fn set_selected(&self, index: usize) {
        unsafe {
            SendMessageW(self.hwnd, CB_SETCURSEL, index, 0);
        }
    }

    pub fn clear(&self) {
        unsafe {
            SendMessageW(self.hwnd, CB_RESETCONTENT, 0, 0);
        }
    }

    pub fn reset_items(&self, items: &[&str], selected: Option<usize>) {
        self.clear();
        for item in items {
            self.add_item(item);
        }
        if let Some(sel) = selected {
            self.set_selected(sel);
        }
    }

    pub fn get_item_text(&self, index: usize) -> String {
        unsafe {
            let len = SendMessageW(self.hwnd, CB_GETLBTEXTLEN, index, 0);
            if len <= 0 || len == CB_ERR {
                return String::new();
            }
            let mut buf = vec![0u16; (len + 1) as usize];
            let res = SendMessageW(self.hwnd, CB_GETLBTEXT, index, buf.as_mut_ptr() as isize);
            if res != CB_ERR && res >= 0 {
                String::from_utf16_lossy(&buf[..res as usize])
            } else {
                String::new()
            }
        }
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let sel = self.get_selected()?;
        let text = self.get_item_text(sel);
        if text.is_empty() { None } else { Some(text) }
    }
}
