//! Zero-Allocation GDI Drawing Helpers & Double-Buffering BackBuffer
//!
//! Cung cấp các hàm vẽ đồ họa tối ưu hóa:
//! - Sử dụng trực tiếp Stock Objects (`DC_BRUSH`, `DC_PEN`), không tạo và không rò rỉ GDI Handle.
//! - Bộ đệm vẽ hai lớp (`BackBuffer`) tự động BitBlt khi Drop, triệt tiêu hiện tượng giật nháy màn hình.

use super::ffi::*;
use super::types::*;

/// Fills `rc` with a solid color.
pub fn fill_rect_color(hdc: isize, rc: &RECT, color: u32) {
    unsafe {
        SetDCBrushColor(hdc, color);
        FillRect(hdc, rc, GetStockObject(DC_BRUSH));
    }
}

#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// 1px straight line from (x1, y1) to (x2, y2).
pub fn draw_line(hdc: isize, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
    unsafe {
        SetDCPenColor(hdc, color);
        let old_pen = SelectObject(hdc, GetStockObject(DC_PEN));
        MoveToEx(hdc, x1, y1, std::ptr::null_mut());
        LineTo(hdc, x2, y2);
        SelectObject(hdc, old_pen);
    }
}

/// Off-screen buffer for flicker-free painting. Draw into [`BackBuffer::hdc`];
/// the content is blitted to the target DC when the buffer is dropped.
pub struct BackBuffer {
    target: isize,
    mem: isize,
    bmp: isize,
    old_bmp: isize,
    width: i32,
    height: i32,
}

impl BackBuffer {
    pub fn new(target: isize, width: i32, height: i32) -> Option<Self> {
        unsafe {
            let mem = CreateCompatibleDC(target);
            if mem == 0 {
                return None;
            }
            let bmp = CreateCompatibleBitmap(target, width.max(1), height.max(1));
            if bmp == 0 {
                DeleteDC(mem);
                return None;
            }
            let old_bmp = SelectObject(mem, bmp);
            Some(Self {
                target,
                mem,
                bmp,
                old_bmp,
                width,
                height,
            })
        }
    }

    #[inline]
    pub fn hdc(&self) -> isize {
        self.mem
    }
}

impl Drop for BackBuffer {
    fn drop(&mut self) {
        unsafe {
            BitBlt(
                self.target,
                0,
                0,
                self.width,
                self.height,
                self.mem,
                0,
                0,
                SRCCOPY,
            );
            SelectObject(self.mem, self.old_bmp);
            DeleteObject(self.bmp);
            DeleteDC(self.mem);
        }
    }
}