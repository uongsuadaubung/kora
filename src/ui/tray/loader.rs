//! In-memory ICO file parser and HICON creator using pure Win32 API.
//! Embeds .ico assets without needing external file dependencies or rc.exe.

#[repr(C, packed)]
struct IcoHeader {
    reserved: u16,
    icon_type: u16,
    image_count: u16,
}

#[repr(C, packed)]
struct IcoDirEntry {
    width: u8,
    height: u8,
    color_count: u8,
    reserved: u8,
    planes: u16,
    bit_count: u16,
    bytes_in_res: u32,
    image_offset: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn CreateIconFromResourceEx(
        pb_icon_bits: *const u8,
        cb_icon_bits: u32,
        f_icon: i32,
        dw_version: u32,
        cx_desired: i32,
        cy_desired: i32,
        u_flags: u32,
    ) -> isize;

    fn DestroyIcon(h_icon: isize) -> i32;
    fn GetSystemMetrics(n_index: i32) -> i32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegOpenKeyExW(
        hKey: isize,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut isize,
    ) -> i32;

    fn RegQueryValueExW(
        hKey: isize,
        lpValueName: *const u16,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> i32;

    fn RegCloseKey(hKey: isize) -> i32;
}

const HKEY_CURRENT_USER: isize = -2147483647; // 0x80000001
const KEY_READ: u32 = 0x20019;
const SM_CXSMICON: i32 = 49;
const SM_CYSMICON: i32 = 50;
const LR_DEFAULTCOLOR: u32 = 0x0000;

/// Detects whether Windows taskbar is currently using Dark Theme.
/// On Windows 10 & 11, taskbar theme is determined by registry key:
/// `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\SystemUsesLightTheme`
/// - 0: Dark taskbar (modern white/monochrome icon is recommended)
/// - 1: Light taskbar (classic colored icon is required; otherwise flat white icons are invisible!)
pub fn is_windows_dark_taskbar() -> bool {
    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let val_name: Vec<u16> = "SystemUsesLightTheme\0".encode_utf16().collect();

    let mut hkey: isize = 0;
    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
            let mut val: u32 = 0;
            let mut val_type: u32 = 0;
            let mut val_len: u32 = std::mem::size_of::<u32>() as u32;
            let res = RegQueryValueExW(
                hkey,
                val_name.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                &mut val as *mut u32 as *mut u8,
                &mut val_len,
            );
            RegCloseKey(hkey);
            if res == 0 {
                return val == 0;
            }
        }
    }
    // Default to dark taskbar for fallback
    true
}

/// Loads an HICON from raw ICO file bytes matching the desired system size.
pub fn load_icon_from_memory(ico_bytes: &[u8], desired_w: i32, desired_h: i32) -> Option<isize> {
    if ico_bytes.len() < std::mem::size_of::<IcoHeader>() {
        return None;
    }

    let header = unsafe { &*(ico_bytes.as_ptr() as *const IcoHeader) };
    if header.reserved != 0 || header.icon_type != 1 || header.image_count == 0 {
        return None;
    }

    let count = header.image_count as usize;
    let entry_size = std::mem::size_of::<IcoDirEntry>();
    let header_size = std::mem::size_of::<IcoHeader>();

    if ico_bytes.len() < header_size + count * entry_size {
        return None;
    }

    let entries_ptr = unsafe { ico_bytes.as_ptr().add(header_size) as *const IcoDirEntry };

    // Find best match for desired size (e.g. 16, 24, 32)
    let mut best_entry: Option<&IcoDirEntry> = None;
    let mut best_diff = i32::MAX;

    for i in 0..count {
        let entry = unsafe { &*entries_ptr.add(i) };
        let w = if entry.width == 0 {
            256
        } else {
            entry.width as i32
        };
        let diff = (w - desired_w).abs();
        if diff < best_diff {
            best_diff = diff;
            best_entry = Some(entry);
        }
    }

    let entry = best_entry?;
    let offset = entry.image_offset as usize;
    let len = entry.bytes_in_res as usize;

    if offset + len > ico_bytes.len() {
        return None;
    }

    let res_data = &ico_bytes[offset..offset + len];
    let hicon = unsafe {
        CreateIconFromResourceEx(
            res_data.as_ptr(),
            len as u32,
            1,          // 1 = icon
            0x00030000, // Version 3.0
            desired_w,
            desired_h,
            LR_DEFAULTCOLOR,
        )
    };

    if hicon != 0 { Some(hicon) } else { None }
}

/// Helper to get standard small icon size (for tray)
pub fn get_small_icon_size() -> (i32, i32) {
    unsafe {
        let cx = GetSystemMetrics(SM_CXSMICON);
        let cy = GetSystemMetrics(SM_CYSMICON);
        (if cx > 0 { cx } else { 16 }, if cy > 0 { cy } else { 16 })
    }
}

/// Safely destroy an icon
pub fn safe_destroy_icon(hicon: isize) {
    if hicon != 0 {
        unsafe {
            DestroyIcon(hicon);
        }
    }
}
