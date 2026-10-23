//! Modal Dialog Application Helpers (`src/ui/views/dialogs/base.rs`)
//!
//! Re-exports the framework dialog scaffold and GDI painters from [`crate::ui_framework::dialog`]
//! and provides application-level profile display helpers.

pub use crate::ui_framework::dialog::*;

use crate::platform::win32::ENGINE_INSTANCE;
use crate::ui_framework::core::{
    BgStyle, BorderStyle, Element, Styleable, Variant, button, col, elements, row, spacer, text,
};
use crate::ui_framework::tokens::radii;

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

/// Khối Callout hướng dẫn / giới thiệu ở đầu hộp thoại Modal
pub fn dialog_callout<M: Clone + Send + Sync + 'static>(prompt: &str) -> Element<M> {
    col(elements![text(prompt).secondary().small()])
        .bg_card()
        .border()
        .rounded(radii::RADIUS_MD)
        .px(3)
        .py(2)
}

/// Khung chụp phím tương tác chuẩn cho các hộp thoại gán / đổi / khóa phím
pub fn key_capture_box<M: Clone + Send + Sync + 'static>(
    label: &str,
    is_listening: bool,
    is_placeholder: bool,
    on_click: M,
) -> Element<M> {
    let (bg_kind, border_kind, text_elem) = if is_listening {
        (
            BgStyle::Subtle(Variant::Primary),
            BorderStyle::Accent,
            text(label).accent().bold(),
        )
    } else if is_placeholder {
        (BgStyle::Input, BorderStyle::Dashed, text(label).secondary())
    } else {
        (BgStyle::Card, BorderStyle::Normal, text(label).bold())
    };

    col(elements![text_elem.text_center().w_full().nowrap()])
        .bg(bg_kind)
        .border_style(border_kind)
        .rounded(radii::RADIUS_MD)
        .w_full()
        .h(46)
        .items_center()
        .justify_center()
        .on_click(on_click)
}

/// Thanh Footer hộp thoại chuẩn với ghi chú Profile bên trái và các nút Hủy / Lưu bên phải
pub fn dialog_footer<M: Clone + Send + Sync + 'static>(
    profile_name: &str,
    cancel_text: &str,
    cancel_msg: M,
    apply_text: &str,
    apply_msg: M,
    apply_variant: Variant,
    apply_disabled: bool,
) -> Element<M> {
    row(elements![
        text(profile_name).secondary().small(),
        spacer(),
        button(cancel_text, Variant::Secondary, cancel_msg),
        button(apply_text, apply_variant, apply_msg).disabled(apply_disabled),
    ])
    .items_center()
    .pt(2)
    .gap(2)
}
