//! Kora Control Panel GDI Painting & Custom Card Rendering

use crate::ui::colors::ThemePalette;
use crate::ui::tokens::{borders, layout, radii, spacing};
use crate::ui::win32::*;

/// Custom paints the Control Panel window background, sidebar, rounded card containers, and borders
pub fn paint_control_panel(hwnd: isize, hdc: isize, is_dark: bool) {
    unsafe {
        let palette = ThemePalette::get(is_dark);

        let mut client_rc = RECT::default();
        GetClientRect(hwnd, &mut client_rc);

        // 1. Fill entire window background (right panel area)
        let bg_brush = CreateSolidBrush(palette.bg_window);
        FillRect(hdc, &client_rc, bg_brush);
        DeleteObject(bg_brush);

        // 2. Fill Sidebar Background (x: 0..230)
        let sidebar_rc = RECT {
            left: 0,
            top: 0,
            right: layout::SIDEBAR_DIVIDER_X,
            bottom: client_rc.bottom,
        };
        let sb_brush = CreateSolidBrush(palette.bg_sidebar);
        FillRect(hdc, &sidebar_rc, sb_brush);
        DeleteObject(sb_brush);

        // 3. Draw subtle vertical column separator line between Sidebar and Right Panel
        let sep_rc = RECT {
            left: 229,
            top: 0,
            right: 230,
            bottom: client_rc.bottom,
        };
        let sep_brush = CreateSolidBrush(palette.border_separator);
        FillRect(hdc, &sep_rc, sep_brush);

        // Subtle horizontal separator line above sidebar footer
        let h_sep_rc = RECT {
            left: 10,
            top: layout::SIDEBAR_SEPARATOR_Y,
            right: 220,
            bottom: layout::SIDEBAR_SEPARATOR_Y + 1,
        };
        FillRect(hdc, &h_sep_rc, sep_brush);
        DeleteObject(sep_brush);

        let controls = crate::ui::controls().or_else(|| {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                as *const crate::ui::views::ControlPanelControls;
            if !ptr.is_null() { Some(&*ptr) } else { None }
        });



        let b_br = CreateSolidBrush(palette.bg_card);
        let b_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
        let ob = SelectObject(hdc, b_br);
        let op = SelectObject(hdc, b_pen);

        let right_bound = client_rc.right - layout::CARD_MARGIN_RIGHT;

        if let Some(c) = controls {
            c.paint_active_view(hwnd, hdc, right_bound, palette);
        } else {
            RoundRect(
                hdc,
                238,
                100,
                client_rc.right - spacing::SPACE_8,
                client_rc.bottom - 52,
                radii::RADIUS_LG,
                radii::RADIUS_LG,
            );
        }

        SelectObject(hdc, ob);
        SelectObject(hdc, op);
        DeleteObject(b_br);
        DeleteObject(b_pen);
    }
}
