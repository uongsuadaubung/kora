//! Windows Messages & System Event Constants
//!
//! Định nghĩa các mã thông điệp sự kiện của Windows gửi đến ứng dụng:
//! sự kiện phím, chuột, quản lý nguồn điện (Sleep/Wake), khóa phiên làm việc (Lock/Unlock),
//! chuyển cửa sổ (Foreground) và hộp thoại thông báo.

// --- 1. Sự kiện Bàn phím (Keyboard Messages) ---
pub const WM_KEYDOWN: usize = 0x0100;
pub const WM_KEYUP: usize = 0x0101;
pub const WM_SYSKEYDOWN: usize = 0x0104;
pub const WM_SYSKEYUP: usize = 0x0105;

// --- 2. Sự kiện Chuột (Mouse Messages) ---
pub const WM_LBUTTONDOWN: usize = 0x0201;
pub const WM_RBUTTONDOWN: usize = 0x0204;
pub const WM_MBUTTONDOWN: usize = 0x0207;

// --- 3. Sự kiện Quản lý nguồn điện (Power Broadcast & Sleep/Wake) ---
pub const WM_POWERBROADCAST: u32 = 0x0218;
pub const PBT_APMSUSPEND: usize = 0x0004;
pub const PBT_APMRESUMESUSPEND: usize = 0x0007;
pub const PBT_APMRESUMEAUTOMATIC: usize = 0x0012;

// --- 4. Sự kiện Phiên làm việc Windows (WTS Session: Lock/Unlock Screen) ---
pub const WM_WTSSESSION_CHANGE: u32 = 0x02B1;
pub const WTS_CONSOLE_CONNECT: usize = 0x1;
pub const WTS_SESSION_LOGON: usize = 0x5;
pub const WTS_SESSION_LOCK: usize = 0x7;
pub const WTS_SESSION_UNLOCK: usize = 0x8;
pub const NOTIFY_FOR_THIS_SESSION: u32 = 0;

// --- 5. Sự kiện Chuyển đổi Cửa sổ Foreground (WinEvent Hook) ---
pub const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
pub const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
pub const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;

// --- 6. Sự kiện Điều khiển IME Đa ngôn ngữ (Japanese/Chinese IME) ---
pub const WM_IME_CONTROL: u32 = 0x0283;
pub const IMC_GETOPENSTATUS: usize = 0x0005;
pub const SMTO_ABORTIFHUNG: u32 = 0x0002;

// --- 7. Cờ Hộp thoại Thông báo (MessageBox Constants) ---
pub const MB_OK: u32 = 0x00000000;
pub const MB_YESNO: u32 = 0x00000004;
pub const MB_ICONINFORMATION: u32 = 0x00000040;
pub const MB_ICONWARNING: u32 = 0x00000030;
pub const MB_ICONERROR: u32 = 0x00000010;
pub const IDYES: i32 = 6;
pub const IDNO: i32 = 7;
pub const IDOK: i32 = 1;

