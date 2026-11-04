use crate::ui::win32::*;

const MF_STRING: u32 = 0x0000;
const MF_CHECKED: u32 = 0x0008;
const MF_POPUP: u32 = 0x0010;
const MF_SEPARATOR: u32 = 0x0800;
const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_BOTTOMALIGN: u32 = 0x0020;

pub struct PopupMenu {
    hmenu: isize,
}

impl PopupMenu {
    pub fn new() -> Option<Self> {
        let hmenu = unsafe { CreatePopupMenu() };
        if hmenu != 0 {
            Some(Self { hmenu })
        } else {
            None
        }
    }

    pub fn add_item(&self, id: usize, text: &str) {
        let wide = to_wide(text);
        unsafe {
            AppendMenuW(self.hmenu, MF_STRING, id, wide.as_ptr());
        }
    }

    pub fn add_checked_item(&self, id: usize, text: &str, checked: bool) {
        let wide = to_wide(text);
        let flags = if checked {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING
        };
        unsafe {
            AppendMenuW(self.hmenu, flags, id, wide.as_ptr());
        }
    }

    pub fn add_submenu(&self, label: &str, submenu: PopupMenu) {
        let wide = to_wide(label);
        unsafe {
            AppendMenuW(self.hmenu, MF_POPUP, submenu.hmenu as usize, wide.as_ptr());
        }
        std::mem::forget(submenu); // The parent menu now owns the submenu handle
    }

    pub fn add_separator(&self) {
        unsafe {
            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, std::ptr::null());
        }
    }

    pub fn track(&self, hwnd: isize, x: i32, y: i32) {
        unsafe {
            SetForegroundWindow(hwnd);
            TrackPopupMenuEx(
                self.hmenu,
                TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                x,
                y,
                hwnd,
                std::ptr::null_mut(),
            );
        }
    }
}

impl Drop for PopupMenu {
    fn drop(&mut self) {
        if self.hmenu != 0 {
            unsafe {
                DestroyMenu(self.hmenu);
            }
        }
    }
}
