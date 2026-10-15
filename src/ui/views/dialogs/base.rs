//! Modal Dialog Application Helpers (`src/ui/views/dialogs/base.rs`)
//!
//! Re-exports the framework dialog scaffold and GDI painters from [`crate::ui_framework::dialog`]
//! and provides application-level profile display helpers.

pub use crate::ui_framework::dialog::*;

use crate::platform::win32::ENGINE_INSTANCE;

/// Lấy chuỗi hiển thị ở thanh Footer Dialog cho Profile đang được chọn (vd: `"Áp dụng cho: 🌐 Mặc định"`).
pub fn active_profile_display_name() -> String {
    let strings = crate::language::current();
    let default_root_name = format!("🌐 {}", strings.banner_root_title);
    let profile_name = if let Ok(guard) = ENGINE_INSTANCE.lock()
        && let Some(ref engine) = *guard
    {
        let sel_id = &engine.profile_manager.selected_id;
        if sel_id == "global" {
            default_root_name
        } else if let Some(child) = engine.profile_manager.get_profile(sel_id) {
            format!("{} {}", child.icon, child.name)
        } else {
            default_root_name
        }
    } else {
        default_root_name
    };
    format!("{} {}", strings.hotkey_apply_prefix, profile_name)
}
