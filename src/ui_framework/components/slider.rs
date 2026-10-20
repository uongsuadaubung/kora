//! Native Win32 Trackbar / Slider Component
//! Uses msctls_trackbar32 from comctl32.dll for smooth native Windows slider controls.

use crate::ui_framework::theme::is_current_dark;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;

const TBS_HORZ: u32 = 0x0000;
const TBS_AUTOTICKS: u32 = 0x0001;

const TBM_GETPOS: u32 = 0x0400;
const TBM_SETRANGE: u32 = 0x0406;
const TBM_SETPOS: u32 = 0x0405;
const TBM_SETTICFREQ: u32 = 0x0414;

pub struct Slider {
    hwnd: isize,
    id: u32,
}

impl Slider {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        min: u32,
        max: u32,
    ) -> Option<Self> {
        let id = super::resolve_id(id);
        let class_name = to_wide("msctls_trackbar32");
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                null_mut(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | TBS_HORZ | TBS_AUTOTICKS,
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
            return None;
        }

        unsafe {
            let range = ((max as isize) << 16) | (min as isize & 0xFFFF);
            SendMessageW(hwnd, TBM_SETRANGE, 1, range);
            SendMessageW(hwnd, TBM_SETTICFREQ, 10, 0);
        }

        let slider = Self { hwnd, id };
        slider.apply_theme(is_current_dark());
        Some(slider)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_with_value(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        min: u32,
        max: u32,
        initial_value: u32,
    ) -> Option<Self> {
        let slider = Self::create(parent, id, x, y, width, height, min, max)?;
        slider.set_pos(initial_value);
        Some(slider)
    }

    #[inline]
    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn is_scrolled(&self, lparam: isize) -> bool {
        lparam != 0 && lparam == self.hwnd
    }

    #[inline]
    pub const fn is_dragging(wparam: usize) -> bool {
        const TB_THUMBTRACK: usize = 5;
        (wparam & 0xFFFF) == TB_THUMBTRACK
    }

    #[inline]
    pub fn parse_scroll(&self, wparam: usize, lparam: isize) -> Option<(u32, bool)> {
        if !self.is_scrolled(lparam) {
            return None;
        }
        Some((self.get_pos(), !Self::is_dragging(wparam)))
    }

    pub fn get_pos(&self) -> u32 {
        unsafe { SendMessageW(self.hwnd, TBM_GETPOS, 0, 0) as u32 }
    }

    pub fn set_pos(&self, pos: u32) {
        unsafe {
            SendMessageW(self.hwnd, TBM_SETPOS, 1, pos as isize);
        }
    }

    pub fn set_range(&self, min: u32, max: u32) {
        unsafe {
            let range = ((max as isize) << 16) | (min as isize & 0xFFFF);
            SendMessageW(self.hwnd, TBM_SETRANGE, 1, range);
        }
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
        if visible {
            self.apply_theme(is_current_dark());
        }
    }

    pub fn apply_theme(&self, is_dark: bool) {
        unsafe {
            allow_window_dark_mode(self.hwnd, is_dark);
            let theme_str = if is_dark {
                "DarkMode_Explorer"
            } else {
                "Explorer"
            };
            let res = SetWindowTheme(self.hwnd, to_wide(theme_str).as_ptr(), null_mut());
            if is_dark && res != 0 {
                SetWindowTheme(self.hwnd, to_wide("DarkMode_CFD").as_ptr(), null_mut());
            }
            SetWindowPos(
                self.hwnd,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
            InvalidateRect(self.hwnd, null_mut(), 1);
            UpdateWindow(self.hwnd);
        }
    }
}
