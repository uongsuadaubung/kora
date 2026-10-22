//! Layout types and geometric primitives for UI Auto-Layout
//! Lightweight, zero-dependency layout definitions (Padding, Rect, Length, CrossAlign).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    #[inline]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    #[inline]
    pub const fn right(&self) -> i32 {
        self.x + self.width
    }

    #[inline]
    pub const fn bottom(&self) -> i32 {
        self.y + self.height
    }
}

/// Sizing constraint for a layout item (Fixed, Stretch, Auto).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Length {
    /// Fixed dimension in logical pixels.
    Fixed(i32),
    /// Proportionally stretches to fill available remaining space (flex factor).
    Stretch(u32),
    /// Auto-sized to content.
    Auto,
}

impl Default for Length {
    #[inline]
    fn default() -> Self {
        Self::Stretch(1)
    }
}

/// Alignment along the cross-axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossAlign {
    /// Stretches to fill the entire cross-axis dimension.
    #[default]
    Stretch,
    /// Aligns at the top (for horizontal) or left (for vertical).
    Start,
    /// Centers along the cross-axis.
    Center,
    /// Aligns at the bottom (for horizontal) or right (for vertical).
    End,
}

/// Insets padding around a layout container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Padding {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Padding {
    pub const ZERO: Self = Self { left: 0, top: 0, right: 0, bottom: 0 };

    #[inline]
    pub const fn all(val: i32) -> Self {
        Self {
            left: val,
            top: val,
            right: val,
            bottom: val,
        }
    }

    #[inline]
    pub const fn symmetric(horizontal: i32, vertical: i32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    #[inline]
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }

    #[inline]
    pub const fn horizontal(&self) -> i32 {
        self.left + self.right
    }

    #[inline]
    pub const fn vertical(&self) -> i32 {
        self.top + self.bottom
    }
}

