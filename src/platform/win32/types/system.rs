//! Windows System Structures, Coordinates & Process Data Types
//!
//! Định nghĩa các cấu trúc hình học (POINT, RECT), thông điệp Message Loop (MSG),
//! thông tin con trỏ ứng dụng (GUITHREADINFO) và thời gian hệ thống (SystemTime).

/// Quyền truy cập tối thiểu an toàn để đọc thông tin tiến trình
pub const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

/// Điểm tọa độ màn hình (x, y)
#[repr(C)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

/// Vùng hình chữ nhật (trái, trên, phải, dưới)
#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Thông điệp gửi trong vòng lặp Win32 Message Pump (`GetMessageW` / `DispatchMessageW`)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MSG {
    pub hwnd: isize,
    pub message: u32,
    pub w_param: usize,
    pub l_param: isize,
    pub time: u32,
    pub pt: POINT,
}

/// Thông tin về trạng thái GUI của một luồng ứng dụng (Active, Focus, Caret vị trí gõ)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUITHREADINFO {
    pub cb_size: u32,
    pub flags: u32,
    pub hwnd_active: isize,
    pub hwnd_focus: isize,
    pub hwnd_capture: isize,
    pub hwnd_menu_owner: isize,
    pub hwnd_move_size: isize,
    pub hwnd_caret: isize,
    pub rc_caret: RECT,
}

impl Default for GUITHREADINFO {
    fn default() -> Self {
        Self {
            cb_size: std::mem::size_of::<GUITHREADINFO>() as u32,
            flags: 0,
            hwnd_active: 0,
            hwnd_focus: 0,
            hwnd_capture: 0,
            hwnd_menu_owner: 0,
            hwnd_move_size: 0,
            hwnd_caret: 0,
            rc_caret: RECT::default(),
        }
    }
}

/// Cấu trúc ngày giờ hệ thống Windows chính xác đến mili-giây
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

