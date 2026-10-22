//! Form Layout Solver for Win32 (FormLayout)
//! Perfectly aligns `[Label] : [Control] : [Optional Value/Unit]` rows across a unified vertical baseline.

use super::types::{Padding, Rect};
use crate::ui_framework::win32::SetWindowPos;

const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_NOOWNERZORDER: u32 = 0x0200;

#[derive(Debug, Clone)]
struct FormRowItem {
    label_hwnd: Option<isize>,
    control_hwnd: Option<isize>,
    value_hwnd: Option<isize>,
    value_width: i32,
    row_height_override: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct FormLayout {
    bounds: Rect,
    label_width: i32,
    col_gap: i32,
    row_spacing: i32,
    default_row_height: i32,
    padding: Padding,
    rows: Vec<FormRowItem>,
}

impl FormLayout {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            label_width: 130,
            col_gap: 12,
            row_spacing: 10,
            default_row_height: 28,
            padding: Padding::ZERO,
            rows: Vec::new(),
        }
    }

    #[inline]
    pub fn at(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self::new(Rect::new(x, y, width, height))
    }

    #[inline]
    pub fn label_width(&mut self, width: i32) -> &mut Self {
        self.label_width = width;
        self
    }

    #[inline]
    pub fn col_gap(&mut self, gap: i32) -> &mut Self {
        self.col_gap = gap;
        self
    }

    #[inline]
    pub fn row_spacing(&mut self, spacing: i32) -> &mut Self {
        self.row_spacing = spacing;
        self
    }

    #[inline]
    pub fn row_height(&mut self, height: i32) -> &mut Self {
        self.default_row_height = height;
        self
    }

    #[inline]
    pub fn padding(&mut self, padding: Padding) -> &mut Self {
        self.padding = padding;
        self
    }

    #[inline]
    pub fn add_row(&mut self, label_hwnd: Option<isize>, control_hwnd: Option<isize>) -> &mut Self {
        self.rows.push(FormRowItem {
            label_hwnd,
            control_hwnd,
            value_hwnd: None,
            value_width: 0,
            row_height_override: None,
        });
        self
    }

    #[inline]
    pub fn row(&mut self, label_hwnd: Option<isize>, control_hwnd: Option<isize>) -> &mut Self {
        self.add_row(label_hwnd, control_hwnd)
    }

    #[inline]
    pub fn add_row_with_value(
        &mut self,
        label_hwnd: Option<isize>,
        control_hwnd: Option<isize>,
        value_hwnd: Option<isize>,
        value_width: i32,
    ) -> &mut Self {
        self.rows.push(FormRowItem {
            label_hwnd,
            control_hwnd,
            value_hwnd,
            value_width,
            row_height_override: None,
        });
        self
    }

    #[inline]
    pub fn row_with_value(
        &mut self,
        label_hwnd: Option<isize>,
        control_hwnd: Option<isize>,
        value_hwnd: Option<isize>,
        value_width: i32,
    ) -> &mut Self {
        self.add_row_with_value(label_hwnd, control_hwnd, value_hwnd, value_width)
    }

    /// Computes layout rectangles for all form rows: `(label_rect, control_rect, Option<value_rect>)`.
    pub fn compute(&self) -> Vec<(Rect, Rect, Option<Rect>)> {
        let inner_w = (self.bounds.width - self.padding.left - self.padding.right).max(0);
        let mut cur_y = self.bounds.y + self.padding.top;
        let mut results = Vec::with_capacity(self.rows.len());

        for (i, row) in self.rows.iter().enumerate() {
            let row_h = row.row_height_override.unwrap_or(self.default_row_height);

            // 1. Label Position
            let lbl_x = self.bounds.x + self.padding.left;
            let lbl_w = self.label_width.min(inner_w);
            let lbl_rect = Rect::new(lbl_x, cur_y + 4, lbl_w, (row_h - 4).max(0));

            // 2. Control Position
            let ctrl_x = lbl_x + lbl_w + self.col_gap;
            let val_space = if row.value_width > 0 {
                row.value_width + self.col_gap
            } else {
                0
            };
            let ctrl_w = (self.bounds.x + self.bounds.width - self.padding.right - ctrl_x - val_space).max(0);
            let ctrl_rect = Rect::new(ctrl_x, cur_y, ctrl_w, row_h);

            // 3. Optional Value
            let val_rect = if row.value_width > 0 {
                let val_x = ctrl_x + ctrl_w + self.col_gap;
                Some(Rect::new(val_x, cur_y + 4, row.value_width, (row_h - 4).max(0)))
            } else {
                None
            };

            results.push((lbl_rect, ctrl_rect, val_rect));

            cur_y += row_h;
            if i + 1 < self.rows.len() {
                cur_y += self.row_spacing;
            }
        }

        results
    }

    /// Computes positions and applies `SetWindowPos` to all row labels and controls.
    /// Returns total height consumed by the form rows.
    pub fn apply(&self) -> i32 {
        let rects = self.compute();
        let flags = SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER;

        for (i, row) in self.rows.iter().enumerate() {
            let (lbl_rect, ctrl_rect, val_rect) = rects[i];

            if let Some(hwnd) = row.label_hwnd {
                if hwnd != 0 {
                    unsafe {
                        SetWindowPos(
                            hwnd,
                            0,
                            lbl_rect.x,
                            lbl_rect.y,
                            lbl_rect.width,
                            lbl_rect.height,
                            flags,
                        );
                    }
                }
            }

            if let Some(hwnd) = row.control_hwnd {
                if hwnd != 0 {
                    unsafe {
                        SetWindowPos(
                            hwnd,
                            0,
                            ctrl_rect.x,
                            ctrl_rect.y,
                            ctrl_rect.width,
                            ctrl_rect.height,
                            flags,
                        );
                    }
                }
            }

            if let (Some(hwnd), Some(v_rect)) = (row.value_hwnd, val_rect) {
                if hwnd != 0 {
                    unsafe {
                        SetWindowPos(
                            hwnd,
                            0,
                            v_rect.x,
                            v_rect.y,
                            v_rect.width,
                            v_rect.height,
                            flags,
                        );
                    }
                }
            }
        }

        if let Some(last) = rects.last() {
            (last.1.y + last.1.height) - self.bounds.y
        } else {
            0
        }
    }
}
