//! Kora Native Win32 UI Framework (`src/ui_framework/`)
//!
//! Phân hệ nền tảng giao diện thuần Win32 độc lập hoàn toàn với nghiệp vụ bộ gõ (`engine`):
//! - [`win32`]: FFI bindings, GDI zero-allocation primitives, `BackBuffer`, `OCM_*` reflection & `WM_CTLCOLOR*` handlers.
//! - [`colors`]: Bảng màu `ThemePalette` chuẩn Fluent Light / Dark.
//! - [`tokens`]: Design tokens (`typography`, `spacing`, `borders`, `radii`, `layout`).
//! - [`theme`]: Quản lý trạng thái Dark/Light, GDI Brush cache & Font cache (`cached_font`).
//! - [`components`]: Bộ 9 UI Component tự chủ + `CardLayout` (Auto-ID, Auto-HWND, Auto-I18n, 1-line Row Builders).
//! - [`dialog`]: Khung sườn tạo & vẽ Modal Dialog dùng chung (`create_modal_dialog`, `reflect_dialog_messages`, GDI helpers).

pub mod colors;
pub mod components;
pub mod dialog;
pub mod theme;
pub mod tokens;
pub mod win32;

pub use colors::ThemePalette;
pub use theme::{
    apply_window_theme, cached_font, get_app_font_bold, get_app_font_normal, is_current_dark,
    recreate_theme_brushes,
};

/// Trait đại diện cho bảng chuỗi đa ngôn ngữ tĩnh (`&'static Self`) của ứng dụng.
/// Giúp `CardLayout` và `CheckBox<Inheritable>` tự động cập nhật đa ngôn ngữ mà không phụ thuộc ngược vào crate ứng dụng.
pub trait I18nSource: 'static {
    /// Trả về bảng chuỗi ngôn ngữ hiện hành.
    fn current() -> &'static Self;

    /// Chuỗi chú thích hiển thị cạnh `CheckBox<Inheritable>` khi ở trạng thái kế thừa (`BST_INDETERMINATE`).
    fn inherit_note(&self, _inherited_on: bool) -> &'static str {
        ""
    }
}

impl I18nSource for () {
    fn current() -> &'static Self {
        &()
    }
}

/// Prelude tiện lợi cho tầng tiêu thụ (`src/ui/views/`):
/// chỉ cần `use crate::ui_framework::prelude::*;` là có trọn bộ Component, CardLayout, ThemePalette, Tokens và Dialog helpers.
pub mod prelude {
    pub use crate::ui_framework::I18nSource;
    pub use crate::ui_framework::colors::ThemePalette;
    pub use crate::ui_framework::components::*;
    pub use crate::ui_framework::dialog::*;
    pub use crate::ui_framework::theme::{
        cached_font, get_app_font_bold, get_app_font_normal, is_current_dark,
    };
    pub use crate::ui_framework::tokens::{borders, layout, radii, spacing, typography};
    pub use crate::ui_framework::win32::{RECT, set_hwnds_visible, to_wide};
}
