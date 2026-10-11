//! System Tray Icon Component using Shell_NotifyIconW

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cb_size: u32,
    pub h_wnd: isize,
    pub u_id: u32,
    pub u_flags: u32,
    pub u_callback_message: u32,
    pub h_icon: isize,
    pub sz_tip: [u16; 128],
    pub dw_state: u32,
    pub dw_state_mask: u32,
    pub sz_info: [u16; 256],
    pub u_timeout_or_version: u32,
    pub sz_info_title: [u16; 64],
    pub dw_info_flags: u32,
    pub guid_item: [u8; 16],
    pub h_balloon_icon: isize,
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(dw_message: u32, lp_data: *mut NOTIFYICONDATAW) -> i32;
}

pub const NIM_ADD: u32 = 0x00000000;
pub const NIM_MODIFY: u32 = 0x00000001;
pub const NIM_DELETE: u32 = 0x00000002;

pub const NIF_MESSAGE: u32 = 0x00000001;
pub const NIF_ICON: u32 = 0x00000002;
pub const NIF_TIP: u32 = 0x00000004;

pub struct TrayIcon {
    nid: NOTIFYICONDATAW,
    is_added: bool,
}

impl TrayIcon {
    pub fn new(hwnd: isize, id: u32, callback_msg: u32) -> Self {
        let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.h_wnd = hwnd;
        nid.u_id = id;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.u_callback_message = callback_msg;

        Self {
            nid,
            is_added: false,
        }
    }

    pub fn set_icon(&mut self, hicon: isize) {
        self.nid.h_icon = hicon;
        self.nid.u_flags |= NIF_ICON;
        if self.is_added {
            unsafe {
                Shell_NotifyIconW(NIM_MODIFY, &mut self.nid);
            }
        }
    }

    pub fn set_tooltip(&mut self, text: &str) {
        let wide: Vec<u16> = OsStr::new(text).encode_wide().chain(Some(0)).collect();
        let len = wide.len().min(127);
        self.nid.sz_tip[..len].copy_from_slice(&wide[..len]);
        self.nid.sz_tip[len] = 0;
        self.nid.u_flags |= NIF_TIP;
        if self.is_added {
            unsafe {
                Shell_NotifyIconW(NIM_MODIFY, &mut self.nid);
            }
        }
    }

    pub fn show(&mut self) -> bool {
        if !self.is_added {
            let res = unsafe { Shell_NotifyIconW(NIM_ADD, &mut self.nid) };
            if res != 0 {
                self.is_added = true;
                return true;
            }
        }
        false
    }

    pub fn hide(&mut self) -> bool {
        if self.is_added {
            let res = unsafe { Shell_NotifyIconW(NIM_DELETE, &mut self.nid) };
            if res != 0 {
                self.is_added = false;
                return true;
            }
        }
        false
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.hide();
    }
}
