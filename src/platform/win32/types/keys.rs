//! Virtual-Key Codes (Mã định danh phím ảo của Windows)
//!
//! Danh mục các mã phím phần cứng thường gặp trong xử lý gõ phím tiếng Việt,
//! phím bổ trợ (Shift, Ctrl, Alt, Win) và các phím điều hướng con trỏ.

// --- 1. Phím điều khiển soạn thảo cơ bản ---
pub const VK_BACK: u32 = 0x08;      // Phím Backspace (xóa lùi)
pub const VK_TAB: u32 = 0x09;       // Phím Tab
pub const VK_RETURN: u32 = 0x0D;    // Phím Enter (xuống dòng)
pub const VK_ESCAPE: u32 = 0x1B;    // Phím Esc (hủy trạng thái)
pub const VK_SPACE: u32 = 0x20;     // Phím Spacebar (dấu cách)

// --- 2. Phím bổ trợ & Chế độ gõ (Modifiers & Toggles) ---
pub const VK_SHIFT: i32 = 0x10;     // Phím Shift
pub const VK_CONTROL: i32 = 0x11;   // Phím Ctrl
pub const VK_MENU: i32 = 0x12;      // Phím Alt
pub const VK_CAPITAL: i32 = 0x14;   // Phím Caps Lock
pub const VK_LWIN: i32 = 0x5B;      // Phím Windows trái
pub const VK_RWIN: i32 = 0x5C;      // Phím Windows phải

// --- 3. Phím điều hướng & Chỉnh sửa (Navigation & Editing) ---
pub const VK_PRIOR: u32 = 0x21;     // Page Up
pub const VK_NEXT: u32 = 0x22;      // Page Down
pub const VK_END: u32 = 0x23;       // End
pub const VK_HOME: u32 = 0x24;      // Home
pub const VK_LEFT: u32 = 0x25;      // Mũi tên trái
pub const VK_UP: u32 = 0x26;        // Mũi tên lên
pub const VK_RIGHT: u32 = 0x27;     // Mũi tên phải
pub const VK_DOWN: u32 = 0x28;      // Mũi tên xuống
pub const VK_INSERT: u32 = 0x2D;    // Insert
pub const VK_DELETE: u32 = 0x2E;    // Delete (xóa tới)

