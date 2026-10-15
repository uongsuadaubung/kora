//! Native Win32 ListBox Component (Owner-Drawn Support)

use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

pub const LB_ADDSTRING: u32 = 0x0180;
pub const LB_RESETCONTENT: u32 = 0x0184;
pub const LB_SETCURSEL: u32 = 0x0186;
pub const LB_GETCURSEL: u32 = 0x0188;
pub const LB_SETITEMHEIGHT: u32 = 0x01A0;

pub const LBS_NOTIFY: u32 = 0x0001;
pub const LBS_OWNERDRAWFIXED: u32 = 0x0010;
pub const LBS_NOINTEGRALHEIGHT: u32 = 0x0100;

pub type ListBoxItemDrawer = Arc<dyn Fn(&DRAWITEMSTRUCT, bool) -> bool + Send + Sync>;

pub struct ListBoxData {
    pub parent: isize,
    pub item_drawer: Option<ListBoxItemDrawer>,
}

unsafe extern "system" fn listbox_subclass_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    uid_subclass: usize,
    ref_data: usize,
) -> isize {
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_NCDESTROY: u32 = 0x0082;
    const WM_COMMAND: u32 = 0x0111;

    let state_ptr = ref_data as *const Mutex<ListBoxData>;
    if state_ptr.is_null() {
        return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    }
    let state_arc = unsafe { &*state_ptr };

    if msg == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, listbox_subclass_proc, uid_subclass);
            drop(Arc::from_raw(state_ptr));
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
    }

    if msg == OCM_DRAWITEM {
        let dis = unsafe { &*(lparam as *const DRAWITEMSTRUCT) };
        let drawer = if let Ok(d) = state_arc.lock() {
            d.item_drawer.clone()
        } else {
            None
        };
        if let Some(drawer) = drawer {
            let is_dark = is_current_dark();
            if drawer(dis, is_dark) {
                return 1;
            }
        }
    }

    let res = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    if msg == WM_LBUTTONUP {
        let parent = if let Ok(d) = state_arc.lock() {
            d.parent
        } else {
            0
        };
        let id = uid_subclass as u32;
        if parent != 0 {
            unsafe {
                PostMessageW(parent, WM_COMMAND, (1 << 16) | (id as usize), hwnd);
            }
        }
    }
    res
}

pub struct ListBox {
    hwnd: isize,
    id: u32,
    data: Arc<Mutex<ListBoxData>>,
}

impl ListBox {
    pub fn create_owner_draw(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let id = super::resolve_id(id);
        let class_name = to_wide("LISTBOX");
        let style = WS_CHILD
            | WS_VISIBLE
            | WS_VSCROLL
            | WS_TABSTOP
            | LBS_NOTIFY
            | LBS_OWNERDRAWFIXED
            | LBS_NOINTEGRALHEIGHT;

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                null_mut(),
                style,
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
            let data = Arc::new(Mutex::new(ListBoxData {
                parent,
                item_drawer: None,
            }));
            let data_raw = Arc::into_raw(data.clone()) as usize;
            unsafe {
                SetWindowSubclass(hwnd, listbox_subclass_proc, id as usize, data_raw);
            }
            Some(Self { hwnd, id, data })
        } else {
            None
        }
    }

    pub fn set_item_drawer<F>(&self, drawer: F)
    where
        F: Fn(&DRAWITEMSTRUCT, bool) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut d) = self.data.lock() {
            d.item_drawer = Some(Arc::new(drawer));
        }
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn is_sel_changed(&self, ctrl_id: u32, notify_code: u32) -> bool {
        ctrl_id == self.id && notify_code == 1
    }

    pub fn set_item_height(&self, height: i32) {
        unsafe {
            SendMessageW(self.hwnd, LB_SETITEMHEIGHT, 0, height as isize);
        }
    }

    pub fn add_item(&self, text: &str) -> i32 {
        let wide = to_wide(text);
        unsafe { SendMessageW(self.hwnd, LB_ADDSTRING, 0, wide.as_ptr() as isize) as i32 }
    }

    pub fn reset_content(&self) {
        unsafe {
            SendMessageW(self.hwnd, LB_RESETCONTENT, 0, 0);
        }
    }

    pub fn get_selected_index(&self) -> Option<i32> {
        let sel = unsafe { SendMessageW(self.hwnd, LB_GETCURSEL, 0, 0) };
        if sel >= 0 {
            Some(sel as i32)
        } else {
            None
        }
    }

    pub fn set_selected_index(&self, index: i32) {
        unsafe {
            SendMessageW(self.hwnd, LB_SETCURSEL, index as usize, 0);
            InvalidateRect(self.hwnd, null_mut(), 1);
        }
    }
}

