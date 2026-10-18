//! Active Application Context & Focused Target Strategy Tracking

use super::types::{
    CloseHandle, GetForegroundWindow, GetWindowThreadProcessId, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use std::sync::atomic::{AtomicU8, Ordering};

/// Cached surgical target key injection strategy for active focused control:
/// 0: Default / Unset, 1: AutocompleteGuard, 2: Standard, 3: NavigationCollapse, 4: ClipboardPaste, 5: Passthrough
pub static CURRENT_TARGET_STRATEGY: AtomicU8 = AtomicU8::new(0);

#[inline]
pub fn get_current_target_strategy_raw() -> u8 {
    CURRENT_TARGET_STRATEGY.load(Ordering::Relaxed)
}

#[inline]
pub fn set_current_target_strategy_raw(val: u8) {
    CURRENT_TARGET_STRATEGY.store(val, Ordering::Relaxed);
}

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

