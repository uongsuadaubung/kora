//! Native Win32 FFI Bindings, GDI Types, and Platform Helper Functions
//!
//! Module được tách thành các thành phần chuyên biệt:
//! - [`types`]: Structs, handles và các hằng số thông điệp UI/Windows.
//! - [`ffi`]: Khai báo hàm liên kết động extern Win32 API.
//! - [`draw`]: Hàm vẽ GDI tối ưu hóa không cấp phát & BackBuffer chống nháy.
//! - [`styling`]: Windows 11 Styling (DWM dark mode, bo tròn góc, đăng ký cửa sổ).
//! - [`font`]: Quản lý font chữ ứng dụng Segoe UI.

pub mod draw;
pub mod ffi;
pub mod font;
pub mod styling;
pub mod types;

pub use draw::*;
pub use ffi::*;
pub use font::*;
pub use styling::*;
pub use types::*;

