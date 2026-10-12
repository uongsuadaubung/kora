//! Win32 UI Types, Structures, and Constants
//!
//! Định nghĩa các handle kiểu bí danh (HWND, HDC...), các cấu trúc dữ liệu GDI & Common Controls,
//! cùng toàn bộ mã hằng số thông điệp UI và cờ kiểu dáng cửa sổ (Styles & Flags).

pub type HWND = isize;
pub type HDC = isize;
pub type HFONT = isize;
pub type HBRUSH = isize;
pub type HICON = isize;
pub type HCURSOR = isize;
pub type HMENU = isize;
pub type HHOOK = isize;
pub type HINSTANCE = isize;
pub type HMODULE = isize;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: Option<unsafe extern "system" fn(isize, u32, usize, isize) -> isize>,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: isize,
    pub h_icon: isize,
    pub h_cursor: isize,
    pub h_br_background: isize,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
    pub h_icon_sm: isize,
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
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

#[repr(C)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub struct NMHDR {
    pub hwnd_from: isize,
    pub id_from: usize,
    pub code: u32,
}

#[repr(C)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub struct NMCUSTOMDRAW {
    pub hdr: NMHDR,
    pub dw_draw_stage: u32,
    pub hdc: isize,
    pub rc: RECT,
    pub dw_item_spec: usize,
    pub u_item_state: u32,
    pub l_item_lparam: isize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct NMLVCUSTOMDRAW {
    pub nmcd: NMCUSTOMDRAW,
    pub clr_text: u32,
    pub clr_text_bk: u32,
    pub i_sub_item: i32,
    pub dw_item_type: u32,
    pub clr_face: u32,
    pub i_icon_effect: i32,
    pub i_icon_phase: i32,
    pub i_part_id: i32,
    pub i_state_id: i32,
    pub rc_text: RECT,
    pub u_align: u32,
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
pub struct TRACKMOUSEEVENT {
    pub cb_size: u32,
    pub dw_flags: u32,
    pub hwnd_track: isize,
    pub dw_hover_time: u32,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct DRAWITEMSTRUCT {
    pub ctl_type: u32,
    pub ctl_id: u32,
    pub item_id: u32,
    pub item_action: u32,
    pub item_state: u32,
    pub hwnd_item: isize,
    pub hdc: isize,
    pub rc_item: RECT,
    pub item_data: usize,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct MEASUREITEMSTRUCT {
    pub ctl_type: u32,
    pub ctl_id: u32,
    pub item_id: u32,
    pub item_width: u32,
    pub item_height: u32,
    pub item_data: usize,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: isize,
    pub f_erase: i32,
    pub rc_paint: RECT,
    pub f_restore: i32,
    pub f_inc_update: i32,
    pub rgb_reserved: [u8; 32],
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub dw_size: u32,
    pub dw_icc: u32,
}

pub use crate::platform::win32::types::{
    NOTIFY_FOR_THIS_SESSION, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
    WM_POWERBROADCAST, WM_WTSSESSION_CHANGE, WTS_CONSOLE_CONNECT, WTS_SESSION_LOCK,
    WTS_SESSION_LOGON, WTS_SESSION_UNLOCK, WTSRegisterSessionNotification,
    WTSUnRegisterSessionNotification,
};

// Window Styles & Controls
pub const GWL_STYLE: i32 = -16;
pub const GWLP_USERDATA: i32 = -21;

pub const IDOK: u32 = 1;
pub const IDCANCEL: u32 = 2;

pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_GETFONT: u32 = 0x0031;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SETCURSOR: u32 = 0x0020;
pub const WM_DRAWITEM: u32 = 0x002B;
pub const WM_SETICON: u32 = 0x0080;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_SYSKEYDOWN: u32 = 0x0104;
pub const WM_SYSKEYUP: u32 = 0x0105;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_HSCROLL: u32 = 0x0114;
pub const WM_VSCROLL: u32 = 0x0115;
pub const WM_CTLCOLOREDIT: u32 = 0x0133;
pub const WM_CTLCOLORLISTBOX: u32 = 0x0134;
pub const WM_CTLCOLORBTN: u32 = 0x0135;
pub const WM_CTLCOLORSTATIC: u32 = 0x0138;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const WM_MOUSELEAVE: u32 = 0x02A3;

pub const CS_VREDRAW: u32 = 0x0001;
pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_DBLCLKS: u32 = 0x0008;

pub const TME_LEAVE: u32 = 0x00000002;
pub const CBN_SELCHANGE: u32 = 1;

// Message Reflection (OCM_*)
pub const OCM_BASE: u32 = 0x2000;
pub const OCM_NOTIFY: u32 = OCM_BASE + 0x004E;
pub const OCM_COMMAND: u32 = OCM_BASE + 0x0111;
pub const OCM_DRAWITEM: u32 = OCM_BASE + 0x002B;
pub const OCM_CTLCOLORSTATIC: u32 = OCM_BASE + 0x0138;
pub const OCM_CTLCOLOREDIT: u32 = OCM_BASE + 0x0133;
pub const OCM_CTLCOLORLISTBOX: u32 = OCM_BASE + 0x0134;

pub const RDW_INVALIDATE: u32 = 0x0001;
pub const RDW_ERASE: u32 = 0x0004;
pub const RDW_ALLCHILDREN: u32 = 0x0080;
pub const RDW_FRAME: u32 = 0x0400;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_FRAMECHANGED: u32 = 0x0020;
pub const SWP_SHOWWINDOW: u32 = 0x0040;

pub const DT_LEFT: u32 = 0x00000000;
pub const DT_CENTER: u32 = 0x00000001;
pub const DT_VCENTER: u32 = 0x00000004;
pub const DT_WORDBREAK: u32 = 0x00000010;
pub const DT_SINGLELINE: u32 = 0x00000020;
pub const DT_NOPREFIX: u32 = 0x00000800;

pub const PS_SOLID: i32 = 0;
pub const PS_DASH: i32 = 1;
pub const PS_NULL: i32 = 5;

pub const IDC_ARROW: usize = 32512;
pub const IDC_IBEAM: usize = 32513;
pub const IDC_HAND: usize = 32649;

pub const ICC_LISTVIEW_CLASSES: u32 = 0x00000001;
pub const ICC_TAB_CLASSES: u32 = 0x00000008;
pub const ICC_STANDARD_CLASSES: u32 = 0x00004000;

pub const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
pub const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
pub const DWMWCP_ROUND: u32 = 2;

pub const TRANSPARENT: i32 = 1;
pub const OPAQUE: i32 = 2;

pub const WS_OVERLAPPED: u32 = 0x00000000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_MINIMIZEBOX: u32 = 0x00020000;
pub const WS_CLIPCHILDREN: u32 = 0x02000000;
pub const WS_TABSTOP: u32 = 0x00010000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_EX_DLGMODALFRAME: u32 = 0x00000001;
pub const WS_EX_CONTROLPARENT: u32 = 0x00010000;
pub const WS_EX_APPWINDOW: u32 = 0x00040000;

pub const ICON_SMALL: usize = 0;
pub const ICON_BIG: usize = 1;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;

pub const HWND_TOP: isize = 0;

pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;

pub const CLEARTYPE_QUALITY: u32 = 5;
pub const DEFAULT_CHARSET: u32 = 1;
pub const FW_NORMAL: i32 = 400;
pub const FW_SEMIBOLD: i32 = 600;
pub const FW_BOLD: i32 = 700;

pub const SRCCOPY: u32 = 0x00CC0020;
pub const NULL_BRUSH: i32 = 5;
pub const NULL_PEN: i32 = 8;
pub const DC_BRUSH: i32 = 18;
pub const DC_PEN: i32 = 19;

