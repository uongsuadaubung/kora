//! Native Win32 FFI Bindings, GDI Types, and Platform Helper Functions
//!
//! Module được tách thành các thành phần chuyên biệt:
//! - [`types`]: Structs, handles và các hằng số thông điệp UI/Windows (`OCM_*` reflection).
//! - [`ffi`]: Khai báo hàm liên kết động extern Win32 API.
//! - [`buffer`]: Bộ đệm khung hình `BackBuffer` chống nháy và các tiện ích GDI cơ bản.
//! - [`styling`]: Windows 11 Styling (DWM dark mode, bo tròn góc, đăng ký cửa sổ).
//! - [`font`]: Quản lý font chữ ứng dụng Segoe UI.
//! - [`ctlcolor`]: Xử lý tự động màu nền/chữ `WM_CTLCOLOR*` và con trỏ bàn tay `WM_SETCURSOR`.

pub mod buffer;
pub mod ctlcolor;
pub mod ffi;
pub mod font;
pub mod gdiplus;
pub mod styling;
pub mod types;

pub use buffer::*;
pub use ffi::*;
pub use font::*;
pub use gdiplus::*;
pub use styling::*;
pub use types::*;