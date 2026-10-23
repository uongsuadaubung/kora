//! Double-Buffered GDI Surface Painter
//!
//! Paints the declarative [`LayoutNode<M>`] tree to a device context with Fluent Design
//! styling, theme palette colors (Dark / Light), hover states, accent borders,
//! drop shadows, and top-layer select dropdown overlays.

use super::types::*;
use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::core::element::*;
use crate::ui_framework::core::layout_engine::*;
use crate::ui_framework::core::style::BgStyle;
use crate::ui_framework::tokens::{radii, spacing};
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::Mutex;

pub(crate) fn paint_surface<M>(
    hdc: isize,
    state_arc: &Mutex<SurfaceInner<M>>,
    palette: &ThemePalette,
    client_rc: &RECT,
) {
    let Ok(guard) = state_arc.lock() else { return };

    // 1. Fill window background
    if let Some(bg_color) = guard.bg.resolve(palette) {
        fill_rect_color(hdc, client_rc, bg_color);
    }

    let scroll_y = guard.scroll_y;
    let viewport_h = client_rc.bottom - client_rc.top;
    let hover_id = guard.hover_node_id;
    let open_select_key = guard.open_select.as_ref().map(|s| s.key);
    let focused_node_id = guard.focused_node_id;

    // 2. Paint layout tree
    if let Some(ref root) = guard.layout_root {
        paint_node(
            hdc,
            root,
            scroll_y,
            viewport_h,
            hover_id,
            open_select_key,
            focused_node_id,
            palette,
        );
    }

    // 3. Paint slim modern scrollbar thumb (only if scrollable and viewport has enough height)
    if guard.max_scroll > 0 && viewport_h >= 40 {
        let track_top = 4;
        let track_bottom = viewport_h - 4;
        let track_h = (track_bottom - track_top).max(1);
        let total_h = guard.content_h.max(viewport_h);
        let min_thumb = 28.0f32.min(track_h as f32);
        let thumb_h = ((viewport_h as f32 / total_h as f32) * track_h as f32)
            .clamp(min_thumb, track_h as f32) as i32;
        let travel = (track_h - thumb_h).max(1);
        let thumb_top =
            track_top + ((scroll_y as f32 / guard.max_scroll as f32) * travel as f32) as i32;
        let thumb_color = if guard.is_dragging_scroll {
            palette.accent
        } else {
            palette.border_separator
        };
        let thumb_rc = RECT {
            left: guard.content_w + 3,
            top: thumb_top,
            right: guard.content_w + 7,
            bottom: thumb_top + thumb_h,
        };
        round_rect(hdc, &thumb_rc, 4, thumb_color, None);
    }

    // 4. Paint top-layer open Select dropdown overlay (z-index: top)
    if let Some(ref sel) = guard.open_select {
        let menu_rc = compute_dropdown_menu_rect(sel, scroll_y, viewport_h);
        round_rect(
            hdc,
            &menu_rc,
            radii::RADIUS_MD,
            palette.bg_card,
            Some(palette.accent),
        );

        unsafe {
            SetBkMode(hdc, TRANSPARENT);
            let font_norm = resolve_font_handle(FontScale::Normal);
            let font_bold = resolve_font_handle(FontScale::Bold);

            for (i, opt) in sel.options.iter().enumerate() {
                let item_top = menu_rc.top + 2 + i as i32 * SELECT_ITEM_H;
                if item_top + SELECT_ITEM_H > menu_rc.bottom {
                    break;
                }
                let item_rc = RECT {
                    left: menu_rc.left + 2,
                    top: item_top,
                    right: menu_rc.right - 2,
                    bottom: item_top + SELECT_ITEM_H,
                };
                let is_sel = i == sel.selected;
                let is_hov = sel.hover_idx == Some(i);

                if is_hov {
                    round_rect(hdc, &item_rc, 4, palette.accent, None);
                } else if is_sel {
                    round_rect(hdc, &item_rc, 4, palette.tab_active_bg, None);
                }

                let txt_color = if is_hov {
                    palette.text_white
                } else if is_sel {
                    palette.accent
                } else {
                    palette.text_primary
                };
                SetTextColor(hdc, txt_color);
                SelectObject(hdc, if is_sel { font_bold } else { font_norm });

                let mut text_rc = RECT {
                    left: item_rc.left + 10,
                    top: item_rc.top,
                    right: item_rc.right - 24,
                    bottom: item_rc.bottom,
                };
                let wide = to_wide(opt);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut text_rc,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                );

                if is_sel {
                    let mut chk_rc = RECT {
                        left: item_rc.right - 22,
                        top: item_rc.top,
                        right: item_rc.right - 6,
                        bottom: item_rc.bottom,
                    };
                    let chk_w = to_wide("✓");
                    DrawTextW(
                        hdc,
                        chk_w.as_ptr(),
                        -1,
                        &mut chk_rc,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                }
            }
        }
    }
}

