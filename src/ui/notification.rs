//! Reusable System Notification Module for Kora
//! Delivers native Windows desktop notifications (Action Center toasts / balloon notifications)
//! via Win32 Shell_NotifyIconW.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub level: NotificationLevel,
}

impl Notification {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        level: NotificationLevel,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            level,
        }
    }

    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, NotificationLevel::Info)
    }

    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, NotificationLevel::Warning)
    }

    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, NotificationLevel::Error)
    }
}

/// Dispatches a system notification to Windows. Returns `true` if delivered successfully.
pub fn send_notification(notification: &Notification) -> bool {
    let hwnd = crate::ui::helper_hwnd();
    if hwnd == 0 {
        return false;
    }

    use crate::ui::tray::handler::TRAY_ICON_ID;
    use crate::ui::tray::icon::{
        NIF_INFO, NIIF_ERROR, NIIF_INFO, NIIF_WARNING, NIM_MODIFY, NOTIFYICONDATAW,
        Shell_NotifyIconW,
    };

    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.h_wnd = hwnd;
    nid.u_id = TRAY_ICON_ID;
    nid.u_flags = NIF_INFO;

    let wide_title: Vec<u16> = OsStr::new(&notification.title).encode_wide().collect();
    let len_t = wide_title.len().min(63);
    nid.sz_info_title[..len_t].copy_from_slice(&wide_title[..len_t]);
    nid.sz_info_title[len_t] = 0;

    let wide_msg: Vec<u16> = OsStr::new(&notification.message).encode_wide().collect();
    let len_m = wide_msg.len().min(255);
    nid.sz_info[..len_m].copy_from_slice(&wide_msg[..len_m]);
    nid.sz_info[len_m] = 0;

    nid.dw_info_flags = match notification.level {
        NotificationLevel::Info => NIIF_INFO,
        NotificationLevel::Warning => NIIF_WARNING,
        NotificationLevel::Error => NIIF_ERROR,
    };
    nid.u_timeout_or_version = 3000;

    let res = unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut nid) };
    res != 0
}

/// Convenience helper to send an informational notification
pub fn notify_info(title: impl Into<String>, message: impl Into<String>) -> bool {
    send_notification(&Notification::info(title, message))
}

/// Convenience helper to send a warning notification
pub fn notify_warning(title: impl Into<String>, message: impl Into<String>) -> bool {
    send_notification(&Notification::warning(title, message))
}

/// Convenience helper to send an error notification
pub fn notify_error(title: impl Into<String>, message: impl Into<String>) -> bool {
    send_notification(&Notification::error(title, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_constructors() {
        let n_info = Notification::info("Kora", "Đã tạo profile");
        assert_eq!(n_info.title, "Kora");
        assert_eq!(n_info.message, "Đã tạo profile");
        assert_eq!(n_info.level, NotificationLevel::Info);

        let n_warn = Notification::warning("Cảnh báo", "Xung đột phím");
        assert_eq!(n_warn.level, NotificationLevel::Warning);

        let n_err = Notification::error("Lỗi", "Không thể tải");
        assert_eq!(n_err.level, NotificationLevel::Error);
    }

    #[test]
    fn test_send_notification_without_hwnd_returns_false() {
        let n = Notification::info("Test", "No crash when hwnd is 0");
        // In headless test runner, helper_hwnd is 0
        let res = send_notification(&n);
        assert!(!res);
    }
}
