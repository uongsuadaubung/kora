//! Low-Level Keyboard/Mouse Hook & Synthetic Input Structures
//!
//! Các cấu trúc và cờ điều khiển để nhận diện phím bấm phần cứng từ Windows Hook
//! và phát phím ảo qua Windows `SendInput` API.

/// Loại sự kiện gửi vào SendInput: Bàn phím
pub const INPUT_KEYBOARD: u32 = 1;

/// Cờ gửi phím SendInput
pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001; // Phím mở rộng (mũi tên, numpad enter, v.v.)
pub const KEYEVENTF_KEYUP: u32 = 0x0002; // Nhả phím (Key Up)
pub const KEYEVENTF_UNICODE: u32 = 0x0004; // Ký tự Unicode UTF-16

/// Mã nhận diện độc quyền của Kora gắn vào dwExtraInfo để phân biệt phím ảo do Kora phát
/// với phím phần cứng do người dùng bấm, triệt tiêu đệ quy vòng lặp vô tận.
pub const MAGIC_EXTRA_INFO: usize = 0x4B4F5241; // Chuỗi ký tự ASCII "KORA"

/// Dữ liệu chi tiết gửi về từ Low-Level Keyboard Hook (`WH_KEYBOARD_LL`)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KBDLLHOOKSTRUCT {
    pub vk_code: u32,
    pub scan_code: u32,
    pub flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

/// Cấu trúc mô tả một sự kiện gõ bàn phím phát đi qua `SendInput`
#[repr(C)]
#[derive(Copy, Clone)]
pub struct KEYBDINPUT {
    pub w_vk: u16,
    pub w_scan: u16,
    pub dw_flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

/// Cấu trúc mô tả sự kiện chuột (di chuyển, click)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MOUSEINPUT {
    pub dx: i32,
    pub dy: i32,
    pub mouse_data: u32,
    pub dw_flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

/// Cấu trúc mô tả sự kiện phần cứng chung
#[repr(C)]
#[derive(Copy, Clone)]
pub struct HARDWAREINPUT {
    pub u_msg: u32,
    pub w_param_l: u16,
    pub w_param_h: u16,
}

/// Union chứa nội dung của 1 trong 3 loại input: Chuột, Bàn phím, hoặc Phần cứng
#[repr(C)]
#[derive(Copy, Clone)]
pub union INPUT_UNION {
    pub mi: MOUSEINPUT,
    pub ki: KEYBDINPUT,
    pub hi: HARDWAREINPUT,
}

/// Cấu trúc đầu vào tổng hợp truyền vào hàm `SendInput` của Windows
#[repr(C)]
#[derive(Copy, Clone)]
pub struct INPUT {
    pub r#type: u32,
    pub u: INPUT_UNION,
}
