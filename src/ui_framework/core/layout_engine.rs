//! Two-Pass Flexbox & Box-Model Layout Engine (Measure + Arrange)
//!
//! Computes exact `(x, y, width, height)` rectangles for an arbitrary [`Element<M>`] tree,
//! including automatic multiline word-break text height measurement via `DrawTextW(DT_CALCRECT)`.

use super::element::*;
use crate::ui_framework::theme::{cached_font, get_app_font_bold, get_app_font_normal};
use crate::ui_framework::tokens::typography;
use crate::ui_framework::win32::*;

/// A positioned node in the resolved layout tree ready for rendering and hit-testing.
pub struct LayoutNode<M> {
    pub id: usize,
    pub rect: RECT,
    pub element: Element<M>,
    pub children: Vec<LayoutNode<M>>,
}

#[inline]
pub fn resolve_font_handle(scale: FontScale) -> isize {
    match scale {
        FontScale::Normal => get_app_font_normal(),
        FontScale::Bold => get_app_font_bold(),
        FontScale::Title => cached_font(typography::FONT_SIZE_TITLE, FW_BOLD),
        FontScale::Hero => cached_font(typography::FONT_SIZE_HERO, FW_BOLD),
        FontScale::Small => cached_font(typography::FONT_SIZE_SUBTITLE, FW_NORMAL),
    }
}

/// Measures single-line or multiline text dimensions `(width, height)` in pixels.
pub fn measure_text_rect(
    hdc: isize,
    text: &str,
    font: FontScale,
    max_width: i32,
    wrap: bool,
) -> (i32, i32) {
    if text.is_empty() {
        return (0, 0);
    }
    let avail_w = max_width.max(1);
    if hdc != 0 {
        unsafe {
            let hf = resolve_font_handle(font);
            let old_f = if hf != 0 { SelectObject(hdc, hf) } else { 0 };
            let wide = to_wide(text);
            let mut rc = RECT {
                left: 0,
                top: 0,
                right: avail_w,
                bottom: 0,
            };
            let flags = DT_CALCRECT | DT_NOPREFIX | if wrap { DT_WORDBREAK } else { DT_SINGLELINE };
            DrawTextW(hdc, wide.as_ptr(), -1, &mut rc, flags);
            if old_f != 0 {
                SelectObject(hdc, old_f);
            }
            let w = (rc.right - rc.left).max(1);
            let h = (rc.bottom - rc.top).max(16);
            return (w, h);
        }
    }

    // Deterministic fallback for headless unit tests (when hdc == 0)
    let char_count = text.chars().count() as i32;
    let avg_char_w = match font {
        FontScale::Hero => 11,
        FontScale::Title => 8,
        FontScale::Bold => 7,
        FontScale::Normal | FontScale::Small => 7,
    };
    let line_h = match font {
        FontScale::Hero => 26,
        FontScale::Title => 20,
        _ => 16,
    };
    let single_w = char_count * avg_char_w;
    if !wrap || single_w <= avail_w {
        (single_w, line_h)
    } else {
        let lines = ((single_w + avail_w - 1) / avail_w).max(1);
        (avail_w, lines * line_h)
    }
}

/// Resolves an [`Element<M>`] tree into a positioned [`LayoutNode<M>`] tree within `avail_w`.
pub fn compute_layout<M>(
    hdc: isize,
    root: impl IntoElement<M>,
    x: i32,
    y: i32,
    avail_w: i32,
) -> LayoutNode<M> {
    let mut next_id = 1usize;
    layout_node(hdc, root.into_element(), x, y, avail_w, &mut next_id)
}

