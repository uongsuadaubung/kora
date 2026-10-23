//! UI Component Library for Pure Win32 API

use std::sync::atomic::{AtomicU32, Ordering};

pub mod action_table;
pub mod button;
pub mod combobox;
pub mod label;
pub mod menu;
pub mod tab_bar;
pub mod textbox;

pub use action_table::{
    ActionHover, ActionTable, ActionVariant, BadgeConfig, BadgeVariant, CellAlign, ColumnDef,
    ColumnStyle, ColumnWidth, TableAction, TableCell, TableRow,
};
pub use button::{ButtonVariant, PushButton};
pub use combobox::ComboBox;
pub use label::{Label, LabelKind, LabelTone, Link, PillBadge, StaticText, SurfaceRole};
pub use menu::PopupMenu;
pub use tab_bar::TabBar;
pub use textbox::TextBox;

pub type TextFn<L = ()> = fn(&L) -> &'static str;
pub type TextListFn<L = ()> = fn(&L) -> Vec<&'static str>;

static NEXT_CONTROL_ID: AtomicU32 = AtomicU32::new(10_000);

/// Generates a unique runtime Win32 control ID (`>= 10_000`).
#[inline]
pub fn next_control_id() -> u32 {
    NEXT_CONTROL_ID.fetch_add(1, Ordering::Relaxed)
}

/// Returns `id` if non-zero, or allocates a fresh runtime ID via [`next_control_id`].
#[inline]
pub fn resolve_id(id: u32) -> u32 {
    if id == 0 { next_control_id() } else { id }
}

/// Trait implemented by all Win32 UI components so parent containers can automatically
/// track their `HWND` for visibility toggling and localization updates.
pub trait ComponentHandle {
    fn hwnd(&self) -> isize;
}

impl<K: LabelKind> ComponentHandle for Label<K> {
    #[inline]
    fn hwnd(&self) -> isize {
        self.hwnd()
    }
}

macro_rules! impl_component_handle {
    ($($t:ty),+ $(,)?) => {
        $(
            impl ComponentHandle for $t {
                #[inline]
                fn hwnd(&self) -> isize {
                    <$t>::hwnd(self)
                }
            }
        )+
    };
}

impl_component_handle!(PushButton, ComboBox, TextBox, TabBar, ActionTable,);
