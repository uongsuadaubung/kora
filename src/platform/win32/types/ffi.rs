//! Native Windows API External Declarations (FFI)
//!
//! Khai báo các hàm gọi trực tiếp vào các thư viện liên kết động (.dll) của hệ điều hành Windows:
//! - `user32.dll`: Hook bàn phím/chuột, Message Pump, SendInput, Window Manager
//! - `kernel32.dll`: Quản lý tiến trình, bộ nhớ làm việc (Working Set), đồng hồ thời gian
//! - `imm32.dll`: Giao diện tương thích với bộ gõ quốc tế (IME Nhật/Trung/Hàn)
//! - `wtsapi32.dll`: Lắng nghe sự kiện đăng nhập / khóa màn hình Windows

use super::input::INPUT;
use super::system::{GUITHREADINFO, MSG, SystemTime};

/// Định dạng con trỏ hàm Callback cho Windows Hook toàn cục
pub type HOOKPROC = unsafe extern "system" fn(code: i32, w_param: usize, l_param: isize) -> isize;

/// Định dạng con trỏ hàm Callback cho Windows Event Hook (Chuyển cửa sổ Foreground)
pub type WINEVENTPROC = unsafe extern "system" fn(
    hWinEventHook: isize,
    event: u32,
    hwnd: isize,
    idObject: i32,
    idChild: i32,
    idEventThread: u32,
    dwmsEventTime: u32,
);

// ============================================================================
// 1. user32.dll — Giao diện Người dùng, Hook & Nhập xuất Bàn phím / Chuột
// ============================================================================
pub const WH_KEYBOARD_LL: i32 = 13;
pub const WH_MOUSE_LL: i32 = 14;
pub const GA_PARENT: u32 = 1;
pub const GA_ROOT: u32 = 2;
pub const GA_ROOTOWNER: u32 = 3;

#[link(name = "user32")]
unsafe extern "system" {
    // --- Hook bàn phím & chuột toàn cục ---
    pub fn SetWindowsHookExW(idHook: i32, lpfn: HOOKPROC, hmod: isize, dwThreadId: u32) -> isize;
    pub fn UnhookWindowsHookEx(hhk: isize) -> i32;
    pub fn CallNextHookEx(hhk: isize, nCode: i32, wParam: usize, lParam: isize) -> isize;

    // --- Vòng lặp thông điệp (Message Loop) ---
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: isize, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    pub fn TranslateMessage(lpMsg: *const MSG) -> i32;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> isize;

    // --- Phát phím tổng hợp (Synthetic Key Injection) ---
    pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;

    // --- Truy vấn trạng thái phần cứng bàn phím ---
    pub fn GetKeyState(nVirtKey: i32) -> i16;
    pub fn GetAsyncKeyState(vKey: i32) -> i16;
    pub fn GetKeyboardState(lpKeyState: *mut u8) -> i32;
    pub fn ToUnicode(
        wVirtKey: u32,
        wScanCode: u32,
        lpKeyState: *const u8,
        pwszBuff: *mut u16,
        cchBuff: i32,
        wFlags: u32,
    ) -> i32;

    // --- Quản lý cửa sổ & Luồng ứng dụng ---
    pub fn GetForegroundWindow() -> isize;
    pub fn GetAncestor(hWnd: isize, gaFlags: u32) -> isize;
    pub fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    pub fn GetGUIThreadInfo(idThread: u32, pgui: *mut GUITHREADINFO) -> i32;
    pub fn GetClassNameW(hWnd: isize, lpClassName: *mut u16, nMaxCount: i32) -> i32;
    pub fn MessageBoxW(hWnd: isize, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
    pub fn PostMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> i32;

    // --- Gửi thông điệp có Timeout an toàn (Tránh đơ khi app đích bị treo) ---
    pub fn SendMessageTimeoutW(
        hWnd: isize,
        Msg: u32,
        wParam: usize,
        lParam: isize,
        fuFlags: u32,
        uTimeout: u32,
        lpdwResult: *mut usize,
    ) -> isize;

    // --- Bắt sự kiện chuyển cửa sổ hệ thống ---
    pub fn SetWinEventHook(
        eventMin: u32,
        eventMax: u32,
        hmodWinEventProc: isize,
        pfnWinEventProc: Option<WINEVENTPROC>,
        idProcess: u32,
        idThread: u32,
        dwFlags: u32,
    ) -> isize;
    pub fn UnhookWinEvent(hWinEventHook: isize) -> i32;
}

// ============================================================================
// 2. imm32.dll — Quản lý Bộ gõ Quốc tế (IME: Nhật, Trung, Hàn)
// ============================================================================
#[link(name = "imm32")]
unsafe extern "system" {
    /// Lấy Handle của cửa sổ bộ gõ IME mặc định của ứng dụng đang hoạt động
    pub fn ImmGetDefaultIMEWnd(hWnd: isize) -> isize;
}

// ============================================================================
// 3. wtsapi32.dll — Quản lý Phiên làm việc Windows (Lock, Unlock, Sleep)
// ============================================================================
#[link(name = "wtsapi32")]
unsafe extern "system" {
    /// Đăng ký nhận thông báo khi người dùng khóa màn hình (Win+L), Logoff hoặc chuyển phiên
    pub fn WTSRegisterSessionNotification(hWnd: isize, dwFlags: u32) -> i32;
    pub fn WTSUnRegisterSessionNotification(hWnd: isize) -> i32;
}

// ============================================================================
// 4. kernel32.dll — Nhân Hệ điều hành: Quản lý Tiến trình, RAM & Thời gian
// ============================================================================
#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> isize;
    pub fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
    pub fn CloseHandle(hObject: isize) -> i32;
    pub fn QueryFullProcessImageNameW(
        hProcess: isize,
        dwFlags: u32,
        lpExeName: *mut u16,
        lpdwSize: *mut u32,
    ) -> i32;
    pub fn GetCurrentProcess() -> isize;

    /// Giải phóng Working Set RAM khi ẩn cửa sổ (đưa mức tiêu thụ RAM về tối thiểu)
    pub fn SetProcessWorkingSetSize(
        hProcess: isize,
        dwMinimumWorkingSetSize: usize,
        dwMaximumWorkingSetSize: usize,
    ) -> i32;

    /// Đọc ngày giờ hiện tại của hệ thống chính xác đến mili-giây
    pub fn GetLocalTime(lpSystemTime: *mut SystemTime);
}

