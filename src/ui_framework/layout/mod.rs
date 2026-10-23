//! UI Auto-Layout Engine (Horizontal Stack)
//! Lightweight, zero-dependency layout engine for native Win32 controls.

pub mod hstack;
pub mod types;

pub use hstack::HStack;
pub use types::{CrossAlign, Length, Padding, Rect};

/// Type alias matching Slint/Flutter/SwiftUI terminology:
pub type HorizontalLayout = HStack;
