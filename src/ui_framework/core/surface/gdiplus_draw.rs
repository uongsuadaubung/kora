//! High-Performance GDI+ RAII Painter Wrappers
//!
//! Cung cấp các công cụ vẽ vector khử răng cưa (Antialiased Graphics) tự động giải phóng tài nguyên,
//! hỗ trợ bo góc mượt mà (Smooth RoundRect), đổ bóng bán trong suốt (Alpha Drop Shadow) và hình tròn hoàn hảo.

use crate::ui_framework::win32::*;

/// RAII wrapper quản lý vòng đời của `GpGraphics` trên nền HDC.
pub struct GdiGraphics(isize);

impl GdiGraphics {
    /// Khởi tạo GDI+ Graphics gắn với Win32 Device Context (`hdc`).
    pub fn new(hdc: isize) -> Option<Self> {
        if hdc == 0 || !ensure_gdiplus_initialized() {
            return None;
        }
        let mut graphics: isize = 0;
        let status = unsafe { GdipCreateFromHDC(hdc, &mut graphics) };
        if status != 0 || graphics == 0 {
            return None;
        }
        unsafe {
            GdipSetSmoothingMode(graphics, SMOOTHING_MODE_ANTI_ALIAS);
            GdipSetPixelOffsetMode(graphics, PIXEL_OFFSET_MODE_HALF);
        }
        Some(Self(graphics))
    }

    #[inline]
    pub fn raw(&self) -> isize {
        self.0
    }

    /// Vẽ hình chữ nhật đặc (Solid Rectangle).
    pub fn fill_rect(&self, x: f32, y: f32, w: f32, h: f32, color_argb: u32) {
        if let Some(brush) = GdiSolidBrush::new(color_argb) {
            unsafe {
                GdipFillRectangle(self.0, brush.raw(), x, y, w, h);
            }
        }
    }

    /// Vẽ viền hình chữ nhật (Stroke Rectangle).
    pub fn draw_rect(&self, x: f32, y: f32, w: f32, h: f32, color_argb: u32, width: f32) {
        if let Some(pen) = GdiPen::new(color_argb, width) {
            unsafe {
                GdipDrawRectangle(self.0, pen.raw(), x, y, w, h);
            }
        }
    }

    /// Vẽ hình tròn hoặc ellipse đặc với khử răng cưa mượt mà.
    pub fn fill_ellipse(&self, x: f32, y: f32, w: f32, h: f32, color_argb: u32) {
        if let Some(brush) = GdiSolidBrush::new(color_argb) {
            unsafe {
                GdipFillEllipse(self.0, brush.raw(), x, y, w, h);
            }
        }
    }

    /// Vẽ viền hình tròn hoặc ellipse với khử răng cưa mượt mà.
    pub fn draw_ellipse(&self, x: f32, y: f32, w: f32, h: f32, color_argb: u32, width: f32) {
        if let Some(pen) = GdiPen::new(color_argb, width) {
            unsafe {
                GdipDrawEllipse(self.0, pen.raw(), x, y, w, h);
            }
        }
    }

    /// Vẽ hình chữ nhật bo góc khử răng cưa (Smooth Antialiased RoundRect).
    /// Áp dụng thuật toán nối 4 cung tròn (Arc) chuẩn xác từng subpixel.
    pub fn round_rect(
        &self,
        rc: &RECT,
        radius: f32,
        fill_color: Option<u32>,
        stroke_color: Option<(u32, f32)>,
    ) {
        let w = (rc.right - rc.left) as f32;
        let h = (rc.bottom - rc.top) as f32;
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        let x = rc.left as f32;
        let y = rc.top as f32;

        if radius <= 1.0 {
            if let Some(f_col) = fill_color {
                self.fill_rect(x, y, w, h, f_col);
            }
            if let Some((s_col, s_w)) = stroke_color {
                self.draw_rect(x + 0.5, y + 0.5, w - 1.0, h - 1.0, s_col, s_w);
            }
            return;
        }

        let max_r = (w.min(h) / 2.0).max(1.0);
        let r = radius.min(max_r);
        let d = r * 2.0;

        let Some(path) = GdiPath::new() else { return };
        unsafe {
            // Arc 1: Top-Left
            GdipAddPathArc(path.raw(), x, y, d, d, 180.0, 90.0);
            // Arc 2: Top-Right
            GdipAddPathArc(path.raw(), x + w - d, y, d, d, 270.0, 90.0);
            // Arc 3: Bottom-Right
            GdipAddPathArc(path.raw(), x + w - d, y + h - d, d, d, 0.0, 90.0);
            // Arc 4: Bottom-Left
            GdipAddPathArc(path.raw(), x, y + h - d, d, d, 90.0, 90.0);
            GdipClosePathFigure(path.raw());

            if let Some(f_col) = fill_color
                && let Some(brush) = GdiSolidBrush::new(f_col)
            {
                GdipFillPath(self.0, brush.raw(), path.raw());
            }

            if let Some((s_col, s_w)) = stroke_color
                && let Some(pen) = GdiPen::new(s_col, s_w)
            {
                GdipDrawPath(self.0, pen.raw(), path.raw());
            }
        }
    }
}

impl Drop for GdiGraphics {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                GdipDeleteGraphics(self.0);
            }
        }
    }
}

/// RAII wrapper cho `GpSolidFill` brush.
pub struct GdiSolidBrush(isize);

impl GdiSolidBrush {
    pub fn new(color_argb: u32) -> Option<Self> {
        let mut brush: isize = 0;
        let status = unsafe { GdipCreateSolidFill(color_argb, &mut brush) };
        if status == 0 && brush != 0 {
            Some(Self(brush))
        } else {
            None
        }
    }

    #[inline]
    pub fn raw(&self) -> isize {
        self.0
    }
}

impl Drop for GdiSolidBrush {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                GdipDeleteBrush(self.0);
            }
        }
    }
}

/// RAII wrapper cho `GpPen`.
pub struct GdiPen(isize);

impl GdiPen {
    pub fn new(color_argb: u32, width: f32) -> Option<Self> {
        let mut pen: isize = 0;
        let status = unsafe { GdipCreatePen1(color_argb, width, UNIT_PIXEL, &mut pen) };
        if status == 0 && pen != 0 {
            Some(Self(pen))
        } else {
            None
        }
    }

    #[inline]
    pub fn raw(&self) -> isize {
        self.0
    }
}

impl Drop for GdiPen {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                GdipDeletePen(self.0);
            }
        }
    }
}

/// RAII wrapper cho `GpPath`.
pub struct GdiPath(isize);

impl GdiPath {
    pub fn new() -> Option<Self> {
        let mut path: isize = 0;
        let status = unsafe { GdipCreatePath(FILL_MODE_ALTERNATE, &mut path) };
        if status == 0 && path != 0 {
            Some(Self(path))
        } else {
            None
        }
    }

    #[inline]
    pub fn raw(&self) -> isize {
        self.0
    }
}

impl Drop for GdiPath {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                GdipDeletePath(self.0);
            }
        }
    }
}
