//! ActionTable GDI Rendering Engine
//!
//! Chịu trách nhiệm toàn bộ việc vẽ đồ họa:
//! - Tính toán layout cột (`compute_column_rects`) và con trượt scrollbar (`compute_scroll_thumb_rect`).
//! - Cơ chế vẽ chống giật nháy màn hình (Double Buffering qua Memory DC).
//! - Vẽ các dòng dữ liệu (Text, SecondaryText, Badge phím bấm, Action buttons, Custom renderers).
//! - Vẽ fixed header cố định và viền phân cách.

use super::types::*;
use crate::ui::colors::ThemePalette;
use crate::ui::tokens::borders;
use crate::ui::win32::*;

pub fn pt_in_rect(x: i32, y: i32, rc: &RECT) -> bool {
    x >= rc.left && x <= rc.right && y >= rc.top && y <= rc.bottom
}

pub fn get_dynamic_action_button_rects(rc: &RECT, count: usize) -> Vec<RECT> {
    if count == 0 {
        return Vec::new();
    }
    let (btn_width, spacing) = match count {
        1 => (46, 0),
        2 => (38, 6),
        _ => (34, 6),
    };
    let btn_height = (rc.bottom - rc.top).saturating_sub(6).max(18);
    let top_offset = rc.top + ((rc.bottom - rc.top) - btn_height) / 2;
    let col_w = rc.right - rc.left;
    let total_w = (btn_width * count as i32) + (spacing * (count as i32 - 1));
    let start_x = rc.left + ((col_w - total_w) / 2).max(4);

    let mut rects = Vec::with_capacity(count);
    for i in 0..count {
        let left = start_x + (i as i32 * (btn_width + spacing));
        rects.push(RECT {
            left,
            top: top_offset,
            right: left + btn_width,
            bottom: top_offset + btn_height,
        });
    }
    rects
}

pub fn compute_column_rects(columns: &[ColumnDef], client_width: i32) -> Vec<(i32, i32)> {
    let mut fixed_sum = 0;
    let mut flex_count = 0;
    for col in columns {
        match col.width {
            ColumnWidth::Fixed(w) => fixed_sum += w,
            ColumnWidth::Flex => flex_count += 1,
        }
    }
    let remaining_w = (client_width - fixed_sum).max(0);
    let flex_w = if flex_count > 0 {
        remaining_w / flex_count
    } else {
        0
    };

    let mut rects = Vec::with_capacity(columns.len());
    let mut current_x = 0;
    for (i, col) in columns.iter().enumerate() {
        let is_last = i == columns.len() - 1;
        let w = match col.width {
            ColumnWidth::Fixed(w) => {
                if is_last && flex_count == 0 {
                    (client_width - current_x).max(w)
                } else {
                    w
                }
            }
            ColumnWidth::Flex => {
                if is_last {
                    (client_width - current_x).max(60)
                } else {
                    flex_w
                }
            }
        };
        rects.push((current_x, w));
        current_x += w;
    }
    rects
}

pub fn compute_scroll_thumb_rect(
    width: i32,
    height: i32,
    header_h: i32,
    total_rows: usize,
    scroll_y: i32,
    is_expanded: bool,
) -> Option<(RECT, i32)> {
    let body_h = (height - header_h).max(0);
    let total_h = total_rows as i32 * ROW_HEIGHT;
    let max_scroll = total_h - body_h;
    if max_scroll <= 0 || body_h <= 0 {
        return None;
    }
    let track_top = header_h + 3;
    let track_bottom = height - 3;
    let track_h = (track_bottom - track_top).max(10);
    let thumb_h = ((body_h as f32 / total_h as f32) * track_h as f32).clamp(18.0, track_h as f32) as i32;
    let travel = (track_h - thumb_h).max(1);
    let thumb_top = track_top + ((scroll_y as f32 / max_scroll as f32) * travel as f32) as i32;

    let bar_w = if is_expanded { 5 } else { 3 };
    let thumb_rc = RECT {
        left: width - bar_w - 3,
        top: thumb_top,
        right: width - 3,
        bottom: thumb_top + thumb_h,
    };
    Some((thumb_rc, max_scroll))
}