/// Pure size measurement pass (does not consume `Element<M>`).
fn intrinsic_size<M>(hdc: isize, el: &Element<M>, avail_w: i32) -> (i32, i32) {
    let pad_x = el.style.padding.horizontal();
    let pad_y = el.style.padding.vertical();

    let target_w = match el.style.width {
        Length::Fixed(w) => w,
        Length::Full | Length::Flex(_) => avail_w,
        Length::Auto => avail_w,
    };
    let inner_avail_w = (target_w - pad_x).max(1);

    let (content_w, content_h) = match &el.kind {
        WidgetKind::Empty => (0, 0),
        WidgetKind::Divider => (inner_avail_w, 1),
        WidgetKind::Text {
            content,
            font,
            wrap,
            ..
        } => measure_text_rect(hdc, content, *font, inner_avail_w, *wrap),
        WidgetKind::Label { content, font, .. } => {
            measure_text_rect(hdc, content, *font, 2000, false)
        }
        WidgetKind::Badge { text, .. } => {
            let (tw, _) = measure_text_rect(hdc, text, FontScale::Bold, 2000, false);
            let w = (tw + 20).max(el.style.min_w).max(32);
            let h = if el.style.min_w >= 80 { 28 } else { 24 };
            (w, h)
        }
        WidgetKind::Button { label, size, .. } => {
            let (tw, _) = measure_text_rect(hdc, label, FontScale::Bold, 2000, false);
            let h = match size {
                Size::Sm => 26,
                Size::Md => 28,
                Size::Lg => 32,
            };
            let w = (tw + 24).max(el.style.min_w).max(28);
            (w, h)
        }
        WidgetKind::Switch { label, note, .. } => {
            let (lw, _) = measure_text_rect(hdc, label, FontScale::Bold, 2000, false);
            let (nw, _) = measure_text_rect(hdc, note, FontScale::Small, 2000, false);
            let w = 36 + if lw > 0 { 8 + lw } else { 0 } + if nw > 0 { 6 + nw } else { 0 };
            (w, 22)
        }
        WidgetKind::Checkbox { label, note, .. } => {
            let (lw, _) = measure_text_rect(hdc, label, FontScale::Normal, 2000, false);
            let (nw, _) = measure_text_rect(hdc, note, FontScale::Small, 2000, false);
            let w = 16 + if lw > 0 { 8 + lw } else { 0 } + if nw > 0 { 6 + nw } else { 0 };
            (w, 22)
        }
        WidgetKind::Select { options, .. } => {
            let mut max_tw = 80;
            for opt in options {
                let (tw, _) = measure_text_rect(hdc, opt, FontScale::Normal, 2000, false);
                if tw > max_tw {
                    max_tw = tw;
                }
            }
            let w = match el.style.width {
                Length::Auto => max_tw + 36,
                _ => inner_avail_w,
            };
            (w, 28)
        }
        WidgetKind::Slider { .. } => (inner_avail_w, 24),
        WidgetKind::TabGroup { .. } => (inner_avail_w, 32),
        WidgetKind::Input { .. } => {
            let h = match el.style.height {
                Length::Fixed(fixed_h) => fixed_h,
                _ => 32,
            };
            let w = match el.style.width {
                Length::Fixed(fixed_w) => fixed_w,
                _ => inner_avail_w.max(el.style.min_w).max(80),
            };
            (w, h)
        }
        WidgetKind::Col(children) => {
            let mut total_h = 0;
            let mut max_cw = 0;
            let mut visible_count = 0;
            for c in children {
                if c.is_empty() {
                    continue;
                }
                let (cw, ch) = intrinsic_size(hdc, c, inner_avail_w);
                total_h += ch;
                if cw > max_cw {
                    max_cw = cw;
                }
                visible_count += 1;
            }
            if visible_count > 1 {
                total_h += el.style.gap * (visible_count - 1);
            }
            let w = match el.style.width {
                Length::Auto => max_cw,
                _ => inner_avail_w,
            };
            (w, total_h)
        }
        WidgetKind::Row(children) => {
            let (_, row_h, used_w) =
                measure_row_children(hdc, children, inner_avail_w, el.style.gap);
            let w = match el.style.width {
                Length::Auto => used_w,
                _ => inner_avail_w,
            };
            (w, row_h)
        }
        WidgetKind::Grid { cols, children } => {
            let cols = (*cols).max(1);
            let gap = el.style.gap;
            let col_w = ((inner_avail_w - gap * (cols as i32 - 1)) / cols as i32).max(10);
            let mut total_h = 0;
            let mut row_max_h = 0;
            let mut col_idx = 0;
            for c in children {
                if c.is_empty() {
                    continue;
                }
                let (_, ch) = intrinsic_size(hdc, c, col_w);
                if ch > row_max_h {
                    row_max_h = ch;
                }
                col_idx += 1;
                if col_idx == cols {
                    total_h += row_max_h + gap;
                    row_max_h = 0;
                    col_idx = 0;
                }
            }
            if col_idx > 0 {
                total_h += row_max_h;
            } else if total_h >= gap {
                total_h -= gap;
            }
            (inner_avail_w, total_h)
        }
    };

    let final_w = match el.style.width {
        Length::Fixed(w) => w,
        Length::Full | Length::Flex(_) => avail_w,
        Length::Auto => (content_w + pad_x).max(el.style.min_w),
    };
    let final_h = match el.style.height {
        Length::Fixed(h) => h,
        _ => (content_h + pad_y).max(el.style.min_h),
    };

    (final_w, final_h)
}

