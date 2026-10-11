//! Active Application Context & Browser Omnibox Autocomplete Detection

use super::types::{
    CloseHandle, GUITHREADINFO, GetClassNameW, GetForegroundWindow, GetGUIThreadInfo,
    GetWindowThreadProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};
use std::sync::atomic::{AtomicIsize, AtomicU8, AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AppKind {
    Other = 0,
    Chromium = 1,
    GenericAutocomplete = 2,
}

impl From<&str> for AppKind {
    fn from(exe_name: &str) -> Self {
        match exe_name {
            "chrome.exe" | "msedge.exe" | "brave.exe" | "opera.exe" | "vivaldi.exe"
            | "coc_coc.exe" => AppKind::Chromium,
            "firefox.exe" | "excel.exe" => AppKind::GenericAutocomplete,
            _ => AppKind::Other,
        }
    }
}

impl From<u8> for AppKind {
    fn from(val: u8) -> Self {
        match val {
            1 => AppKind::Chromium,
            2 => AppKind::GenericAutocomplete,
            _ => AppKind::Other,
        }
    }
}

static CACHED_PID: AtomicU32 = AtomicU32::new(0);
static CACHED_APP_KIND: AtomicU8 = AtomicU8::new(0);
static CACHED_FOCUS_HWND: AtomicIsize = AtomicIsize::new(0);
static CACHED_FIX_TYPE: AtomicU8 = AtomicU8::new(0);

/// Retrieves the executable file name (e.g., "cs2.exe") for a given process ID
pub fn get_process_name_for_pid(pid: u32) -> String {
    if pid == 0 {
        return String::new();
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle != 0 {
            let mut buf = [0u16; 1024];
            let mut size = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
            CloseHandle(handle);

            if ok != 0 && size > 0 {
                let full_path = String::from_utf16_lossy(&buf[..size as usize]);
                full_path.rsplit('\\').next().unwrap_or("").to_lowercase()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}

/// Retrieves the executable file name of the current foreground active window
pub fn get_foreground_process_name() -> String {
    unsafe {
        let fg = GetForegroundWindow();
        if fg == 0 {
            return String::new();
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, &mut pid);
        get_process_name_for_pid(pid)
    }
}

pub fn get_app_kind_for_pid(pid: u32) -> AppKind {
    let cached_pid = CACHED_PID.load(Ordering::Relaxed);
    if cached_pid == pid {
        return AppKind::from(CACHED_APP_KIND.load(Ordering::Relaxed));
    }

    let kind = unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle != 0 {
            let mut buf = [0u16; 1024];
            let mut size = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
            CloseHandle(handle);

            if ok != 0 && size > 0 {
                let full_path = String::from_utf16_lossy(&buf[..size as usize]);
                let exe_name = full_path.rsplit('\\').next().unwrap_or("").to_lowercase();
                AppKind::from(exe_name.as_str())
            } else {
                AppKind::Other
            }
        } else {
            AppKind::Other
        }
    };

    CACHED_PID.store(pid, Ordering::Relaxed);
    CACHED_APP_KIND.store(kind as u8, Ordering::Relaxed);
    kind
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutocompleteFixType {
    None,
    ChromiumOmnibox,
    GenericAutocomplete,
}

pub fn detect_autocomplete_context() -> AutocompleteFixType {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground == 0 {
            return AutocompleteFixType::None;
        }

        let mut pid = 0u32;
        let thread_id = GetWindowThreadProcessId(foreground, &mut pid);
        if pid == 0 {
            return AutocompleteFixType::None;
        }

        let app_kind = get_app_kind_for_pid(pid);
        if app_kind == AppKind::Other {
            return AutocompleteFixType::None;
        }

        let mut gui = GUITHREADINFO::default();
        let focus_hwnd = if GetGUIThreadInfo(thread_id, &mut gui) != 0 && gui.hwnd_focus != 0 {
            gui.hwnd_focus
        } else {
            foreground
        };

        let cached_hwnd = CACHED_FOCUS_HWND.load(Ordering::Relaxed);
        if cached_hwnd != 0 && cached_hwnd == focus_hwnd {
            return match CACHED_FIX_TYPE.load(Ordering::Relaxed) {
                1 => AutocompleteFixType::ChromiumOmnibox,
                2 => AutocompleteFixType::GenericAutocomplete,
                _ => AutocompleteFixType::None,
            };
        }

        let mut class_buf = [0u16; 128];
        let class_len = GetClassNameW(focus_hwnd, class_buf.as_mut_ptr(), 128);
        let class_slice = if class_len > 0 {
            &class_buf[..class_len as usize]
        } else {
            &[]
        };

        let fix_type = match app_kind {
            AppKind::Chromium => {
                // When focus is inside a web page (Facebook, Google Docs, ChatGPT, YouTube, etc.):
                // Chromium uses "Chrome_RenderWidgetHostHWND".
                // In that case, do NOT apply autocomplete fix: type completely normally!
                if utf16_str_eq(class_slice, "Chrome_RenderWidgetHostHWND") {
                    AutocompleteFixType::None
                } else {
                    AutocompleteFixType::ChromiumOmnibox
                }
            }
            AppKind::GenericAutocomplete => {
                if utf16_str_eq(class_slice, "MozillaContentWindowClass") {
                    AutocompleteFixType::None
                } else {
                    AutocompleteFixType::GenericAutocomplete
                }
            }
            AppKind::Other => AutocompleteFixType::None,
        };

        let fix_type_u8 = match fix_type {
            AutocompleteFixType::None => 0,
            AutocompleteFixType::ChromiumOmnibox => 1,
            AutocompleteFixType::GenericAutocomplete => 2,
        };
        CACHED_FOCUS_HWND.store(focus_hwnd, Ordering::Relaxed);
        CACHED_FIX_TYPE.store(fix_type_u8, Ordering::Relaxed);
        fix_type
    }
}

fn utf16_str_eq(slice: &[u16], ascii_str: &str) -> bool {
    if slice.len() != ascii_str.len() {
        return false;
    }
    slice
        .iter()
        .zip(ascii_str.bytes())
        .all(|(&u, b)| u == b as u16)
}