pub fn render_table(hwnd: isize, hdc: isize, state: &ActionTableState) {
    let is_dark = crate::ui::is_current_dark();
    let palette = ThemePalette::get(is_dark);

    let mut client_rc = RECT::default();
    unsafe { GetClientRect(hwnd, &mut client_rc); }
    let width = client_rc.right - client_rc.left;
    let height = client_rc.bottom - client_rc.top;
    if width <= 0 || height <= 0 {
        return;
    }

    unsafe {
        // Double buffering: Create compatible DC and Bitmap
        let mem_dc = CreateCompatibleDC(hdc);
        let mem_bmp = CreateCompatibleBitmap(hdc, width, height);
        let old_bmp = SelectObject(mem_dc, mem_bmp);

        let col_rects = compute_column_rects(&state.columns, width);

        // 1. Fill entire background
        let bg_br = CreateSolidBrush(palette.bg_input);
        FillRect(mem_dc, &client_rc, bg_br);
        DeleteObject(bg_br);

        // 2. Render visible Body rows
        let header_h = state.header_h;
        let body_h = (height - header_h).max(0);
        let first_row = (state.scroll_y / ROW_HEIGHT).max(0) as usize;
        let last_row = ((state.scroll_y + body_h + ROW_HEIGHT) / ROW_HEIGHT) as usize;
        let visible_range = first_row..last_row.min(state.rows.len());

        let app_font = if state.font != 0 { state.font } else { crate::ui::theme::get_app_font_normal() };
        let app_font_bold = if state.font_bold != 0 { state.font_bold } else { crate::ui::theme::get_app_font_bold() };

        for row_idx in visible_range {
            let row_y = header_h + (row_idx as i32 * ROW_HEIGHT) - state.scroll_y;
            let is_selected = state.selected_row == Some(row_idx);
            let is_hot = state.hover_row == Some(row_idx);

            let row_bg_color = if is_selected {
                palette.row_selected_bg
            } else if is_hot {
                palette.row_hover_bg
            } else {
                palette.bg_input
            };

            let row_rc = RECT {
                left: 0,
                top: row_y,
                right: width,
                bottom: row_y + ROW_HEIGHT - borders::WIDTH_THIN,
            };
            let r_br = CreateSolidBrush(row_bg_color);
            FillRect(mem_dc, &row_rc, r_br);
            DeleteObject(r_br);

            // Horizontal row bottom divider
            let div_rc = RECT {
                left: 0,
                top: row_y + ROW_HEIGHT - borders::WIDTH_THIN,
                right: width,
                bottom: row_y + ROW_HEIGHT,
            };
            let div_br = CreateSolidBrush(palette.border_separator);
            FillRect(mem_dc, &div_rc, div_br);
            DeleteObject(div_br);

            // Render columns for this row
            if let Some(cells) = state.rows.get(row_idx) {
                for (col_idx, &(col_x, col_w)) in col_rects.iter().enumerate() {
                    let is_last_col = col_idx == col_rects.len() - 1;
                    let cell_rc = RECT {
                        left: col_x,
                        top: row_y,
                        right: col_x + col_w,
                        bottom: row_y + ROW_HEIGHT - borders::WIDTH_THIN,
                    };

                    // Vertical column separator line
                    if !is_last_col {
                        let sep_rc = RECT {
                            left: cell_rc.right - borders::WIDTH_THIN,
                            top: cell_rc.top,
                            right: cell_rc.right,
                            bottom: cell_rc.bottom + borders::WIDTH_THIN,
                        };
                        let sep_br = CreateSolidBrush(palette.border_separator);
                        FillRect(mem_dc, &sep_rc, sep_br);
                        DeleteObject(sep_br);
                    }

                    if let Some(cell) = cells.get(col_idx) {
                        match cell {
                            TableCell::Badge(cfg) => {
                                if cfg.text.trim().is_empty() {
                                    continue;
                                }
                                let (bg_clr, border_clr, text_clr) = match cfg.variant {
                                    BadgeVariant::KeyPill => (palette.bg_card, palette.border, palette.text_link),
                                    BadgeVariant::Danger => (palette.danger_bg, palette.danger_border, palette.danger_text),
                                };

                                let wide_text = to_wide(&cfg.text);
                                let old_f = SelectObject(mem_dc, app_font_bold);
                                SetBkMode(mem_dc, TRANSPARENT);

                                let mut sz = SIZE { cx: 0, cy: 0 };
                                GetTextExtentPoint32W(mem_dc, wide_text.as_ptr(), wide_text.len().saturating_sub(1) as i32, &mut sz);

                                let has_suffix = cfg.suffix.is_some();
                                let suffix_reserve = if has_suffix { 28 } else { 0 };
                                let max_w = (cell_rc.right - cell_rc.left).saturating_sub(16 + suffix_reserve);
                                let pill_w = (sz.cx + 16).min(max_w).max(28);
                                let pill_h = 20;
                                let top_offset = cell_rc.top + ((cell_rc.bottom - cell_rc.top) - pill_h) / 2;

                                let pill_rc = RECT {
                                    left: cell_rc.left + 8,
                                    top: top_offset,
                                    right: cell_rc.left + 8 + pill_w,
                                    bottom: top_offset + pill_h,
                                };

                                let br = CreateSolidBrush(bg_clr);
                                let pen = CreatePen(PS_SOLID, 1, border_clr);
                                let ob = SelectObject(mem_dc, br);
                                let op = SelectObject(mem_dc, pen);
                                RoundRect(mem_dc, pill_rc.left, pill_rc.top, pill_rc.right, pill_rc.bottom, 4, 4);
                                SelectObject(mem_dc, ob);
                                SelectObject(mem_dc, op);
                                DeleteObject(br);
                                DeleteObject(pen);

                                SetTextColor(mem_dc, text_clr);
                                let mut text_rc = pill_rc;
                                DrawTextW(mem_dc, wide_text.as_ptr(), -1, &mut text_rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                                if let Some(ref sfx) = cfg.suffix {
                                    SetTextColor(mem_dc, palette.text_secondary);
                                    let mut arrow_rc = RECT {
                                        left: cell_rc.right - 24,
                                        top: cell_rc.top,
                                        right: cell_rc.right - 4,
                                        bottom: cell_rc.bottom,
                                    };
                                    let arrow_text = to_wide(sfx);
                                    DrawTextW(mem_dc, arrow_text.as_ptr(), -1, &mut arrow_rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                                }
                                SelectObject(mem_dc, old_f);
                            }
                            TableCell::Text(text) => {
                                if text.is_empty() {
                                    continue;
                                }
                                let wide_text = to_wide(text);
                                SetBkMode(mem_dc, TRANSPARENT);
                                let text_clr = if is_selected {
                                    palette.text_white
                                } else {
                                    palette.text_primary
                                };
                                SetTextColor(mem_dc, text_clr);
                                let old_f = SelectObject(mem_dc, app_font);
                                let mut text_rc = RECT {
                                    left: cell_rc.left + 8,
                                    top: cell_rc.top,
                                    right: cell_rc.right - 8,
                                    bottom: cell_rc.bottom,
                                };
                                let col_def = &state.columns[col_idx];
                                let align_flag = match col_def.align {
                                    CellAlign::Left => DT_LEFT,
                                    CellAlign::Center => DT_CENTER,
                                    CellAlign::Right => DT_RIGHT,
                                };
                                DrawTextW(mem_dc, wide_text.as_ptr(), -1, &mut text_rc, align_flag | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);
                                SelectObject(mem_dc, old_f);
                            }
                            TableCell::SecondaryText(text) => {
                                let wide_text = to_wide(text);
                                SetBkMode(mem_dc, TRANSPARENT);
                                SetTextColor(mem_dc, palette.text_secondary);
                                let old_f = SelectObject(mem_dc, app_font);
                                let mut text_rc = RECT {
                                    left: cell_rc.left + 4,
                                    top: cell_rc.top,
                                    right: cell_rc.right - 4,
                                    bottom: cell_rc.bottom,
                                };
                                DrawTextW(mem_dc, wide_text.as_ptr(), -1, &mut text_rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                                SelectObject(mem_dc, old_f);
                            }
                            TableCell::Actions(actions) => {
                                let old_f = SelectObject(mem_dc, app_font_bold);
                                SetBkMode(mem_dc, TRANSPARENT);

                                let btn_rects = get_dynamic_action_button_rects(&cell_rc, actions.len());
                                for (act_idx, act) in actions.iter().enumerate() {
                                    if let Some(rc_btn) = btn_rects.get(act_idx) {
                                        let is_h = state.hover_action == ActionHover::Action { row: row_idx, action_idx: act_idx };
                                        let (bg_clr, border_clr, text_clr) = match act.variant {
                                            ActionVariant::Primary => {
                                                if is_h {
                                                    (palette.accent, palette.accent, palette.text_white)
                                                } else {
                                                    (palette.badge_root_bg, palette.accent, palette.text_link)
                                                }
                                            }
                                            ActionVariant::Danger => {
                                                if is_h {
                                                    (palette.danger_border, palette.danger_border, palette.text_white)
                                                } else {
                                                    (palette.danger_bg, palette.danger_border, palette.danger_text)
                                                }
                                            }
                                            ActionVariant::Secondary => {
                                                if is_h {
                                                    (palette.border_separator, palette.border, palette.text_primary)
                                                } else {
                                                    (palette.bg_card, palette.border, palette.text_secondary)
                                                }
                                            }
                                        };

                                        let br = CreateSolidBrush(bg_clr);
                                        let pen = CreatePen(PS_SOLID, 1, border_clr);
                                        let ob = SelectObject(mem_dc, br);
                                        let op = SelectObject(mem_dc, pen);
                                        RoundRect(mem_dc, rc_btn.left, rc_btn.top, rc_btn.right, rc_btn.bottom, 4, 4);
                                        SelectObject(mem_dc, ob);
                                        SelectObject(mem_dc, op);
                                        DeleteObject(br);
                                        DeleteObject(pen);

                                        SetTextColor(mem_dc, text_clr);
                                        let mut tr_btn = *rc_btn;
                                        let text_w = to_wide(&act.label);
                                        DrawTextW(mem_dc, text_w.as_ptr(), -1, &mut tr_btn, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                                    }
                                }
                                SelectObject(mem_dc, old_f);
                            }
                            TableCell::Custom(painter) => {
                                painter(mem_dc, &cell_rc, palette);
                            }
                        }
                    }
                }
            }
        }

        // 3. Render Fixed Header on top (0..header_h), skipped when hidden
        if header_h > 0 {
            let hdr_rc = RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: header_h,
            };
            let hdr_br = CreateSolidBrush(palette.bg_card_header);
            FillRect(mem_dc, &hdr_rc, hdr_br);
            DeleteObject(hdr_br);

            // Header bottom divider line
            let hdr_div = RECT {
                left: 0,
                top: header_h - borders::WIDTH_THIN,
                right: width,
                bottom: header_h,
            };
            let h_div_br = CreateSolidBrush(palette.border_separator);
            FillRect(mem_dc, &hdr_div, h_div_br);
            DeleteObject(h_div_br);

            // Header column titles & vertical separators
            let old_f = SelectObject(mem_dc, app_font_bold);
            SetBkMode(mem_dc, TRANSPARENT);
            SetTextColor(mem_dc, palette.text_secondary);

            for (col_idx, &(col_x, col_w)) in col_rects.iter().enumerate() {
                let is_last_col = col_idx == col_rects.len() - 1;
                let col_def = &state.columns[col_idx];

                if !is_last_col {
                    let h_sep = RECT {
                        left: col_x + col_w - borders::WIDTH_THIN,
                        top: 4,
                        right: col_x + col_w,
                        bottom: header_h - 4,
                    };
                    let sep_br = CreateSolidBrush(palette.border_separator);
                    FillRect(mem_dc, &h_sep, sep_br);
                    DeleteObject(sep_br);
                }

                let mut title_rc = RECT {
                    left: col_x + 8,
                    top: 0,
                    right: col_x + col_w - 8,
                    bottom: header_h - borders::WIDTH_THIN,
                };
                let fmt = match col_def.align {
                    CellAlign::Left => DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    CellAlign::Center => DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    CellAlign::Right => DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
                };
                let wide_title = to_wide(&col_def.title);
                DrawTextW(mem_dc, wide_title.as_ptr(), -1, &mut title_rc, fmt);
            }
            SelectObject(mem_dc, old_f);
        }

        // 4. Render slim modern scrollbar overlay
        let is_expanded = state.is_dragging_scroll || state.is_hover_scrollbar;
        if let Some((thumb_rc, _)) = compute_scroll_thumb_rect(width, height, header_h, state.rows.len(), state.scroll_y, is_expanded) {
            let bar_color = if is_expanded {
                palette.text_secondary
            } else {
                palette.border_hover
            };

            let br = CreateSolidBrush(bar_color);
            let pen = CreatePen(PS_SOLID, 1, bar_color);
            let ob = SelectObject(mem_dc, br);
            let op = SelectObject(mem_dc, pen);
            RoundRect(mem_dc, thumb_rc.left, thumb_rc.top, thumb_rc.right, thumb_rc.bottom, 4, 4);
            SelectObject(mem_dc, ob);
            SelectObject(mem_dc, op);
            DeleteObject(br);
            DeleteObject(pen);
        }

        // 5. Draw outer border
        let b_rc = RECT { left: 0, top: 0, right: width, bottom: height };
        let border_br = CreateSolidBrush(palette.border);
        FrameRect(mem_dc, &b_rc, border_br);
        DeleteObject(border_br);

        // 6. BitBlt entire buffer to window DC
        BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

        SelectObject(mem_dc, old_bmp);
        DeleteObject(mem_bmp);
        DeleteDC(mem_dc);
    }
}

