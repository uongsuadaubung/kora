//! Double-Buffering Off-Screen BackBuffer & GDI Fast Fill Primitives
//!
//! Cung cấp hạ tầng bộ đệm khung hình chống giật nháy:
//! - [`BackBuffer`]: Tạo Device Context phụ ngoài màn hình và tự động `BitBlt` khi Drop.
//! - Các tiện ích GDI cơ bản (`fill_rect_color`, `rgb`).

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