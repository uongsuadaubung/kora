//! 2D Grid Layout Solver for Win32 (GridLayout)
//! Arranges controls in an M x N matrix with uniform column widths and customizable gaps.

use super::types::{Padding, Rect};
use crate::ui_framework::win32::SetWindowPos;

const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_NOOWNERZORDER: u32 = 0x0200;

#[derive(Debug, Clone)]
struct GridCell {
    row: usize,
    col: usize,
    col_span: usize,
    row_span: usize,
    hwnd: Option<isize>,
}

#[derive(Debug, Clone)]
pub struct GridLayout {
    bounds: Rect,
    cols: usize,
    rows: usize,
    gap_x: i32,
    gap_y: i32,
    row_height: Option<i32>,
    padding: Padding,
    cells: Vec<GridCell>,
}

impl GridLayout {
    pub fn new(bounds: Rect, cols: usize, rows: usize) -> Self {
        Self {
            bounds,
            cols: cols.max(1),
            rows: rows.max(1),
            gap_x: 0,
            gap_y: 0,
            row_height: None,
            padding: Padding::ZERO,
            cells: Vec::new(),
        }
    }

    #[inline]
    pub fn at(x: i32, y: i32, width: i32, height: i32, cols: usize, rows: usize) -> Self {
        Self::new(Rect::new(x, y, width, height), cols, rows)
    }

    #[inline]
    pub fn gap(&mut self, gap_x: i32, gap_y: i32) -> &mut Self {
        self.gap_x = gap_x;
        self.gap_y = gap_y;
        self
    }

    #[inline]
    pub fn row_height(&mut self, height: i32) -> &mut Self {
        self.row_height = Some(height);
        self
    }

    #[inline]
    pub fn padding(&mut self, padding: Padding) -> &mut Self {
        self.padding = padding;
        self
    }

    #[inline]
    pub fn add(&mut self, row: usize, col: usize, hwnd: Option<isize>) -> &mut Self {
        self.cells.push(GridCell {
            row,
            col,
            col_span: 1,
            row_span: 1,
            hwnd,
        });
        self
    }

    #[inline]
    pub fn cell(&mut self, row: usize, col: usize, hwnd: Option<isize>) -> &mut Self {
        self.add(row, col, hwnd)
    }

    #[inline]
    pub fn add_span(&mut self, row: usize, col: usize, col_span: usize, row_span: usize, hwnd: Option<isize>) -> &mut Self {
        self.cells.push(GridCell {
            row,
            col,
            col_span: col_span.max(1),
            row_span: row_span.max(1),
            hwnd,
        });
        self
    }

    #[inline]
    pub fn cell_span(&mut self, row: usize, col: usize, col_span: usize, row_span: usize, hwnd: Option<isize>) -> &mut Self {
        self.add_span(row, col, col_span, row_span, hwnd)
    }

    /// Computes grid positions and applies `SetWindowPos` to any provided HWNDs.
    /// Returns the resolved `Rect` bounding boxes for all cells in order of addition.
    pub fn apply(&self) -> Vec<Rect> {
        let inner_w = (self.bounds.width - self.padding.left - self.padding.right).max(0);
        let total_gap_x = ((self.cols - 1) as i32) * self.gap_x;
        let col_w = ((inner_w - total_gap_x) / (self.cols as i32)).max(0);

        let default_row_h = if let Some(rh) = self.row_height {
            rh
        } else {
            let inner_h = (self.bounds.height - self.padding.top - self.padding.bottom).max(0);
            let total_gap_y = ((self.rows - 1) as i32) * self.gap_y;
            ((inner_h - total_gap_y) / (self.rows as i32)).max(0)
        };

        let mut results = Vec::with_capacity(self.cells.len());

        for cell in &self.cells {
            let c = cell.col.min(self.cols - 1);
            let r = cell.row.min(self.rows - 1);
            let span_x = cell.col_span.min(self.cols - c);
            let span_y = cell.row_span.min(self.rows - r);

            let cell_x = self.bounds.x + self.padding.left + (c as i32) * (col_w + self.gap_x);
            let cell_y = self.bounds.y + self.padding.top + (r as i32) * (default_row_h + self.gap_y);
            let cell_w = (span_x as i32) * col_w + ((span_x - 1) as i32) * self.gap_x;
            let cell_h = (span_y as i32) * default_row_h + ((span_y - 1) as i32) * self.gap_y;

            let rect = Rect::new(cell_x, cell_y, cell_w, cell_h);
            results.push(rect);

            if let Some(hwnd) = cell.hwnd {
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
        }

        results
    }
}