/// Computes child `(width, height)` allocations for a vertical `Col`.
fn measure_col_children<M>(
    hdc: isize,
    children: &[Element<M>],
    inner_w: i32,
    inner_h: i32,
    gap: i32,
) -> Vec<(i32, i32)> {
    let n = children.len();
    if n == 0 {
        return Vec::new();
    }

    let mut sizes = vec![(0, 0); n];
    let mut fixed_sum = 0;
    let mut flex_weight_sum = 0u32;
    let mut active_count = 0i32;

    // 1. Measure non-flex items first
    for (i, c) in children.iter().enumerate() {
        if c.is_empty() {
            continue;
        }
        active_count += 1;
        match c.style.height {
            Length::Flex(weight) => {
                flex_weight_sum += weight.max(1) as u32;
            }
            Length::Full => {
                flex_weight_sum += 1;
            }
            Length::Fixed(h) => {
                let (w, _) = intrinsic_size(hdc, c, inner_w);
                sizes[i] = (w, h);
                fixed_sum += h;
            }
            Length::Auto => {
                let (w, h) = intrinsic_size(hdc, c, inner_w);
                sizes[i] = (w, h);
                fixed_sum += h;
            }
        }
    }

    let total_gaps = if active_count > 1 {
        gap * (active_count - 1)
    } else {
        0
    };
    let remaining_h = (inner_h - fixed_sum - total_gaps).max(0);

    // 2. Measure flex items using their allocated share of `remaining_h`
    let mut rem_left = remaining_h;
    let mut weight_left = flex_weight_sum;
    for (i, c) in children.iter().enumerate() {
        if c.is_empty() {
            continue;
        }
        let h_weight = match c.style.height {
            Length::Flex(h) => Some(h.max(1) as u32),
            Length::Full => Some(1),
            _ => None,
        };
        if let Some(h_val) = h_weight {
            let alloc_h = if weight_left == h_val {
                rem_left
            } else {
                ((remaining_h as i64 * h_val as i64) / flex_weight_sum.max(1) as i64) as i32
            };
            rem_left = (rem_left - alloc_h).max(0);
            weight_left = weight_left.saturating_sub(h_val);
            let (w, _) = intrinsic_size(hdc, c, inner_w);
            sizes[i] = (w, alloc_h.max(0));
        }
    }

    sizes
}

