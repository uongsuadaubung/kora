//! Native Win32 ComboBox Component (Owner-Drawn with Smooth Padding & Modern Theming)

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::theme::{get_app_font_normal, is_current_dark};
use crate::ui_framework::tokens::spacing;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

pub type ComboBoxChangeCallback = Arc<dyn Fn(usize) + Send + Sync>;

const CBS_DROPDOWNLIST: u32 = 0x0003;
const CBS_OWNERDRAWFIXED: u32 = 0x0010;
const CBS_HASSTRINGS: u32 = 0x0200;

const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;
const CB_RESETCONTENT: u32 = 0x014B;
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
    ref_data: usize,
) -> isize {
    const WM_NCDESTROY: u32 = 0x0082;
    if msg == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, combobox_self_subclass, uid_subclass);
            if ref_data != 0 {
                drop(Arc::from_raw(ref_data as *const Mutex<Option<ComboBoxChangeCallback>>));
            }
        }
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }

    if msg == OCM_COMMAND {
        let notify_code = ((wparam >> 16) & 0xFFFF) as u32;
        if notify_code == 1 /* CBN_SELCHANGE */ && ref_data != 0 {
            let data_ptr = ref_data as *const Mutex<Option<ComboBoxChangeCallback>>;
            let cb = unsafe { &*data_ptr }.lock().ok().and_then(|guard| guard.clone());
            if let Some(cb) = cb {
                let cur = unsafe { SendMessageW(hwnd, CB_GETCURSEL, 0, 0) };
                if cur >= 0 {
                    cb(cur as usize);
                }
            }
            return 1;
        }
    }

    const WM_MOUSEWHEEL: u32 = 0x020A;
    const WM_MOUSEHWHEEL: u32 = 0x020E;
    const CB_GETDROPPEDSTATE: u32 = 0x0157;

    if msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL {
        let is_dropped = unsafe { SendMessageW(hwnd, CB_GETDROPPEDSTATE, 0, 0) != 0 };
        if !is_dropped {
            // Chặn combobox tự ý nhảy lựa chọn khi người dùng cuộn chuột qua lúc menu đang đóng.
            // Chuyển tiếp sự kiện cuộn chuột lên cửa sổ cha (ví dụ: ScrollView/Panel) để cuộn nội dung mượt mà.
            let parent = unsafe { GetParent(hwnd) };
            if parent != 0 {
                unsafe {
                    SendMessageW(parent, msg, wparam, lparam);
                }
            }
            return 0;
        }
    }

    if msg == OCM_DRAWITEM {
        let dis = unsafe { &*(lparam as *const DRAWITEMSTRUCT) };
        let is_dark = is_current_dark();
        if ComboBox::render_item(dis, is_dark) {
            return 1;
        }
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub struct ComboBox {
    hwnd: isize,
    id: u32,
    on_change: Arc<Mutex<Option<ComboBoxChangeCallback>>>,
}

impl ComboBox {
    /// Creates a ComboBox for auto-layout engines without requiring manual (x, y, w, h) coordinates.
    pub fn new(parent: isize, hfont: isize) -> Option<Self> {
        Self::create(parent, 0, 0, 0, 0, 160, hfont)
    }

    /// Creates a ComboBox with items for auto-layout engines without requiring manual coordinates.
    pub fn new_with_items(parent: isize, hfont: isize, items: &[&str], selected: usize) -> Option<Self> {
        Self::create_with_items(parent, 0, 0, 0, 0, 160, hfont, items, selected)
    }

    pub fn create(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let id = super::resolve_id(id);
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
            let on_change = Arc::new(Mutex::new(None));
            unsafe {
                if hfont != 0 {
                    SendMessageW(hwnd, WM_SETFONT, hfont as usize, 1);
                }
                const COMBO_SELECTION_HEIGHT: isize = 22;
                const COMBO_LIST_ITEM_HEIGHT: isize = 24;
                SendMessageW(hwnd, CB_SETITEMHEIGHT, usize::MAX, COMBO_SELECTION_HEIGHT);
                SendMessageW(hwnd, CB_SETITEMHEIGHT, 0, COMBO_LIST_ITEM_HEIGHT);

                SetWindowSubclass(
                    hwnd,
                    combobox_self_subclass,
                    id as usize,
                    Arc::into_raw(on_change.clone()) as usize,
                );
            }
            Self::apply_theme(hwnd, is_current_dark());
            Some(Self { hwnd, id, on_change })
        } else {
            None
        }
    }

    pub fn on_change<F>(self, callback: F) -> Self
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        self.set_on_change(callback);
        self
    }

    pub fn set_on_change<F>(&self, callback: F)
    where
        F: Fn(usize) + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.on_change.lock() {
            *guard = Some(Arc::new(callback));
        }
    }

    /// Binds an auto-sync callback to this combobox, executed whenever the selected index changes.
    pub fn bind<F>(&self, sync: F) -> &Self
    where
        F: FnMut(usize) + Send + Sync + 'static,
    {
        let sync_cell = std::sync::Mutex::new(sync);
        self.set_on_change(move |idx| {
            if let Ok(mut s) = sync_cell.lock() {
                s(idx);
            }
        });
        self
    }

    /// Applies Dark Mode CFD / Explorer theme to a ComboBox HWND
    pub fn apply_theme(hwnd: isize, is_dark: bool) {
        allow_window_dark_mode(hwnd, is_dark);
        let cfd_theme_str = if is_dark { "CFD" } else { "" };
        let theme_cfd_w = to_wide(cfd_theme_str);
        let res = unsafe { SetWindowTheme(hwnd, theme_cfd_w.as_ptr(), null_mut()) };
        if is_dark && res != 0 {
            unsafe {
                SetWindowTheme(hwnd, to_wide("DarkMode_CFD").as_ptr(), null_mut());
            }
        }
        unsafe {
            unsafe extern "system" fn enum_sub_theme(child: isize, lparam: isize) -> i32 {
                let is_dark = lparam != 0;
                allow_window_dark_mode(child, is_dark);
                let cfd_theme_str = if is_dark { "CFD" } else { "" };
                let theme_cfd_w = to_wide(cfd_theme_str);
                let res = unsafe { SetWindowTheme(child, theme_cfd_w.as_ptr(), null_mut()) };
                if is_dark && res != 0 {
                    unsafe {
                        SetWindowTheme(child, to_wide("DarkMode_CFD").as_ptr(), null_mut());
                    }
                }
                1
            }
            EnumChildWindows(hwnd, Some(enum_sub_theme), is_dark as isize);

            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
            InvalidateRect(hwnd, null_mut(), 1);
        }
    }

    pub fn render_item(dis: &DRAWITEMSTRUCT, is_dark: bool) -> bool {
        if dis.ctl_type != 3
        /* ODT_COMBOBOX */
        {
            return false;
        }

        unsafe {
            let palette = ThemePalette::get(is_dark);
            let is_selected = (dis.item_state & 0x0001/* ODS_SELECTED */) != 0;
            let is_disabled = (dis.item_state & 0x0004/* ODS_DISABLED */) != 0;
            let is_edit = (dis.item_state & 0x1000/* ODS_COMBOBOXEDIT */) != 0;

            let bg_color = if is_edit {
                palette.bg_input
            } else if is_selected {
                palette.accent
            } else {
                palette.bg_input
            };

            let text_color = if is_disabled {
                palette.text_secondary
            } else if !is_edit && is_selected {
                palette.text_white
            } else {
                palette.text_primary
            };

            let brush = CreateSolidBrush(bg_color);
            FillRect(dis.hdc, &dis.rc_item, brush);
            DeleteObject(brush);

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

                    let font = get_app_font_normal();
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

    /// Returns `true` when this ComboBox's selection changed in `WM_COMMAND` (`CBN_SELCHANGE = 1`).
    #[inline]
    pub fn is_changed(&self, ctrl_id: u32, notify_code: u32) -> bool {
        ctrl_id == self.id && notify_code == CBN_SELCHANGE
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

    /// Replaces all items while preserving the currently selected index.
    pub fn set_items_preserve_selection(&self, items: &[&str]) {
        Self::set_items_by_hwnd(self.hwnd, items);
    }

    /// Replaces all items on a ComboBox `hwnd` while preserving its currently selected index.
    pub fn set_items_by_hwnd(hwnd: isize, items: &[&str]) {
        if hwnd == 0 {
            return;
        }
        unsafe {
            let sel = SendMessageW(hwnd, CB_GETCURSEL, 0, 0);
            SendMessageW(hwnd, CB_RESETCONTENT, 0, 0);
            for item in items {
                let wide = to_wide(item);
                SendMessageW(hwnd, CB_ADDSTRING, 0, wide.as_ptr() as isize);
            }
            if !items.is_empty() && sel != CB_ERR && sel >= 0 {
                let clamped = (sel as usize).min(items.len() - 1);
                SendMessageW(hwnd, CB_SETCURSEL, clamped, 0);
            }
            InvalidateRect(hwnd, null_mut(), 1);
        }
    }
}
