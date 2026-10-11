//! Kora Control Panel GDI Painting & Custom Card Rendering

use crate::ui::colors::ThemePalette;
use crate::ui::win32::*;
use crate::ui::tokens::{borders, layout, radii, spacing};

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
            top: crate::ui::tokens::layout::SIDEBAR_SEPARATOR_Y,
            right: 220,
            bottom: crate::ui::tokens::layout::SIDEBAR_SEPARATOR_Y + 1,
        };
        FillRect(hdc, &h_sep_rc, sep_brush);
        DeleteObject(sep_brush);

        let controls_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const crate::ui::views::ControlPanelControls;
        let mode = if !controls_ptr.is_null() {
            (*controls_ptr).get_view_mode()
        } else {
            crate::ui::views::control_panel::ViewMode::Profile(0)
        };

        if matches!(
            mode,
            crate::ui::views::control_panel::ViewMode::SystemSettings
                | crate::ui::views::control_panel::ViewMode::About
        ) {
            let h_sep = RECT {
                left: crate::ui::tokens::layout::SIDEBAR_DIVIDER_X,
                top: crate::ui::tokens::layout::HEADER_DIVIDER_Y,
                right: client_rc.right,
                bottom: crate::ui::tokens::layout::HEADER_DIVIDER_Y + 1,
            };
            let sep_brush = CreateSolidBrush(palette.border_separator);
            FillRect(hdc, &h_sep, sep_brush);
            DeleteObject(sep_brush);
        }

        let b_br = CreateSolidBrush(palette.bg_card);
        let b_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
        let ob = SelectObject(hdc, b_br);
        let op = SelectObject(hdc, b_pen);

        let right_bound = client_rc.right - layout::CARD_MARGIN_RIGHT;

        match mode {
            crate::ui::views::control_panel::ViewMode::Profile(0) => {
                // Card 1: Thiết lập Bộ gõ tiếng Việt
                RoundRect(hdc, layout::CARD_X, layout::CARD_START_Y, right_bound, 218, radii::RADIUS_CARD, radii::RADIUS_CARD);

                // Card 2: Phím chuyển đổi E ⇄ V
                RoundRect(hdc, layout::CARD_X, 218 + layout::CARD_GAP, right_bound, 310, radii::RADIUS_CARD, radii::RADIUS_CARD);

                // Card 3: Tính năng gõ thông minh
                RoundRect(hdc, layout::CARD_X, 310 + layout::CARD_GAP, right_bound, 416, radii::RADIUS_CARD, radii::RADIUS_CARD);

                // Inside Card 2: Key Badge pill box for Hotkey
                let badge_br = CreateSolidBrush(palette.bg_input);
                let badge_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
                let old_b = SelectObject(hdc, badge_br);
                let old_p = SelectObject(hdc, badge_pen);
                // Pill hugs the auto-sized badge label (2px horizontal, 6px vertical padding)
                let mut br = RECT { left: 372, top: 270, right: 488, bottom: 288 };
                let badge = GetDlgItem(hwnd, crate::ui::tokens::id::IDC_LABEL_SWITCH_KEY_BADGE as i32);
                if badge != 0 {
                    GetWindowRect(badge, &mut br);
                    let mut tl = POINT { x: br.left, y: br.top };
                    let mut bt = POINT { x: br.right, y: br.bottom };
                    ScreenToClient(hwnd, &mut tl);
                    ScreenToClient(hwnd, &mut bt);
                    br = RECT { left: tl.x, top: tl.y, right: bt.x, bottom: bt.y };
                }
                RoundRect(hdc, br.left - 2, br.top - 6, br.right + 2, br.bottom + 6, radii::RADIUS_MD, radii::RADIUS_MD);
                SelectObject(hdc, old_b);
                SelectObject(hdc, old_p);
                DeleteObject(badge_br);
                DeleteObject(badge_pen);
            }
            crate::ui::views::control_panel::ViewMode::Profile(1) => {
                // One card below the sub-tab bar (Đổi phím / Chặn phím share it)
                let top = crate::ui::views::control_panel::tab_remap::SUB_CARD_Y;
                RoundRect(hdc, layout::CARD_X, top, right_bound, layout::CARD_BOTTOM_Y, radii::RADIUS_CARD, radii::RADIUS_CARD);
            }
            crate::ui::views::control_panel::ViewMode::Profile(2) => {
                // Card: Âm thanh gõ phím cơ (Typing Sound)
                RoundRect(hdc, layout::CARD_X, layout::CARD_START_Y, right_bound, 286, radii::RADIUS_CARD, radii::RADIUS_CARD);
            }
            crate::ui::views::control_panel::ViewMode::Profile(3) => {
                // Card 1: 2 Checkboxes (Bật gõ tắt & Gõ tắt trong tiếng Anh)
                const C1_TOP: i32 = layout::CARD_START_Y;
                const C1_BOTTOM: i32 = C1_TOP + 48;
                RoundRect(hdc, layout::CARD_X, C1_TOP, right_bound, C1_BOTTOM, radii::RADIUS_CARD, radii::RADIUS_CARD);

                // Card 2: Toolbar chứa nút "+ Thêm"
                const C2_TOP: i32 = C1_BOTTOM + layout::CARD_GAP;
                const C2_BOTTOM: i32 = C2_TOP + 48;
                RoundRect(hdc, layout::CARD_X, C2_TOP, right_bound, C2_BOTTOM, radii::RADIUS_CARD, radii::RADIUS_CARD);

                // Card 3: Khung viền chứa Bảng gõ tắt (ListView 5 cột)
                const C3_TOP: i32 = C2_BOTTOM + layout::CARD_GAP;
                RoundRect(hdc, layout::CARD_X, C3_TOP, right_bound, layout::CARD_BOTTOM_Y, radii::RADIUS_CARD, radii::RADIUS_CARD);
            }
            crate::ui::views::control_panel::ViewMode::SystemSettings => {
                // Card 1: Hệ thống Windows
                RoundRect(hdc, 246, 86, right_bound, 232, radii::RADIUS_CARD, radii::RADIUS_CARD);

                // Card 2: Giao diện (Theme) & Ngôn ngữ
                RoundRect(hdc, 246, 248, right_bound, 368, radii::RADIUS_CARD, radii::RADIUS_CARD);
            }
            crate::ui::views::control_panel::ViewMode::About => {
                // Main About Hero Card
                RoundRect(hdc, 246, 86, right_bound, 336, radii::RADIUS_CARD, radii::RADIUS_CARD);

                // Blue Logo Icon Square centered in the card
                let logo_x = 246 + (right_bound - 246 - 48) / 2;
                let logo_br = CreateSolidBrush(palette.accent);
                let logo_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.accent);
                let old_b = SelectObject(hdc, logo_br);
                let old_p = SelectObject(hdc, logo_pen);
                RoundRect(hdc, logo_x, 114, logo_x + 48, 162, radii::RADIUS_LG, radii::RADIUS_LG);
                SelectObject(hdc, old_b);
                SelectObject(hdc, old_p);
                DeleteObject(logo_br);
                DeleteObject(logo_pen);

                // Draw white letter 'K' inside logo box
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, palette.text_white);
                let logo_font = create_app_font(
                    crate::ui::tokens::typography::FONT_SIZE_HERO,
                    FW_BOLD,
                );
                let old_f = SelectObject(hdc, logo_font);
                let mut text_rc = RECT {
                    left: logo_x,
                    top: 114,
                    right: logo_x + 48,
                    bottom: 162,
                };
                DrawTextW(
                    hdc,
                    to_wide("K").as_ptr(),
                    -1,
                    &mut text_rc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
                SelectObject(hdc, old_f);
                DeleteObject(logo_font);
            }
            _ => {
                let body_rc = RECT {
                    left: 238,
                    top: 100,
                    right: client_rc.right - spacing::SPACE_8,
                    bottom: client_rc.bottom - 52,
                };
                RoundRect(
                    hdc,
                    body_rc.left,
                    body_rc.top,
                    body_rc.right,
                    body_rc.bottom,
                    radii::RADIUS_LG,
                    radii::RADIUS_LG,
                );
            }
        }

        SelectObject(hdc, ob);
        SelectObject(hdc, op);
        DeleteObject(b_br);
        DeleteObject(b_pen);
    }
}
