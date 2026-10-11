//! Native Win32 Label (Static text) Component

use crate::ui::win32::*;
use std::ptr::null_mut;

const SS_LEFT: u32 = 0x00000000;
const SS_CENTER: u32 = 0x00000001;
const SS_RIGHT: u32 = 0x0002;
const SS_NOPREFIX: u32 = 0x00000080;
const SS_NOTIFY: u32 = 0x00000100;

pub struct Label {
    hwnd: isize,
}

impl Label {
    pub fn create(
        parent: isize,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        Self::create_with_id(parent, 0, text, x, y, width, height, hfont)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_centered_with_id(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let class_name = to_wide("STATIC");
        let window_name = to_wide(text);
        let style = WS_CHILD | WS_VISIBLE | SS_CENTER | SS_NOTIFY | SS_NOPREFIX;

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
            Some(Self { hwnd })
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_right_with_id(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let class_name = to_wide("STATIC");
        let window_name = to_wide(text);
        let style = WS_CHILD | WS_VISIBLE | SS_RIGHT | SS_NOTIFY | SS_NOPREFIX;

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
            Some(Self { hwnd })
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_with_id(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let class_name = to_wide("STATIC");
        let window_name = to_wide(text);

        let style = if id != 0 {
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_NOTIFY | SS_NOPREFIX
        } else {
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_NOPREFIX
        };

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
            Some(Self { hwnd })
        } else {
            None
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_centered(
        parent: isize,
        id: u32,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hfont: isize,
    ) -> Option<Self> {
        let class_name = to_wide("STATIC");
        let window_name = to_wide(text);
        const SS_CENTER: u32 = 0x00000001;

        let style = if id != 0 {
            WS_CHILD | WS_VISIBLE | SS_CENTER | SS_NOTIFY | SS_NOPREFIX
        } else {
            WS_CHILD | WS_VISIBLE | SS_CENTER | SS_NOPREFIX
        };

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
            Some(Self { hwnd })
        } else {
            None
        }
    }

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
