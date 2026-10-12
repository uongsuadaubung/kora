//! Native Win32 Popup / Context Menu Component

use crate::ui::win32::to_wide;
use std::ptr::null_mut;

#[link(name = "user32")]
unsafe extern "system" {
    fn CreatePopupMenu() -> isize;
    fn DestroyMenu(h_menu: isize) -> i32;
    fn AppendMenuW(
        h_menu: isize,
        u_flags: u32,
        u_id_new_item: usize,
        lp_new_item: *const u16,
    ) -> i32;
    fn TrackPopupMenu(
        h_menu: isize,
        u_flags: u32,
        x: i32,
        y: i32,
        n_reserved: i32,
        h_wnd: isize,
        prc_rect: *const std::ffi::c_void,
    ) -> u32;
    fn SetForegroundWindow(h_wnd: isize) -> i32;
}

const MF_STRING: u32 = 0x0000;
const MF_SEPARATOR: u32 = 0x0800;
const MF_CHECKED: u32 = 0x0008;
const MF_UNCHECKED: u32 = 0x0000;

const TPM_RETURNCMD: u32 = 0x0100;
const TPM_NONOTIFY: u32 = 0x0080;
const TPM_RIGHTBUTTON: u32 = 0x0002;

pub struct PopupMenu {
    h_menu: isize,
}

impl PopupMenu {
    pub fn new() -> Option<Self> {
        let h_menu = unsafe { CreatePopupMenu() };
        if h_menu != 0 {
            Some(Self { h_menu })
        } else {
            None
        }
    }

    pub fn add_item(&mut self, id: u32, text: &str) {
        let wide = to_wide(text);
        unsafe {
            AppendMenuW(self.h_menu, MF_STRING, id as usize, wide.as_ptr());
        }
    }

    pub fn add_checked_item(&mut self, id: u32, text: &str, checked: bool) {
        let flags = MF_STRING | if checked { MF_CHECKED } else { MF_UNCHECKED };
        let wide = to_wide(text);
        unsafe {
            AppendMenuW(self.h_menu, flags, id as usize, wide.as_ptr());
        }
    }

    pub fn add_separator(&mut self) {
        unsafe {
            AppendMenuW(self.h_menu, MF_SEPARATOR, 0, null_mut());
        }
    }

    pub fn track(&self, hwnd: isize, x: i32, y: i32) -> u32 {
        unsafe {
            SetForegroundWindow(hwnd);
            let cmd = TrackPopupMenu(
                self.h_menu,
                TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
                x,
                y,
                0,
                hwnd,
                null_mut(),
            );
            // KB135788: ensure the menu closes properly when the user clicks elsewhere
            crate::ui::win32::PostMessageW(hwnd, 0 /* WM_NULL */, 0, 0);
            cmd
        }
    }
}

impl Drop for PopupMenu {
    fn drop(&mut self) {
        if self.h_menu != 0 {
            unsafe {
                DestroyMenu(self.h_menu);
            }
        }
    }
}