pub(crate) fn paint_node<M>(
    hdc: isize,
    node: &LayoutNode<M>,
    scroll_y: i32,
    viewport_h: i32,
    hover_id: Option<usize>,
    open_select_key: Option<u32>,
    focused_node_id: Option<usize>,
    palette: &ThemePalette,
) {
    let rc = offset_rect(&node.rect, -scroll_y);
    if rc.bottom < 0 || rc.top > viewport_h {
        return;
    }

    let style = &node.element.style;
    let is_hover = hover_id == Some(node.id);

    // 1. Paint Container Box Background & Borders
    let is_clickable_container = node.element.on_click.is_some();
    let bg_opt = if is_hover && is_clickable_container && style.bg != BgStyle::TabActive {
        Some(palette.tab_hover_bg)
    } else {
        style.bg.resolve(palette)
    };
    let border_opt = if is_hover && is_clickable_container && style.bg != BgStyle::TabActive {
        Some((palette.border, false))
    } else {
        style.border.resolve(palette)
    };
    let radius = if style.radius > 0 {
        style.radius
    } else if is_clickable_container {
        6
    } else {
        0
    };

    if bg_opt.is_some() || border_opt.is_some() {
        let fill = bg_opt.unwrap_or(palette.bg_card);
        if let Some((b_col, is_dashed)) = border_opt {
            if is_dashed {
                unsafe {
                    round_rect(hdc, &rc, radius, fill, None);
                    let pen = CreatePen(PS_DASH, 1, b_col);
                    let op = SelectObject(hdc, pen);
                    let ob = SelectObject(hdc, GetStockObject(NULL_BRUSH));
                    RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
                    SelectObject(hdc, ob);
                    SelectObject(hdc, op);
                    DeleteObject(pen);
                }
            } else {
                round_rect(hdc, &rc, radius, fill, Some(b_col));
            }
        } else {
            round_rect(hdc, &rc, radius, fill, None);
        }
    }

    // Left accent bar (e.g. Alert Banner)
    if let Some((bar_w, variant)) = style.border_left_accent {
        let bar_col = variant.colors(palette, false).2;
        let bar_rc = RECT {
            left: rc.left,
            top: rc.top + 2,
            right: (rc.left + bar_w).min(rc.right),
            bottom: rc.bottom - 2,
        };
        round_rect(hdc, &bar_rc, 4, bar_col, None);
    }

    if style.border_bottom {
        draw_line(
            hdc,
            rc.left,
            rc.bottom - 1,
            rc.right,
            rc.bottom - 1,
            palette.border_separator,
        );
    }
    if style.border_top {
        draw_line(
            hdc,
            rc.left,
            rc.top,
            rc.right,
            rc.top,
            palette.border_separator,
        );
    }

    // 2. Paint Atomic Widget Content
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        match &node.element.kind {
            WidgetKind::Empty => {}
            WidgetKind::Divider => {
                fill_rect_color(hdc, &rc, palette.border_separator);
            }
            WidgetKind::Text {
                content,
                font,
                tone,
                align,
                wrap,
                on_click,
            } => {
                let hf = resolve_font_handle(*font);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                let color = if on_click.is_some() && is_hover {
                    palette.accent_hover
                } else {
                    tone.resolve(palette)
                };
                SetTextColor(hdc, color);
                let mut text_rc = rc;
                let align_flag = match align {
                    TextAlign::Left => DT_LEFT,
                    TextAlign::Center => DT_CENTER,
                    TextAlign::Right => DT_RIGHT,
                };
                let wrap_flag = if *wrap {
                    DT_WORDBREAK
                } else {
                    DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS
                };
                let wide = to_wide(content);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut text_rc,
                    align_flag | wrap_flag | DT_NOPREFIX,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Label {
                content,
                font,
                tone,
                align,
            } => {
                let hf = resolve_font_handle(*font);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                let is_clickable = node.element.on_click.is_some();
                let color = if is_clickable && is_hover {
                    palette.accent_hover
                } else if is_clickable {
                    palette.accent
                } else {
                    tone.resolve(palette)
                };
                SetTextColor(hdc, color);
                let mut text_rc = rc;
                let align_flag = match align {
                    TextAlign::Left => DT_LEFT,
                    TextAlign::Center => DT_CENTER,
                    TextAlign::Right => DT_RIGHT,
                };
                let wide = to_wide(content);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut text_rc,
                    align_flag | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Badge {
                text,
                variant,
                is_pill,
                disabled,
                ..
            } => {
                let (bg, border, fg) = match variant {
                    Variant::Primary => (
                        palette.bg_input,
                        if is_hover { palette.accent } else { palette.border },
                        if *disabled {
                            palette.text_secondary
                        } else {
                            palette.accent
                        },
                    ),
                    v => v.colors(palette, is_hover),
                };
                let rad = if *is_pill {
                    radii::RADIUS_PILL
                } else {
                    radii::RADIUS_MD
                };
                round_rect(hdc, &rc, rad, bg, Some(border));
                let hf = resolve_font_handle(FontScale::Bold);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                SetTextColor(hdc, if *disabled { palette.text_secondary } else { fg });
                let mut trc = rc;
                let wide = to_wide(text);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut trc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Button {
                label,
                variant,
                disabled,
                ..
            } => {
                let (bg, border, fg) = if *disabled {
                    (palette.bg_input, palette.border_separator, palette.text_secondary)
                } else {
                    variant.colors(palette, is_hover)
                };
                round_rect(hdc, &rc, radii::RADIUS_MD, bg, Some(border));
                let hf = resolve_font_handle(FontScale::Bold);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                SetTextColor(hdc, fg);
                let mut trc = rc;
                let wide = to_wide(label);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut trc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Switch {
                checked,
                label,
                note,
                disabled,
                ..
            } => {
                let sw_w = 34;
                let sw_h = 18;
                let sw_top = rc.top + ((rc.bottom - rc.top) - sw_h) / 2;
                let sw_rc = RECT {
                    left: rc.left,
                    top: sw_top,
                    right: rc.left + sw_w,
                    bottom: sw_top + sw_h,
                };
                let (track_bg, track_border, thumb_col) = if *disabled {
                    (palette.bg_input, palette.border_separator, palette.text_secondary)
                } else if *checked {
                    let c = if is_hover {
                        palette.accent_hover
                    } else {
                        palette.accent
                    };
                    (c, c, palette.text_white)
                } else {
                    (
                        palette.bg_input,
                        if is_hover {
                            palette.accent
                        } else {
                            palette.border_hover
                        },
                        palette.text_secondary,
                    )
                };
                round_rect(hdc, &sw_rc, sw_h, track_bg, Some(track_border));

                let thumb_d = 12;
                let thumb_left = if *checked {
                    sw_rc.right - thumb_d - 3
                } else {
                    sw_rc.left + 3
                };
                let thumb_rc = RECT {
                    left: thumb_left,
                    top: sw_rc.top + 3,
                    right: thumb_left + thumb_d,
                    bottom: sw_rc.top + 3 + thumb_d,
                };
                fill_ellipse(hdc, &thumb_rc, thumb_col);

                let mut cur_x = sw_rc.right + 8;
                if !label.is_empty() {
                    let hf = resolve_font_handle(FontScale::Bold);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(
                        hdc,
                        if *disabled {
                            palette.text_secondary
                        } else {
                            palette.text_primary
                        },
                    );
                    let (lw, _) = measure_text_rect(hdc, label, FontScale::Bold, 1000, false);
                    let mut lrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: (cur_x + lw + 2).min(rc.right),
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(label);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut lrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                    cur_x = lrc.right + 6;
                }
                if !note.is_empty() && cur_x < rc.right {
                    let hf = resolve_font_handle(FontScale::Small);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(hdc, palette.text_secondary);
                    let mut nrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: rc.right,
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(note);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut nrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                }
            }
            WidgetKind::Checkbox {
                checked,
                indeterminate,
                label,
                note,
                disabled,
                ..
            } => {
                let box_sz = 15;
                let box_top = rc.top + ((rc.bottom - rc.top) - box_sz) / 2;
                let box_rc = RECT {
                    left: rc.left,
                    top: box_top,
                    right: rc.left + box_sz,
                    bottom: box_top + box_sz,
                };
                let is_active = *checked || *indeterminate;
                let (bg, border) = if *disabled {
                    (palette.bg_input, palette.border_separator)
                } else if is_active {
                    let c = if is_hover {
                        palette.accent_hover
                    } else {
                        palette.accent
                    };
                    (c, c)
                } else {
                    (
                        palette.bg_input,
                        if is_hover {
                            palette.accent
                        } else {
                            palette.border_hover
                        },
                    )
                };
                round_rect(hdc, &box_rc, 3, bg, Some(border));

                if *indeterminate {
                    let dash_rc = RECT {
                        left: box_rc.left + 3,
                        top: box_rc.top + 6,
                        right: box_rc.right - 3,
                        bottom: box_rc.top + 9,
                    };
                    fill_rect_color(hdc, &dash_rc, palette.text_white);
                } else if *checked {
                    let pen = CreatePen(PS_SOLID, 2, palette.text_white);
                    let op = SelectObject(hdc, pen);
                    MoveToEx(hdc, box_rc.left + 3, box_rc.top + 7, null_mut());
                    LineTo(hdc, box_rc.left + 6, box_rc.top + 10);
                    LineTo(hdc, box_rc.left + 12, box_rc.top + 4);
                    SelectObject(hdc, op);
                    DeleteObject(pen);
                }

                let mut cur_x = box_rc.right + 8;
                if !label.is_empty() {
                    let hf = resolve_font_handle(FontScale::Normal);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(
                        hdc,
                        if *disabled {
                            palette.text_secondary
                        } else {
                            palette.text_primary
                        },
                    );
                    let (lw, _) = measure_text_rect(hdc, label, FontScale::Normal, 1000, false);
                    let mut lrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: (cur_x + lw + 2).min(rc.right),
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(label);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut lrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                    cur_x = lrc.right + 6;
                }
                if !note.is_empty() && cur_x < rc.right {
                    let hf = resolve_font_handle(FontScale::Small);
                    let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                    SetTextColor(hdc, palette.text_secondary);
                    let mut nrc = RECT {
                        left: cur_x,
                        top: rc.top,
                        right: rc.right,
                        bottom: rc.bottom,
                    };
                    let wide = to_wide(note);
                    DrawTextW(
                        hdc,
                        wide.as_ptr(),
                        -1,
                        &mut nrc,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                    );
                    if old_f != 0 {
                        SelectObject(hdc, old_f);
                    }
                }
            }
            WidgetKind::Select {
                key,
                selected,
                options,
                disabled,
                ..
            } => {
                let is_open = open_select_key == Some(*key);
                let border_col = if *disabled {
                    palette.border_separator
                } else if is_open || is_hover {
                    palette.accent
                } else {
                    palette.border
                };
                round_rect(hdc, &rc, radii::RADIUS_MD, palette.bg_input, Some(border_col));

                let sel_text = options.get(*selected).map(|s| s.as_str()).unwrap_or("");
                let hf = resolve_font_handle(FontScale::Normal);
                let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                SetTextColor(
                    hdc,
                    if *disabled {
                        palette.text_secondary
                    } else {
                        palette.text_primary
                    },
                );
                let mut trc = RECT {
                    left: rc.left + spacing::SPACE_8,
                    top: rc.top,
                    right: rc.right - 24,
                    bottom: rc.bottom,
                };
                let wide = to_wide(sel_text);
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    -1,
                    &mut trc,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                );

                // Draw chevron arrow `▾`
                SetTextColor(hdc, palette.text_secondary);
                let mut crc = RECT {
                    left: rc.right - 22,
                    top: rc.top,
                    right: rc.right - 6,
                    bottom: rc.bottom,
                };
                let chev = to_wide(if is_open { "▴" } else { "▾" });
                DrawTextW(
                    hdc,
                    chev.as_ptr(),
                    -1,
                    &mut crc,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
                if old_f != 0 {
                    SelectObject(hdc, old_f);
                }
            }
            WidgetKind::Slider {
                value,
                min,
                max,
                disabled,
                ..
            } => {
                let pad = 8;
                let track_l = rc.left + pad;
                let track_r = (rc.right - pad).max(track_l + 4);
                let cy = rc.top + (rc.bottom - rc.top) / 2;
                let track_rc = RECT {
                    left: track_l,
                    top: cy - 3,
                    right: track_r,
                    bottom: cy + 3,
                };
                round_rect(hdc, &track_rc, 6, palette.bg_input, Some(palette.border));

                let ratio = ((*value - *min) as f32 / (*max - *min).max(1) as f32).clamp(0.0, 1.0);
                let thumb_x = track_l + (ratio * (track_r - track_l) as f32).round() as i32;
                let active_col = if *disabled {
                    palette.border_hover
                } else {
                    palette.accent
                };

                if thumb_x > track_l {
                    let fill_rc = RECT {
                        left: track_l,
                        top: cy - 3,
                        right: thumb_x,
                        bottom: cy + 3,
                    };
                    round_rect(hdc, &fill_rc, 6, active_col, None);
                }

                let r = if is_hover && !*disabled { 8 } else { 7 };
                let thumb_rc = RECT {
                    left: thumb_x - r,
                    top: cy - r,
                    right: thumb_x + r,
                    bottom: cy + r,
                };
                fill_ellipse(hdc, &thumb_rc, active_col);
            }
            WidgetKind::Row(_) | WidgetKind::Col(_) | WidgetKind::Grid { .. } => {}
            WidgetKind::TabGroup { options, selected, .. } => {
                round_rect(
                    hdc,
                    &rc,
                    radii::RADIUS_PILL,
                    palette.tab_track_bg,
                    Some(palette.tab_track_border),
                );

                let num_tabs = options.len();
                if num_tabs > 0 {
                    let track_padding = 3;
                    let available_w = (rc.right - rc.left) - (track_padding * 2);
                    let tab_w = available_w / num_tabs as i32;

                    for (i, title) in options.iter().enumerate() {
                        let i_i32 = i as i32;
                        let item_left = rc.left + track_padding + i_i32 * tab_w;
                        let item_right = if i == num_tabs - 1 {
                            rc.right - track_padding
                        } else {
                            item_left + tab_w
                        };
                        let item_top = rc.top + track_padding;
                        let item_bottom = rc.bottom - track_padding;

                        let mut item_rc = RECT {
                            left: item_left,
                            top: item_top,
                            right: item_right,
                            bottom: item_bottom,
                        };

                        let is_item_hover = if let Some(hid) = hover_id {
                            node.children.get(i).map(|c| c.id == hid).unwrap_or(false)
                        } else {
                            false
                        };

                        if i == *selected {
                            round_rect(
                                hdc,
                                &item_rc,
                                radii::RADIUS_CARD,
                                palette.tab_active_bg,
                                Some(palette.tab_active_border),
                            );

                            let hf = resolve_font_handle(FontScale::Bold);
                            let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                            SetTextColor(hdc, palette.accent);

                            let line_rc = RECT {
                                left: item_rc.left + 4,
                                top: item_rc.bottom - 2,
                                right: item_rc.right - 4,
                                bottom: item_rc.bottom,
                            };
                            let line_br = CreateSolidBrush(palette.accent);
                            FillRect(hdc, &line_rc, line_br);
                            DeleteObject(line_br);

                            let wide = to_wide(title);
                            DrawTextW(
                                hdc,
                                wide.as_ptr(),
                                -1,
                                &mut item_rc,
                                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                            );
                            if old_f != 0 {
                                SelectObject(hdc, old_f);
                            }
                        } else {
                            if is_item_hover {
                                round_rect(
                                    hdc,
                                    &item_rc,
                                    radii::RADIUS_CARD,
                                    palette.tab_hover_bg,
                                    None,
                                );
                            }
                            let hf = resolve_font_handle(FontScale::Normal);
                            let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
                            SetTextColor(hdc, palette.text_secondary);

                            let wide = to_wide(title);
                            DrawTextW(
                                hdc,
                                wide.as_ptr(),
                                -1,
                                &mut item_rc,
                                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                            );
                            if old_f != 0 {
                                SelectObject(hdc, old_f);
                            }
                        }
                    }
                }
            }
            WidgetKind::Input { disabled, .. } => {
                let is_focused = focused_node_id == Some(node.id);
                let (border_color, border_w) = if *disabled {
                    (palette.border, 1)
                } else if is_focused {
                    (palette.accent, 2)
                } else if is_hover {
                    (palette.border_hover, 1)
                } else {
                    (palette.border, 1)
                };
                let bg_color = if *disabled {
                    palette.bg_window
                } else {
                    palette.bg_input
                };

                let brush = CreateSolidBrush(bg_color);
                let pen = CreatePen(PS_SOLID, border_w, border_color);
                let old_b = SelectObject(hdc, brush);
                let old_p = SelectObject(hdc, pen);

                RoundRect(
                    hdc,
                    rc.left,
                    rc.top,
                    rc.right,
                    rc.bottom,
                    radii::RADIUS_MD,
                    radii::RADIUS_MD,
                );

                SelectObject(hdc, old_b);
                SelectObject(hdc, old_p);
                DeleteObject(brush);
                DeleteObject(pen);
            }
        }
    }

    // 3. Paint Child Nodes
    if !matches!(node.element.kind, WidgetKind::TabGroup { .. } | WidgetKind::Input { .. }) {
        for child in &node.children {
            paint_node(
                hdc,
                child,
                scroll_y,
                viewport_h,
                hover_id,
                open_select_key,
                focused_node_id,
                palette,
            );
        }
    }
}
