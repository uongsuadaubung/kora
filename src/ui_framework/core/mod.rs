//! Project-Agnostic Declarative UI Core (React Component Model + Bootstrap Utility System)
//!
//! Exports [`Element<M>`], [`Surface<M>`], [`Variant`], [`Size`], and all Bootstrap-style
//! layout and widget constructors (`row`, `col`, `grid`, `card`, `alert`, `text`, `badge`,
//! `button`, `switch`, `checkbox`, `select`, `slider`, `tab_group`, `tabs`, `divider`, `spacer`, `empty`, `modal`).

pub mod badge;
pub mod button;
pub mod checkbox;
pub mod containers;
pub mod element;
pub mod input;
pub mod label;
pub mod layout_engine;
pub mod modal;
pub mod select;
pub mod slider;
pub mod style;
pub mod surface;
pub mod switch;
pub mod tab_group;
pub mod text;

pub use badge::*;
pub use button::*;
pub use checkbox::*;
pub use containers::*;
pub use element::*;
pub use input::*;
pub use label::*;
pub use layout_engine::*;
pub use modal::{
    modal_dialog, modal_footer, open_modal, open_modal_with_wndproc, ModalConfig, ModalDialog,
    ModalHandle,
};
pub use select::*;
pub use slider::*;
pub use surface::*;
pub use switch::*;
pub use tab_group::*;
pub use text::*;
