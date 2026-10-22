//! UI Component Library for Pure Win32 API

use std::sync::atomic::{AtomicU32, Ordering};

pub mod action_table;
pub mod button;
pub mod card_layout;
pub mod checkbox;
pub mod combobox;
pub mod label;
pub mod listbox;
pub mod menu;
pub mod scroll_view;
pub mod slider;
pub mod tab_bar;
pub mod textbox;

pub use action_table::{
    ActionHover, ActionTable, ActionVariant, BadgeConfig, BadgeVariant, CellAlign, ColumnDef,
    ColumnStyle, ColumnWidth, TableAction, TableCell, TableRow,
};
pub use button::{ButtonVariant, PushButton};
pub use card_layout::{CardLayout, InCard, LayoutState, OutsideCard, TextFn, TextListFn};
pub use checkbox::{CheckBox, Inheritable, TwoState};
pub use combobox::ComboBox;
pub use label::{Label, LabelKind, LabelTone, Link, PillBadge, StaticText, SurfaceRole};
pub use listbox::ListBox;
pub use menu::PopupMenu;
pub use scroll_view::ScrollView;
pub use slider::Slider;
pub use tab_bar::TabBar;
pub use textbox::TextBox;

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

/// Trait implemented by all Win32 UI components so [`CardLayout`] can automatically
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

impl<M> ComponentHandle for CheckBox<M> {
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

impl_component_handle!(
    PushButton,
    ComboBox,
    TextBox,
    Slider,
    TabBar,
    ListBox,
    ActionTable,
);

pub trait DependsOnExt: ComponentHandle + Sized {
    /// Declares that this component's enabled state depends on the checked state of a parent CheckBox.
    /// It is automatically enabled when the parent is checked, and disabled when unchecked.
    #[inline]
    fn depends_on<M>(self, parent: &CheckBox<M>) -> Self {
        parent.add_dependent(&self);
        self
    }
}

impl<T: ComponentHandle> DependsOnExt for T {}