/// Computes child `(width, height)` allocations for a horizontal `Row`.
fn measure_row_children<M>(
    hdc: isize,
    children: &[Element<M>],
    inner_w: i32,
    gap: i32,
) -> (Vec<(i32, i32)>, i32, i32) {
    let n = children.len();
    if n == 0 {
        return (Vec::new(), 0, 0);
    }

    let mut sizes = vec![(0, 0); n];
    let mut fixed_sum = 0;
    let mut flex_weight_sum = 0u32;
    let mut active_count = 0i32;

    // 1. Measure non-flex items first
    for (i, c) in children.iter().enumerate() {
        if c.is_empty() {
            continue;
        }
        active_count += 1;
        match c.style.width {
            Length::Flex(weight) => {
                flex_weight_sum += weight.max(1) as u32;
            }
            Length::Full => {
                flex_weight_sum += 1;
            }
            Length::Fixed(w) => {
                let (_, h) = intrinsic_size(hdc, c, w);
                sizes[i] = (w, h);
                fixed_sum += w;
            }
            Length::Auto => {
                let (w, h) = intrinsic_size(hdc, c, inner_w);
                sizes[i] = (w, h);
                fixed_sum += w;
            }
        }
    }

    let total_gaps = if active_count > 1 {
        gap * (active_count - 1)
    } else {
        0
    };
    let remaining_w = (inner_w - fixed_sum - total_gaps).max(0);

    // 2. Measure flex items using their allocated share of `remaining_w`
    let mut rem_left = remaining_w;
    let mut weight_left = flex_weight_sum;
    for (i, c) in children.iter().enumerate() {
        if c.is_empty() {
            continue;
        }
        let w_weight = match c.style.width {
            Length::Flex(w) => Some(w.max(1) as u32),
            Length::Full => Some(1),
            _ => None,
        };
        if let Some(w_val) = w_weight {
            let alloc_w = if weight_left == w_val {
                rem_left
            } else {
                ((remaining_w as i64 * w_val as i64) / flex_weight_sum.max(1) as i64) as i32
            };
            rem_left = (rem_left - alloc_w).max(0);
            weight_left = weight_left.saturating_sub(w_val);
            let (_, h) = intrinsic_size(hdc, c, alloc_w.max(1));
            sizes[i] = (alloc_w.max(0), h);
        }
    }

    let max_h = sizes.iter().map(|&(_, h)| h).max().unwrap_or(0);
    let used_w: i32 = sizes.iter().map(|&(w, _)| w).sum::<i32>() + total_gaps;
    (sizes, max_h, used_w)
}

