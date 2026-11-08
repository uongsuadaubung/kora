//! GDI+ Native Flat C API FFI Bindings & Process-Wide Lifecycle Management
//!
//! Khai báo các hàm C Flat API từ `gdiplus.dll` để hỗ trợ khử răng cưa (Antialiasing),
//! bo góc mượt mà (Smooth RoundRect) và đổ bóng mờ bán trong suốt (Alpha Drop Shadow).
//!
//! Thư viện `gdiplus.dll` được tích hợp sẵn 100% trong mọi phiên bản Windows từ Windows XP
//! đến Windows 11, đảm bảo Kora duy trì 0 external dependencies và dung lượng binary ~3 MB.

use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(C)]
pub struct GdiplusStartupInput {
    pub gdiplus_version: u32,
    pub debug_event_callback: usize,
    pub suppress_background_thread: i32,
    pub suppress_external_codecs: i32,
}

impl Default for GdiplusStartupInput {
    fn default() -> Self {
        Self {
            gdiplus_version: 1,
            debug_event_callback: 0,
            suppress_background_thread: 0,
            suppress_external_codecs: 0,
        }
    }
}

pub const SMOOTHING_MODE_DEFAULT: i32 = 0;
pub const SMOOTHING_MODE_HIGH_SPEED: i32 = 1;
pub const SMOOTHING_MODE_HIGH_QUALITY: i32 = 2;
pub const SMOOTHING_MODE_NONE: i32 = 3;
pub const SMOOTHING_MODE_ANTI_ALIAS: i32 = 4;

pub const PIXEL_OFFSET_MODE_DEFAULT: i32 = 0;
pub const PIXEL_OFFSET_MODE_HIGH_SPEED: i32 = 1;
pub const PIXEL_OFFSET_MODE_HIGH_QUALITY: i32 = 2;
pub const PIXEL_OFFSET_MODE_NONE: i32 = 3;
pub const PIXEL_OFFSET_MODE_HALF: i32 = 4;

pub const FILL_MODE_ALTERNATE: i32 = 0;
pub const FILL_MODE_WINDING: i32 = 1;

pub const UNIT_PIXEL: i32 = 2;

#[link(name = "gdiplus")]
unsafe extern "system" {
    pub fn GdiplusStartup(
        token: *mut usize,
        input: *const GdiplusStartupInput,
        output: *mut std::ffi::c_void,
    ) -> i32;
    pub fn GdiplusShutdown(token: usize);

    pub fn GdipCreateFromHDC(hdc: isize, graphics: *mut isize) -> i32;
    pub fn GdipDeleteGraphics(graphics: isize) -> i32;
    pub fn GdipSetSmoothingMode(graphics: isize, mode: i32) -> i32;
    pub fn GdipSetPixelOffsetMode(graphics: isize, mode: i32) -> i32;

    pub fn GdipCreateSolidFill(color: u32, brush: *mut isize) -> i32;
    pub fn GdipDeleteBrush(brush: isize) -> i32;

    pub fn GdipCreatePen1(color: u32, width: f32, unit: i32, pen: *mut isize) -> i32;
    pub fn GdipDeletePen(pen: isize) -> i32;

    pub fn GdipCreatePath(fill_mode: i32, path: *mut isize) -> i32;
    pub fn GdipDeletePath(path: isize) -> i32;
    pub fn GdipResetPath(path: isize) -> i32;
    pub fn GdipAddPathArc(
        path: isize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        start_angle: f32,
        sweep_angle: f32,
    ) -> i32;
    pub fn GdipAddPathLine(path: isize, x1: f32, y1: f32, x2: f32, y2: f32) -> i32;
    pub fn GdipClosePathFigure(path: isize) -> i32;

    pub fn GdipFillPath(graphics: isize, brush: isize, path: isize) -> i32;
    pub fn GdipDrawPath(graphics: isize, pen: isize, path: isize) -> i32;

    pub fn GdipFillEllipse(
        graphics: isize,
        brush: isize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;
    pub fn GdipDrawEllipse(
        graphics: isize,
        pen: isize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;

    pub fn GdipFillRectangle(
        graphics: isize,
        brush: isize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;
    pub fn GdipDrawRectangle(
        graphics: isize,
        pen: isize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> i32;
}

static GDIP_TOKEN: AtomicUsize = AtomicUsize::new(0);

/// Đảm bảo GDI+ runtime đã được khởi tạo trong process.
/// Được gọi an toàn từ bất kỳ đâu, chỉ thực thi 1 lần duy nhất.
pub fn ensure_gdiplus_initialized() -> bool {
    if GDIP_TOKEN.load(Ordering::Acquire) != 0 {
        return true;
    }
    let input = GdiplusStartupInput::default();
    let mut token: usize = 0;
    let status = unsafe { GdiplusStartup(&mut token, &input, std::ptr::null_mut()) };
    if status == 0 && token != 0 {
        GDIP_TOKEN.store(token, Ordering::Release);
        true
    } else {
        false
    }
}

/// Giải phóng GDI+ runtime khi ứng dụng thoát.
pub fn shutdown_gdiplus() {
    let token = GDIP_TOKEN.swap(0, Ordering::AcqRel);
    if token != 0 {
        unsafe {
            GdiplusShutdown(token);
        }
    }
}

/// Chuyển đổi màu Win32 COLORREF (`0x00BBGGRR`) sang định dạng GDI+ ARGB (`0xAARRGGBB`).
#[inline]
pub fn colorref_to_argb(color: u32, alpha: u8) -> u32 {
    let r = color & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = (color >> 16) & 0xFF;
    ((alpha as u32) << 24) | (r << 16) | (g << 8) | b
}
