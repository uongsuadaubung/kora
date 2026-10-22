//! Two-Pane Split Layout Solver for Win32 (SplitLayout)
//! Divides a container into two panes (Horizontal or Vertical) with fixed size or percentage ratio.

use super::types::{Padding, Rect};
use crate::ui_framework::win32::SetWindowPos;

const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_NOOWNERZORDER: u32 = 0x0200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// Left and Right panes
    Horizontal,
    /// Top and Bottom panes
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum SplitRule {
    /// First pane has a fixed size in pixels, second pane takes remainder
    FixedFirst(i32),
    /// Second pane has a fixed size in pixels, first pane takes remainder
    FixedSecond(i32),
    /// Ratio of first pane to total available size (0.0 to 1.0, e.g. 0.4 = 40%)
    Ratio(f32),
}

#[derive(Debug, Clone)]
pub struct SplitLayout {
    bounds: Rect,
    direction: SplitDirection,
    rule: SplitRule,
    gap: i32,
    padding: Padding,
    first_hwnd: Option<isize>,
    second_hwnd: Option<isize>,
}

impl SplitLayout {
    pub fn new(bounds: Rect, direction: SplitDirection, rule: SplitRule) -> Self {
        Self {
            bounds,
            direction,
            rule,
            gap: 0,
            padding: Padding::ZERO,
            first_hwnd: None,
            second_hwnd: None,
        }
    }

    #[inline]
    pub fn horizontal(bounds: Rect, rule: SplitRule) -> Self {
        Self::new(bounds, SplitDirection::Horizontal, rule)
    }

    #[inline]
    pub fn vertical(bounds: Rect, rule: SplitRule) -> Self {
        Self::new(bounds, SplitDirection::Vertical, rule)
    }

    #[inline]
    pub fn at(x: i32, y: i32, width: i32, height: i32, direction: SplitDirection, rule: SplitRule) -> Self {
        Self::new(Rect::new(x, y, width, height), direction, rule)
    }

    #[inline]
    pub fn gap(&mut self, gap: i32) -> &mut Self {
        self.gap = gap.max(0);
        self
    }

    #[inline]
    pub fn padding(&mut self, pad: Padding) -> &mut Self {
        self.padding = pad;
        self
    }

    #[inline]
    pub fn first(&mut self, hwnd: isize) -> &mut Self {
        self.first_hwnd = Some(hwnd);
        self
    }

    #[inline]
    pub fn second(&mut self, hwnd: isize) -> &mut Self {
        self.second_hwnd = Some(hwnd);
        self
    }

    /// Computes layout rectangles for both panes: `(first_rect, second_rect)`
    pub fn compute(&self) -> (Rect, Rect) {
        let content_x = self.bounds.x + self.padding.left;
        let content_y = self.bounds.y + self.padding.top;
        let content_w = (self.bounds.width - self.padding.horizontal()).max(0);
        let content_h = (self.bounds.height - self.padding.vertical()).max(0);

        match self.direction {
            SplitDirection::Horizontal => {
                let avail = (content_w - self.gap).max(0);
                let first_w = match self.rule {
                    SplitRule::FixedFirst(w) => w.clamp(0, avail),
                    SplitRule::FixedSecond(w) => (avail - w).max(0),
                    SplitRule::Ratio(r) => ((avail as f32 * r.clamp(0.0, 1.0)).round() as i32).clamp(0, avail),
                };
                let second_w = (avail - first_w).max(0);
                let second_x = content_x + first_w + self.gap;

                (
                    Rect::new(content_x, content_y, first_w, content_h),
                    Rect::new(second_x, content_y, second_w, content_h),
                )
            }
            SplitDirection::Vertical => {
                let avail = (content_h - self.gap).max(0);
                let first_h = match self.rule {
                    SplitRule::FixedFirst(h) => h.clamp(0, avail),
                    SplitRule::FixedSecond(h) => (avail - h).max(0),
                    SplitRule::Ratio(r) => ((avail as f32 * r.clamp(0.0, 1.0)).round() as i32).clamp(0, avail),
                };
                let second_h = (avail - first_h).max(0);
                let second_y = content_y + first_h + self.gap;

                (
                    Rect::new(content_x, content_y, content_w, first_h),
                    Rect::new(content_x, second_y, content_w, second_h),
                )
            }
        }
    }

    /// Calculates and applies `SetWindowPos` to pane HWNDs if specified.
    pub fn apply(&self) -> (Rect, Rect) {
        let (r1, r2) = self.compute();
        let flags = SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER;

        if let Some(h1) = self.first_hwnd {
            unsafe {
                SetWindowPos(h1, 0, r1.x, r1.y, r1.width, r1.height, flags);
            }
        }
        if let Some(h2) = self.second_hwnd {
            unsafe {
                SetWindowPos(h2, 0, r2.x, r2.y, r2.width, r2.height, flags);
            }
        }

        (r1, r2)
    }

    #[inline]
    pub fn first_rect(&self) -> Rect {
        self.compute().0
    }

    #[inline]
    pub fn second_rect(&self) -> Rect {
        self.compute().1
    }
}
