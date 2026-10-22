//! Vertical Flex/Stack Layout Solver for Win32 (VerticalLayout)
//! Computes (x, y, width, height) vertically with spacing, padding, fixed sizes, and stretch factors.

use super::types::{CrossAlign, Length, Padding, Rect};
use crate::ui_framework::win32::SetWindowPos;

const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_NOOWNERZORDER: u32 = 0x0200;

#[derive(Debug, Clone)]
struct VStackItem {
    length: Length,
    hwnd: Option<isize>,
    width_override: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct VStack {
    bounds: Rect,
    spacing: i32,
    padding: Padding,
    align: CrossAlign,
    items: Vec<VStackItem>,
}

impl VStack {
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
        self.items.push(VStackItem {
            length,
            hwnd,
            width_override: None,
        });
        self
    }

    #[inline]
    pub fn add_fixed(&mut self, height: i32, hwnd: Option<isize>) -> &mut Self {
        self.add(Length::Fixed(height), hwnd)
    }

    #[inline]
    pub fn add_fixed_with_width(&mut self, width: i32, height: i32, hwnd: Option<isize>) -> &mut Self {
        self.items.push(VStackItem {
            length: Length::Fixed(height),
            hwnd,
            width_override: Some(width),
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

    /// Computes vertical layout positions and applies `SetWindowPos` to any provided HWNDs.
    /// Returns the resolved `Rect` bounding boxes for all items in order.
    pub fn apply(&self) -> Vec<Rect> {
        let n = self.items.len();
        if n == 0 {
            return Vec::new();
        }

        let inner_h = (self.bounds.height - self.padding.top - self.padding.bottom).max(0);
        let total_spacing = ((n - 1) as i32) * self.spacing;

        let mut fixed_sum = 0;
        let mut total_stretch = 0;

        for item in &self.items {
            match item.length {
                Length::Fixed(h) => fixed_sum += h,
                Length::Stretch(weight) => total_stretch += weight,
                Length::Auto => fixed_sum += 0,
            }
        }

        let remaining_h = (inner_h - total_spacing - fixed_sum).max(0);

        let mut cur_y = self.bounds.y + self.padding.top;
        let inner_w = (self.bounds.width - self.padding.left - self.padding.right).max(0);
        let mut results = Vec::with_capacity(n);

        for item in &self.items {
            let item_h = match item.length {
                Length::Fixed(h) => h,
                Length::Stretch(weight) => {
                    if total_stretch > 0 {
                        (remaining_h * (weight as i32)) / (total_stretch as i32)
                    } else {
                        0
                    }
                }
                Length::Auto => 0,
            };

            let item_w = item.width_override.unwrap_or(inner_w).min(inner_w);
            let item_x = match self.align {
                CrossAlign::Stretch => self.bounds.x + self.padding.left,
                CrossAlign::Start => self.bounds.x + self.padding.left,
                CrossAlign::Center => self.bounds.x + self.padding.left + (inner_w - item_w) / 2,
                CrossAlign::End => self.bounds.x + self.bounds.width - self.padding.right - item_w,
            };

            let rect = Rect::new(item_x, cur_y, item_w, item_h);
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

            cur_y += item_h + self.spacing;
        }

        results
    }
}
