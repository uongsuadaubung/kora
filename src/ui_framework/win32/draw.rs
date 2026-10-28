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

/// Draws a 1px border around `rc`.
pub fn frame_rect_color(hdc: isize, rc: &RECT, color: u32) {
    unsafe {
        SetDCBrushColor(hdc, color);
        FrameRect(hdc, rc, GetStockObject(DC_BRUSH));
    }
}

/// Filled rounded rectangle with a 1px border (`border = None` → no outline).
/// `radius` is the corner ellipse diameter as passed to `RoundRect`.
pub fn round_rect(hdc: isize, rc: &RECT, radius: i32, fill: u32, border: Option<u32>) {
    unsafe {
        SetDCBrushColor(hdc, fill);
        let pen = match border {
            Some(c) => {
                SetDCPenColor(hdc, c);
                GetStockObject(DC_PEN)
            }
            None => GetStockObject(NULL_PEN),
        };
        let old_brush = SelectObject(hdc, GetStockObject(DC_BRUSH));
        let old_pen = SelectObject(hdc, pen);
        let extra = if border.is_none() { 1 } else { 0 };
        RoundRect(
            hdc,
            rc.left,
            rc.top,
            rc.right + extra,
            rc.bottom + extra,
            radius,
            radius,
        );
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
    }
}

/// Outline-only rounded rectangle.
pub fn round_rect_outline(hdc: isize, rc: &RECT, radius: i32, border: u32) {
    unsafe {
        SetDCPenColor(hdc, border);
        let old_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH));
        let old_pen = SelectObject(hdc, GetStockObject(DC_PEN));
        RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
    }
}

#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// Draws a soft multi-ring elevation drop shadow around `rc`.
/// `radius` is the corner radius of the inner box.
/// `elevation` is the blur spread in pixels (typically 3 to 6).
pub fn draw_elevation_shadow(hdc: isize, rc: &RECT, radius: i32, elevation: i32, is_dark: bool) {
    if elevation <= 0 {
        return;
    }
    for i in (1..=elevation).rev() {
        let shadow_rc = RECT {
            left: rc.left - i,
            top: rc.top - (i / 2).max(1) + 1,
            right: rc.right + i,
            bottom: rc.bottom + i + 1,
        };
        let factor = 1.0 - (i as f32 / (elevation + 1) as f32);
        let color = if is_dark {
            let v = (18.0 * factor) as u8;
            rgb(v, v, v)
        } else {
            let r = (248.0 - 45.0 * factor) as u8;
            let g = (249.0 - 42.0 * factor) as u8;
            let b = (250.0 - 38.0 * factor) as u8;
            rgb(r, g, b)
        };
        round_rect_outline(hdc, &shadow_rc, radius + i, color);
    }
}

/// Filled ellipse (outline in the same color).
pub fn fill_ellipse(hdc: isize, rc: &RECT, color: u32) {
    unsafe {
        SetDCBrushColor(hdc, color);
        SetDCPenColor(hdc, color);
        let old_brush = SelectObject(hdc, GetStockObject(DC_BRUSH));
        let old_pen = SelectObject(hdc, GetStockObject(DC_PEN));
        Ellipse(hdc, rc.left, rc.top, rc.right, rc.bottom);
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
    }
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
