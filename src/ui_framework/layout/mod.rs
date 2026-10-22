//! UI Auto-Layout Engine (Horizontal, Vertical, Grid, Form, Split)
//! Lightweight, zero-dependency layout engine for native Win32 controls.

pub mod form;
pub mod grid;
pub mod hstack;
pub mod split;
pub mod types;
pub mod vstack;

pub use form::FormLayout;
pub use grid::GridLayout;
pub use hstack::HStack;
pub use split::{SplitDirection, SplitRule, SplitLayout};
pub use types::{CrossAlign, Length, Padding, Rect};
pub use vstack::VStack;

/// Type aliases matching Slint/Flutter/SwiftUI terminology:
pub type HorizontalLayout = HStack;
pub type VerticalLayout = VStack;
