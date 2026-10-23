//! Horizontal Flex/Stack Layout Solver for Win32 (HorizontalLayout)
//! Computes (x, y, width, height) with spacing, padding, fixed sizes, and stretch factors.

use super::types::{CrossAlign, Length, Padding, Rect};
use crate::ui_framework::win32::SetWindowPos;

const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_NOOWNERZORDER: u32 = 0x0200;

#[derive(Debug, Clone)]
struct HStackItem {
    length: Length,
    hwnd: Option<isize>,
    height_override: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct HStack {
    bounds: Rect,
    spacing: i32,
    padding: Padding,
    align: CrossAlign,
    items: Vec<HStackItem>,
}

impl HStack {
    #[inline]
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            spacing: 0,
            padding: Padding::ZERO,
            align: CrossAlign::Stretch,
            items: Vec::new(),
        }
    }

    #[inline]
    pub fn at(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self::new(Rect::new(x, y, width, height))
    }

    #[inline]
    pub fn spacing(&mut self, spacing: i32) -> &mut Self {
        self.spacing = spacing;
        self
    }

    #[inline]
    pub fn padding(&mut self, padding: Padding) -> &mut Self {
        self.padding = padding;
        self
    }

    #[inline]
    pub fn align(&mut self, align: CrossAlign) -> &mut Self {
        self.align = align;
        self
    }

    #[inline]
    pub fn add(&mut self, length: Length, hwnd: Option<isize>) -> &mut Self {
        self.items.push(HStackItem {
            length,
            hwnd,
            height_override: None,
        });
        self
    }

    #[inline]
    pub fn add_fixed(&mut self, width: i32, hwnd: Option<isize>) -> &mut Self {
        self.add(Length::Fixed(width), hwnd)
    }

    #[inline]
    pub fn add_fixed_with_height(
        &mut self,
        width: i32,
        height: i32,
        hwnd: Option<isize>,
    ) -> &mut Self {
        self.items.push(HStackItem {
            length: Length::Fixed(width),
            hwnd,
            height_override: Some(height),
        });
        self
    }

    #[inline]
    pub fn add_stretch(&mut self, flex: u32, hwnd: Option<isize>) -> &mut Self {
        self.add(Length::Stretch(flex), hwnd)
    }

    #[inline]
    pub fn add_spacer(&mut self) -> &mut Self {
        self.add(Length::Stretch(1), None)
    }

    /// Computes the layout positions and applies `SetWindowPos` to any provided HWNDs.
    /// Returns the resolved `Rect` bounding boxes for all items in order.
    pub fn apply(&self) -> Vec<Rect> {
        let n = self.items.len();
        if n == 0 {
            return Vec::new();
        }

        let inner_w = (self.bounds.width - self.padding.left - self.padding.right).max(0);
        let total_spacing = ((n - 1) as i32) * self.spacing;

        let mut fixed_sum = 0;
        let mut total_stretch = 0;

        for item in &self.items {
            match item.length {
                Length::Fixed(w) => fixed_sum += w,
                Length::Stretch(weight) => total_stretch += weight,
                Length::Auto => fixed_sum += 0,
            }
        }

        let remaining_w = (inner_w - total_spacing - fixed_sum).max(0);

        let mut cur_x = self.bounds.x + self.padding.left;
        let inner_h = (self.bounds.height - self.padding.top - self.padding.bottom).max(0);
        let mut results = Vec::with_capacity(n);

        for item in &self.items {
            let item_w = match item.length {
                Length::Fixed(w) => w,
                Length::Stretch(weight) => {
                    if total_stretch > 0 {
                        (remaining_w * (weight as i32)) / (total_stretch as i32)
                    } else {
                        0
                    }
                }
                Length::Auto => 0,
            };

            let item_h = item.height_override.unwrap_or(inner_h).min(inner_h);
            let item_y = match self.align {
                CrossAlign::Stretch => self.bounds.y + self.padding.top,
                CrossAlign::Start => self.bounds.y + self.padding.top,
                CrossAlign::Center => self.bounds.y + self.padding.top + (inner_h - item_h) / 2,
                CrossAlign::End => {
                    self.bounds.y + self.bounds.height - self.padding.bottom - item_h
                }
            };

            let rect = Rect::new(cur_x, item_y, item_w, item_h);
            results.push(rect);

            if let Some(hwnd) = item.hwnd {
                if hwnd != 0 {
                    unsafe {
                        SetWindowPos(
                            hwnd,
                            0,
                            rect.x,
                            rect.y,
                            rect.width,
                            rect.height,
                            SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                        );
                    }
                }
            }

            cur_x += item_w + self.spacing;
        }

        results
    }
}