fn layout_node<M>(
    hdc: isize,
    mut el: Element<M>,
    x: i32,
    y: i32,
    avail_w: i32,
    next_id: &mut usize,
) -> LayoutNode<M> {
    let id = *next_id;
    *next_id += 1;

    let (w, h) = intrinsic_size(hdc, &el, avail_w);
    let rect = RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    };

    let inner_left = x + el.style.padding.left;
    let inner_top = y + el.style.padding.top;
    let inner_w = (w - el.style.padding.horizontal()).max(0);
    let inner_h = (h - el.style.padding.vertical()).max(0);

    let mut layout_children = Vec::new();

    match &mut el.kind {
        WidgetKind::Col(children) => {
            let raw_children = std::mem::take(children);
            let sizes = measure_col_children(hdc, &raw_children, inner_w, inner_h, el.style.gap);
            let active_count = raw_children.iter().filter(|c| !c.is_empty()).count();
            let total_used_h: i32 = sizes.iter().map(|&(_, h)| h).sum::<i32>()
                + if active_count > 1 {
                    el.style.gap * (active_count as i32 - 1)
                } else {
                    0
                };
            let extra_space = (inner_h - total_used_h).max(0);
            let mut cur_y = match el.style.justify {
                Justify::Start | Justify::Between => inner_top,
                Justify::Center => inner_top + extra_space / 2,
                Justify::End => inner_top + extra_space,
            };
            let between_extra = if el.style.justify == Justify::Between && active_count > 1 {
                extra_space / (active_count as i32 - 1)
            } else {
                0
            };
            for (mut c, (cw, ch)) in raw_children.into_iter().zip(sizes.into_iter()) {
                if c.is_empty() {
                    continue;
                }
                if matches!(c.style.height, Length::Flex(_) | Length::Full) {
                    c.style.height = Length::Fixed(ch);
                }
                let child_w = match c.style.width {
                    Length::Fixed(w) => w,
                    Length::Full | Length::Flex(_) => inner_w,
                    Length::Auto => {
                        if el.style.align == Align::Stretch {
                            inner_w
                        } else {
                            cw
                        }
                    }
                };
                let child_x = match el.style.align {
                    Align::Start | Align::Stretch => inner_left,
                    Align::Center => inner_left + (inner_w - child_w) / 2,
                    Align::End => inner_left + (inner_w - child_w),
                };
                let mut child_el = c;
                if el.style.align == Align::Stretch {
                    child_el.style.width = Length::Fixed(inner_w);
                }
                let node = layout_node(hdc, child_el, child_x, cur_y, child_w, next_id);
                cur_y = node.rect.bottom + el.style.gap + between_extra;
                layout_children.push(node);
            }
        }
        WidgetKind::Row(children) => {
            let raw_children = std::mem::take(children);
            let (sizes, row_h, used_w) =
                measure_row_children(hdc, &raw_children, inner_w, el.style.gap);
            let row_box_h = inner_h.max(row_h);

            let active_count = raw_children.iter().filter(|c| !c.is_empty()).count();
            let extra_space = (inner_w - used_w).max(0);

            let mut cur_x = match el.style.justify {
                Justify::Start | Justify::Between => inner_left,
                Justify::Center => inner_left + extra_space / 2,
                Justify::End => inner_left + extra_space,
            };
            let between_extra = if el.style.justify == Justify::Between && active_count > 1 {
                extra_space / (active_count as i32 - 1)
            } else {
                0
            };

            for (c, (cw, ch)) in raw_children.into_iter().zip(sizes.into_iter()) {
                if c.is_empty() {
                    continue;
                }
                let child_y = match el.style.align {
                    Align::Start | Align::Stretch => inner_top,
                    Align::Center => inner_top + (row_box_h - ch) / 2,
                    Align::End => inner_top + (row_box_h - ch),
                };
                let mut child_el = c;
                if el.style.align == Align::Stretch {
                    child_el.style.min_h = child_el.style.min_h.max(row_box_h);
                }
                let node = layout_node(hdc, child_el, cur_x, child_y, cw, next_id);
                cur_x += cw + el.style.gap + between_extra;
                layout_children.push(node);
            }
        }
        WidgetKind::Grid { cols, children } => {
            let cols = (*cols).max(1);
            let gap = el.style.gap;
            let col_w = ((inner_w - gap * (cols as i32 - 1)) / cols as i32).max(10);
            let raw_children = std::mem::take(children);

            let mut cur_y = inner_top;
            let mut row_items: Vec<Element<M>> = Vec::with_capacity(cols);

            for c in raw_children {
                if c.is_empty() {
                    continue;
                }
                row_items.push(c);
                if row_items.len() == cols {
                    let mut row_max_h = 0;
                    for (col_idx, item) in row_items.drain(..).enumerate() {
                        let cx = inner_left + col_idx as i32 * (col_w + gap);
                        let node = layout_node(hdc, item, cx, cur_y, col_w, next_id);
                        let nh = node.rect.bottom - node.rect.top;
                        if nh > row_max_h {
                            row_max_h = nh;
                        }
                        layout_children.push(node);
                    }
                    cur_y += row_max_h + gap;
                }
            }
            if !row_items.is_empty() {
                for (col_idx, item) in row_items.drain(..).enumerate() {
                    let cx = inner_left + col_idx as i32 * (col_w + gap);
                    let node = layout_node(hdc, item, cx, cur_y, col_w, next_id);
                    layout_children.push(node);
                }
            }
        }
        WidgetKind::TabGroup {
            options, on_select, ..
        } => {
            let track_padding = 3;
            let available_w = (inner_w - track_padding * 2).max(1);
            let num_tabs = options.len();
            if num_tabs > 0 {
                let tab_w = available_w / num_tabs as i32;
                for (i, _) in options.iter().enumerate() {
                    let item_left = inner_left + track_padding + (i as i32) * tab_w;
                    let item_right = if i == num_tabs - 1 {
                        inner_left + inner_w - track_padding
                    } else {
                        item_left + tab_w
                    };
                    let item_top = inner_top + track_padding;
                    let item_bottom = inner_top + inner_h - track_padding;

                    let mut child_el = Element::new(WidgetKind::Empty);
                    child_el.on_click = on_select.as_ref().map(|cb| cb(i));

                    let child_id = *next_id;
                    *next_id += 1;
                    layout_children.push(LayoutNode {
                        id: child_id,
                        rect: RECT {
                            left: item_left,
                            top: item_top,
                            right: item_right,
                            bottom: item_bottom,
                        },
                        element: child_el,
                        children: Vec::new(),
                    });
                }
            }
        }
        _ => {}
    }

    LayoutNode {
        id,
        rect,
        element: el,
        children: layout_children,
    }
}
