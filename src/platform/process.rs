//! Windows Process Lifecycle & Console Management Helpers

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicIsize, Ordering};

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(
        lpMutexAttributes: *mut std::ffi::c_void,
        bInitialOwner: i32,
        lpName: *const u16,
    ) -> isize;
    fn CloseHandle(hObject: isize) -> i32;
    fn GetLastError() -> u32;
    fn AttachConsole(dwProcessId: u32) -> i32;
    fn AllocConsole() -> i32;
}

#[cfg(target_os = "windows")]
static SINGLE_INSTANCE_MUTEX: AtomicIsize = AtomicIsize::new(0);

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
    fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
    fn SetForegroundWindow(hWnd: isize) -> i32;
}

/// Ensures only a single instance of Kora is running.
/// If an instance is already active and `is_cli` is false:
/// - Brings the existing Kora Control Panel to the foreground.
/// - Returns `false` (signaling that the secondary instance should exit).
///
/// Returns `true` if this is the primary instance.
pub fn ensure_single_instance(is_cli: bool) -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        const ERROR_ALREADY_EXISTS: u32 = 183;
        let mutex_name: Vec<u16> = OsStr::new("Local\\Kora_SingleInstance_Mutex")
            .encode_wide()
            .chain(Some(0))
            .collect();
        let h_mutex = CreateMutexW(std::ptr::null_mut(), 1, mutex_name.as_ptr());

        if GetLastError() == ERROR_ALREADY_EXISTS && !is_cli {
            if h_mutex != 0 {
                CloseHandle(h_mutex);
            }
            let class_name: Vec<u16> = OsStr::new("KoraControlPanelClass")
                .encode_wide()
                .chain(Some(0))
                .collect();
            let hwnd = FindWindowW(class_name.as_ptr(), std::ptr::null());
            if hwnd != 0 {
                ShowWindow(hwnd, 5); // SW_SHOW
                SetForegroundWindow(hwnd);
            }
            return false;
        }
        SINGLE_INSTANCE_MUTEX.store(h_mutex, Ordering::SeqCst);
    }
    true
}

/// Releases the single-instance mutex early so a newly spawned updated binary can start immediately.
pub fn release_single_instance() {
    #[cfg(target_os = "windows")]
    unsafe {
        let h = SINGLE_INSTANCE_MUTEX.swap(0, Ordering::SeqCst);
        if h != 0 {
            CloseHandle(h);
        }
    }
}

/// Attaches to existing parent console if launched from terminal (cmd/PowerShell)
pub fn attach_parent_console() {
    #[cfg(target_os = "windows")]
    unsafe {
        const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Allocates a new console window if running in standalone CLI mode
pub fn alloc_console() {
    #[cfg(target_os = "windows")]
    unsafe {
        AllocConsole();
    }
}
