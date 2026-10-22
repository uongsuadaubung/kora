//! Application Typography & Font Management
//!
//! Khởi tạo font chữ hệ thống Segoe UI chuẩn DPI cao cho toàn bộ giao diện.

use super::ffi::*;
use super::styling::to_wide;
use super::types::*;

/// Creates modern Segoe UI font matching Windows 10/11 standards at a specific DPI
pub fn create_app_font_for_dpi(point_size: i32, weight: i32, dpi: i32) -> isize {
    let font_name = to_wide("Segoe UI");
    let dpi = if dpi <= 0 { 96 } else { dpi };
    let height = -((point_size * dpi + 36) / 72);
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            0,
            0,
            CLEARTYPE_QUALITY,
            0,
            font_name.as_ptr(),
        )
    }
}

/// Creates modern Segoe UI font matching Windows 10/11 standards
pub fn create_app_font(point_size: i32, weight: i32) -> isize {
    create_app_font_for_dpi(point_size, weight, 96)
}
