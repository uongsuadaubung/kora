//! Pure Windows Registry API Wrapper for Kora
//! Zero-dependency FFI bindings to advapi32.dll

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[cfg(target_os = "windows")]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegOpenKeyExW(
        hKey: isize,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut isize,
    ) -> i32;
    fn RegSetValueExW(
        hKey: isize,
        lpValueName: *const u16,
        Reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> i32;
    fn RegQueryValueExW(
        hKey: isize,
        lpValueName: *const u16,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> i32;
    fn RegDeleteValueW(hKey: isize, lpValueName: *const u16) -> i32;
    fn RegCloseKey(hKey: isize) -> i32;
}

#[cfg(target_os = "windows")]
const HKEY_CURRENT_USER: isize = -2147483647; // 0x80000001
#[cfg(target_os = "windows")]
const KEY_SET_VALUE: u32 = 0x0002;
#[cfg(target_os = "windows")]
const KEY_QUERY_VALUE: u32 = 0x0001;
#[cfg(target_os = "windows")]
const REG_SZ: u32 = 1;

#[cfg(target_os = "windows")]
struct RegKeyGuard(isize);

#[cfg(target_os = "windows")]
impl Drop for RegKeyGuard {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// Queries Windows Registry directly (HKCU\Software\Microsoft\Windows\CurrentVersion\Run)
/// to determine if Kora is configured to autostart with Windows.
pub fn is_windows_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        let subkey = to_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let val_name = to_wide("Kora");

        let mut raw_hkey: isize = 0;
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut raw_hkey,
        ) == 0
        {
            let guard = RegKeyGuard(raw_hkey);
            let mut val_type = 0u32;
            let mut len = 0u32;
            let res = RegQueryValueExW(
                guard.0,
                val_name.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut len,
            );
            return res == 0;
        }
    }
    false
}

/// Sets or removes Kora from Windows Startup registry (HKCU\Software\Microsoft\Windows\CurrentVersion\Run)
pub fn set_windows_autostart(enable: bool) {
    #[cfg(target_os = "windows")]
    unsafe {
        let subkey = to_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let val_name = to_wide("Kora");

        let mut raw_hkey: isize = 0;
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut raw_hkey,
        ) == 0
        {
            let guard = RegKeyGuard(raw_hkey);
            if enable {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = format!("\"{}\" --autostart", exe_path.to_string_lossy());
                    let wide_path = to_wide(&path_str);
                    let byte_len = (wide_path.len() * 2) as u32;
                    RegSetValueExW(
                        guard.0,
                        val_name.as_ptr(),
                        0,
                        REG_SZ,
                        wide_path.as_ptr() as *const u8,
                        byte_len,
                    );
                }
            } else {
                RegDeleteValueW(guard.0, val_name.as_ptr());
            }
        }
    }
}
