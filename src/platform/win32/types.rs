//! Win32 FFI Structures, Constants & System Declarations

pub const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

// Win32 Constants
pub const WH_KEYBOARD_LL: i32 = 13;
pub const WM_KEYDOWN: usize = 0x0100;
pub const WM_KEYUP: usize = 0x0101;
pub const WM_SYSKEYDOWN: usize = 0x0104;
pub const WM_SYSKEYUP: usize = 0x0105;

pub const INPUT_KEYBOARD: u32 = 1;
pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
pub const KEYEVENTF_KEYUP: u32 = 0x0002;
pub const KEYEVENTF_UNICODE: u32 = 0x0004;

pub const VK_BACK: u32 = 0x08;
pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SHIFT: i32 = 0x10;
pub const VK_CONTROL: i32 = 0x11;
pub const VK_MENU: i32 = 0x12; // Alt
pub const VK_CAPITAL: i32 = 0x14; // Caps Lock
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_SPACE: u32 = 0x20;
pub const VK_PRIOR: u32 = 0x21; // Page Up
pub const VK_NEXT: u32 = 0x22; // Page Down
pub const VK_END: u32 = 0x23;
pub const VK_HOME: u32 = 0x24;
pub const VK_LEFT: u32 = 0x25;
pub const VK_UP: u32 = 0x26;
pub const VK_RIGHT: u32 = 0x27;
pub const VK_DOWN: u32 = 0x28;
pub const VK_INSERT: u32 = 0x2D;
pub const VK_DELETE: u32 = 0x2E;
pub const VK_LWIN: i32 = 0x5B;
pub const VK_RWIN: i32 = 0x5C;

pub const WH_MOUSE_LL: i32 = 14;
pub const WM_LBUTTONDOWN: usize = 0x0201;
pub const WM_RBUTTONDOWN: usize = 0x0204;
pub const WM_MBUTTONDOWN: usize = 0x0207;

pub const MB_OK: u32 = 0x00000000;
pub const MB_YESNO: u32 = 0x00000004;
pub const MB_ICONINFORMATION: u32 = 0x00000040;
pub const MB_ICONWARNING: u32 = 0x00000030;
pub const MB_ICONERROR: u32 = 0x00000010;
pub const IDYES: i32 = 6;
pub const IDNO: i32 = 7;
pub const IDOK: i32 = 1;

/// Magic identifier to tag our own synthetic input events and prevent infinite hook loops
pub const MAGIC_EXTRA_INFO: usize = 0x4B4F5241; // "KORA" in ASCII

// Win32 FFI Structures
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KBDLLHOOKSTRUCT {
    pub vk_code: u32,
    pub scan_code: u32,
    pub flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KEYBDINPUT {
    pub w_vk: u16,
    pub w_scan: u16,
    pub dw_flags: u32,
    pub time: u32,
    pub dw_extra_info: usize,
}

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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HARDWAREINPUT {
    pub u_msg: u32,
    pub w_param_l: u16,
    pub w_param_h: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union INPUT_UNION {
    pub mi: MOUSEINPUT,
    pub ki: KEYBDINPUT,
    pub hi: HARDWAREINPUT,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct INPUT {
    pub r#type: u32,
    pub u: INPUT_UNION,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

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

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

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

pub type HOOKPROC = unsafe extern "system" fn(code: i32, w_param: usize, l_param: isize) -> isize;

// Win32 API function signatures linked directly to user32.dll and kernel32.dll
#[link(name = "user32")]
unsafe extern "system" {
    pub fn SetWindowsHookExW(idHook: i32, lpfn: HOOKPROC, hmod: isize, dwThreadId: u32) -> isize;
    pub fn UnhookWindowsHookEx(hhk: isize) -> i32;
    pub fn CallNextHookEx(hhk: isize, nCode: i32, wParam: usize, lParam: isize) -> isize;
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: isize, wMsgFilterMin: u32, wMsgFilterMax: u32)
    -> i32;
    pub fn TranslateMessage(lpMsg: *const MSG) -> i32;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
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
    pub fn GetForegroundWindow() -> isize;
    pub fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    pub fn GetGUIThreadInfo(idThread: u32, pgui: *mut GUITHREADINFO) -> i32;
    pub fn GetClassNameW(hWnd: isize, lpClassName: *mut u16, nMaxCount: i32) -> i32;
    pub fn MessageBoxW(hWnd: isize, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
    pub fn PostMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> i32;
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

pub const WM_POWERBROADCAST: u32 = 0x0218;
pub const PBT_APMSUSPEND: usize = 0x0004;
pub const PBT_APMRESUMESUSPEND: usize = 0x0007;
pub const PBT_APMRESUMEAUTOMATIC: usize = 0x0012;

pub const WM_WTSSESSION_CHANGE: u32 = 0x02B1;
pub const WTS_CONSOLE_CONNECT: usize = 0x1;
pub const WTS_SESSION_LOGON: usize = 0x5;
pub const WTS_SESSION_LOCK: usize = 0x7;
pub const WTS_SESSION_UNLOCK: usize = 0x8;
pub const NOTIFY_FOR_THIS_SESSION: u32 = 0;

#[link(name = "wtsapi32")]
unsafe extern "system" {
    pub fn WTSRegisterSessionNotification(hWnd: isize, dwFlags: u32) -> i32;
    pub fn WTSUnRegisterSessionNotification(hWnd: isize) -> i32;
}

pub const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
pub const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
pub const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;

pub type WINEVENTPROC = unsafe extern "system" fn(
    hWinEventHook: isize,
    event: u32,
    hwnd: isize,
    idObject: i32,
    idChild: i32,
    idEventThread: u32,
    dwmsEventTime: u32,
);

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
    pub fn SetProcessWorkingSetSize(
        hProcess: isize,
        dwMinimumWorkingSetSize: usize,
        dwMaximumWorkingSetSize: usize,
    ) -> i32;
    pub fn GetLocalTime(lpSystemTime: *mut SystemTime);
}

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
