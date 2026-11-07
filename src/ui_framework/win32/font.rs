//! Application Typography & Font Management
//!
//! Khởi tạo font chữ hệ thống Segoe UI chuẩn DPI cao cho toàn bộ giao diện.

use super::ffi::*;
use super::styling::to_wide;
use super::types::*;
use std::sync::OnceLock;

static IS_WIN11: OnceLock<bool> = OnceLock::new();

fn is_windows_11() -> bool {
    *IS_WIN11.get_or_init(|| {
        #[repr(C)]
        struct OsVersionInfoW {
            dw_os_version_info_size: u32,
            dw_major_version: u32,
            dw_minor_version: u32,
            dw_build_number: u32,
            dw_platform_id: u32,
            sz_csd_version: [u16; 128],
        }
        unsafe {
            let ntdll = GetModuleHandleW(to_wide("ntdll.dll").as_ptr());
            if ntdll != 0 {
                let func: Option<unsafe extern "system" fn(*mut OsVersionInfoW) -> i32> =
                    std::mem::transmute(GetProcAddress(ntdll, b"RtlGetVersion\0".as_ptr()));
                if let Some(rtl_get_version) = func {
                    let mut info: OsVersionInfoW = std::mem::zeroed();
                    info.dw_os_version_info_size = std::mem::size_of::<OsVersionInfoW>() as u32;
                    if rtl_get_version(&mut info) == 0 {
                        return info.dw_build_number >= 22000;
                    }
                }
            }
        }
        false
    })
}

/// Creates modern Segoe UI font matching Windows 10/11 standards at a specific DPI
pub fn create_app_font_for_dpi(point_size: i32, weight: i32, dpi: i32) -> isize {
    let face = if is_windows_11() {
        "Segoe UI Variable Text"
    } else {
        "Segoe UI"
    };
    let font_name = to_wide(face);
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
