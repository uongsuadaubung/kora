pub mod process;
pub mod registry;
#[cfg(target_os = "windows")]
pub mod win32;

pub use process::{
    alloc_console, attach_parent_console, ensure_single_instance, release_single_instance,
};
pub use registry::{is_windows_autostart_enabled, set_windows_autostart};

#[cfg(target_os = "windows")]
pub use win32::{reinstall_hooks, reset_modifier_states};
