//! Win32 System Types, Constants, Input Structures & FFI Declarations
//!
//! Module này được tách thành các nhóm chuyên biệt theo đúng nhiệm vụ:
//! - [`keys`]: Mã phím ảo phần cứng (Virtual-Key codes: Backspace, Enter, Modifiers, v.v.)
//! - [`messages`]: Mã thông điệp Windows (Keyboard, Mouse, IME, Session, Power, Dialogs)
//! - [`input`]: Cấu trúc phát phím / nhận phím ảo (SendInput, Hook structs, ExtraInfo)
//! - [`system`]: Cấu trúc hệ thống và hình học (POINT, RECT, MSG, GUITHREADINFO, SystemTime)
//! - [`ffi`]: Khai báo hàm gọi Windows API (user32, kernel32, imm32, wtsapi32)

pub mod ffi;
pub mod input;
pub mod keys;
pub mod messages;
pub mod system;

// Re-export toàn bộ ra ngoài để tương thích ngược hoàn hảo với `use types::*;`
pub use ffi::*;
pub use input::*;
pub use keys::*;
pub use messages::*;
pub use system::*;
