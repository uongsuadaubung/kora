//! Declarative Virtual Table Component (`Table`)
//!
//! Web-style & Bootstrap 5-inspired Table architecture for native desktop:
//! - Composable table hierarchy: [`Table`], [`TableColDef`], [`TableRow`], [`TableCell`].
//! - Bootstrap 5-style modifiers: `.striped()`, `.bordered()`, `.borderless()`, `.hover()`, `.sm()` / `.compact()`, `.selectable()`.
//! - Default overflow scrolling with sticky/pinned header (`<thead>`) and auto-calculated scrollbar track.
//! - Pure declarative virtual [`Element<M>`] composition with zero child HWNDs or GDI subclassing.

use super::containers::{col, empty, row};
use super::element::{Element, FontScale, IntoElement, Length, Styleable, WidgetKind};
use super::style::{BgStyle, Style};
use super::text::{text, Text};
use std::sync::Arc;

/// Horizontal text alignment for table columns and cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Column definition for Table geometry and header styling.
#[derive(Debug, Clone)]
pub struct TableColDef {
    pub title: String,
    pub width: Length,
    pub align: TableAlign,
    pub min_w: Option<i32>,
    pub max_w: Option<i32>,
}

/// Creates a new table column definition with default 1fr flex width.
pub fn col_def(title: impl Into<String>) -> TableColDef {
    TableColDef {
        title: title.into(),
        width: Length::Flex(1),
        align: TableAlign::Left,
        min_w: None,
        max_w: None,
    }
}

impl TableColDef {
    #[inline]
    pub fn w(mut self, width: i32) -> Self {
        self.width = Length::Fixed(width);
        self
    }

    #[inline]
    pub fn flex(mut self, weight: u16) -> Self {
        self.width = Length::Flex(weight);
        self
    }

    #[inline]
    pub fn flex_1(self) -> Self {
        self.flex(1)
    }

    /// Automatically sizes column width based on the longest cell content in this column.
    #[inline]
    pub fn auto(mut self) -> Self {
        self.width = Length::Auto;
        self
    }

    /// Sets the minimum width constraint when auto-sizing.
    #[inline]
    pub fn min_w(mut self, min: i32) -> Self {
        self.min_w = Some(min);
        self
    }

    /// Sets the maximum width constraint when auto-sizing.
    #[inline]
    pub fn max_w(mut self, max: i32) -> Self {
        self.max_w = Some(max);
        self
    }

    #[inline]
    pub fn align(mut self, align: TableAlign) -> Self {
        self.align = align;
        self
    }

    #[inline]
    pub fn align_left(self) -> Self {
        self.align(TableAlign::Left)
    }

    #[inline]
    pub fn align_center(self) -> Self {
        self.align(TableAlign::Center)
    }

    #[inline]
    pub fn align_right(self) -> Self {
        self.align(TableAlign::Right)
    }
}

/// A cell inside a table row.
pub struct TableCell<M> {
    pub element: Element<M>,
    pub align: Option<TableAlign>,
    pub style: Style,
}

/// Helper constructor to create a [`TableCell`].
pub fn table_cell<M, E>(content: E) -> TableCell<M>
where
    E: IntoTableCell<M>,
{
    content.into_table_cell()
}

pub trait IntoTableCell<M> {
    fn into_table_cell(self) -> TableCell<M>;
}

impl<M> IntoTableCell<M> for TableCell<M> {
    #[inline]
    fn into_table_cell(self) -> TableCell<M> {
        self
    }
}

impl<M> IntoTableCell<M> for Element<M> {
    #[inline]
    fn into_table_cell(self) -> TableCell<M> {
        TableCell {
            element: self,
            align: None,
            style: Style::default(),
        }
    }
}

impl<M: 'static> IntoTableCell<M> for &str {
    #[inline]
    fn into_table_cell(self) -> TableCell<M> {
        TableCell {
            element: text(self).small().nowrap().into_element(),
            align: None,
            style: Style::default(),
        }
    }
}

impl<M: 'static> IntoTableCell<M> for String {
    #[inline]
    fn into_table_cell(self) -> TableCell<M> {
        TableCell {
            element: text(self).small().nowrap().into_element(),
            align: None,
            style: Style::default(),
        }
    }
}

impl<M: 'static> IntoTableCell<M> for Text<M> {
    #[inline]
    fn into_table_cell(self) -> TableCell<M> {
        TableCell {
            element: self.into_element(),
            align: None,
            style: Style::default(),
        }
    }
}

impl<M> TableCell<M> {
    #[inline]
    pub fn new(element: impl IntoElement<M>) -> Self {
        TableCell {
            element: element.into_element(),
            align: None,
            style: Style::default(),
        }
    }

    #[inline]
    pub fn align(mut self, align: TableAlign) -> Self {
        self.align = Some(align);
        self
    }

    #[inline]
    pub fn align_left(self) -> Self {
        self.align(TableAlign::Left)
    }

    #[inline]
    pub fn align_center(self) -> Self {
        self.align(TableAlign::Center)
    }

    #[inline]
    pub fn align_right(self) -> Self {
        self.align(TableAlign::Right)
    }
}

impl<M> Styleable for TableCell<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

/// A row inside a table body.
pub struct TableRow<M> {
    pub cells: Vec<TableCell<M>>,
    pub on_click: Option<M>,
    pub is_selected: bool,
    pub is_disabled: bool,
    pub style: Style,
}

/// Helper constructor to create a [`TableRow`] from an iterator of cells.
pub fn table_row<M, I, C>(cells: I) -> TableRow<M>
where
    I: IntoIterator<Item = C>,
    C: IntoTableCell<M>,
{
    TableRow {
        cells: cells.into_iter().map(|c| c.into_table_cell()).collect(),
        on_click: None,
        is_selected: false,
        is_disabled: false,
        style: Style::default(),
    }
}

impl<M> TableRow<M> {
    #[inline]
    pub fn cell(mut self, cell: impl IntoTableCell<M>) -> Self {
        self.cells.push(cell.into_table_cell());
        self
    }

    #[inline]
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }

    #[inline]
    pub fn selected(mut self, is_selected: bool) -> Self {
        self.is_selected = is_selected;
        self
    }

    #[inline]
    pub fn disabled(mut self, is_disabled: bool) -> Self {
        self.is_disabled = is_disabled;
        self
    }
}

impl<M> Styleable for TableRow<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

/// Declarative Table Component with Bootstrap 5-inspired styling.
pub struct Table<M> {
    pub columns: Vec<TableColDef>,
    pub rows: Vec<TableRow<M>>,
    pub max_rows: Option<usize>,
    pub scroll_offset: usize,
    pub on_scroll: Option<Arc<dyn Fn(i32) -> M + Send + Sync>>,
    pub striped: bool,
    pub bordered: bool,
    pub borderless: bool,
    pub hover: bool,
    pub compact: bool,
    pub hide_header: bool,
    pub column_dividers: bool,
    pub empty_text: String,
    pub style: Style,
}

/// Creates a new declarative table builder.
pub fn table<M>() -> Table<M> {
    Table {
        columns: Vec::new(),
        rows: Vec::new(),
        max_rows: Some(6), // By default, limits to 6 rows and scrolls when overflowing
        scroll_offset: 0,
        on_scroll: None,
        striped: false,
        bordered: true,
        borderless: false,
        hover: true,
        compact: false,
        hide_header: false,
        column_dividers: true,
        empty_text: String::from("Không có dữ liệu"),
        style: Style::default(),
    }
}

impl<M: Clone + Send + Sync + 'static> Table<M> {
    #[inline]
    pub fn columns<I>(mut self, cols: I) -> Self
    where
        I: IntoIterator<Item = TableColDef>,
    {
        self.columns = cols.into_iter().collect();
        self
    }

    #[inline]
    pub fn column(mut self, col: TableColDef) -> Self {
        self.columns.push(col);
        self
    }

    #[inline]
    pub fn rows<I>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = TableRow<M>>,
    {
        self.rows = rows.into_iter().collect();
        self
    }

    #[inline]
    pub fn row(mut self, row: TableRow<M>) -> Self {
        self.rows.push(row);
        self
    }

    /// Sets the maximum visible rows before overflow scrolling triggers (defaults to `Some(6)`).
    #[inline]
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = Some(n);
        self
    }

    /// Disables row capping, rendering all rows without pagination or scrolling.
    #[inline]
    pub fn unbounded(mut self) -> Self {
        self.max_rows = None;
        self
    }

    /// Sets current vertical scroll row offset.
    #[inline]
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Binds an on-scroll event handler returning message `M` with scroll row delta.
    #[inline]
    pub fn on_scroll<F>(mut self, handler: F) -> Self
    where
        F: Fn(i32) -> M + Send + Sync + 'static,
    {
        self.on_scroll = Some(Arc::new(handler));
        self
    }

    /// Enables alternating row background colors (`table-striped`).
    #[inline]
    pub fn striped(mut self) -> Self {
        self.striped = true;
        self
    }

    /// Enables or disables explicit cell and outer borders (`table-bordered`).
    #[inline]
    pub fn bordered(mut self, is_bordered: bool) -> Self {
        self.bordered = is_bordered;
        self
    }

    /// Disables interior borders between rows (`table-borderless`).
    #[inline]
    pub fn borderless(mut self) -> Self {
        self.borderless = true;
        self
    }

    /// Enables row hover highlight effect (`table-hover`).
    #[inline]
    pub fn hover(mut self, has_hover: bool) -> Self {
        self.hover = has_hover;
        self
    }

    /// Uses compact row padding and height (`table-sm`).
    #[inline]
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// Alias for `.compact()`.
    #[inline]
    pub fn sm(self) -> Self {
        self.compact()
    }

    /// Hides the `<thead>` header bar.
    #[inline]
    pub fn hide_header(mut self, is_hidden: bool) -> Self {
        self.hide_header = is_hidden;
        self
    }

    /// Sets fallback text displayed when table rows are empty.
    #[inline]
    pub fn empty_text(mut self, text: impl Into<String>) -> Self {
        self.empty_text = text.into();
        self
    }

    /// Enables or disables vertical divider lines between columns.
    #[inline]
    pub fn column_dividers(mut self, enabled: bool) -> Self {
        self.column_dividers = enabled;
        self
    }
}

/// Estimates visual content width of an element (in pixels) for auto-sizing table columns.
fn estimate_element_width<M>(el: &Element<M>) -> i32 {
    let pad_x = el.style.padding.horizontal();
    let inner_w = match &el.kind {
        WidgetKind::Text { content, font, .. } => {
            let avg_char_w = match font {
                FontScale::Hero => 11,
                FontScale::Title => 9,
                FontScale::Bold => 8,
                FontScale::Normal => 7,
                FontScale::Small => 6,
            };
            content.chars().count() as i32 * avg_char_w
        }
        WidgetKind::Label { content, font, .. } => {
            let avg_char_w = match font {
                FontScale::Title | FontScale::Bold => 8,
                _ => 7,
            };
            content.chars().count() as i32 * avg_char_w
        }
        WidgetKind::Badge { text, .. } => (text.chars().count() as i32 * 8) + 20,
        WidgetKind::Button { label, .. } => (label.chars().count() as i32 * 8) + 24,
        WidgetKind::Col(children) => {
            children.iter().map(estimate_element_width).max().unwrap_or(0)
        }
        WidgetKind::Row(children) => {
            children.iter().map(estimate_element_width).sum()
        }
        _ => match el.style.width {
            Length::Fixed(w) => w,
            _ => 0,
        },
    };
    pad_x + inner_w
}

impl<M: Clone + Send + Sync + 'static> Table<M> {
    /// Compiles the table into a virtual [`Element<M>`] tree.
    pub fn build(self) -> Element<M> {
        let (row_pad_y, row_pad_x, row_min_h) = if self.compact {
            (1, 2, 26)
        } else {
            (2, 3, 30)
        };

        let total_rows = self.rows.len();
        let is_scrolling = match self.max_rows {
            Some(max_r) => total_rows > max_r,
            None => false,
        };

        let page_size = self.max_rows.unwrap_or(total_rows.max(1));
        let max_offset = total_rows.saturating_sub(page_size);
        let offset = self.scroll_offset.min(max_offset);
        let columns = self.columns;
        let rows = self.rows;

        // Auto-calculate column widths for Length::Auto based on longest content
        let pad_x_px = row_pad_x * 4;
        let mut resolved_widths = Vec::with_capacity(columns.len());
        for (c_idx, col_def) in columns.iter().enumerate() {
            match col_def.width {
                Length::Auto => {
                    let header_w = if !self.hide_header {
                        (col_def.title.chars().count() as i32 * 8) + (pad_x_px * 2)
                    } else {
                        0
                    };

                    let mut max_cell_w = header_w;
                    let check_rows = if is_scrolling {
                        let start = offset.saturating_sub(page_size);
                        let end = (offset + page_size * 2).min(rows.len());
                        &rows[start..end]
                    } else {
                        &rows[..]
                    };

                    for row in check_rows {
                        if let Some(cell) = row.cells.get(c_idx) {
                            let content_w = estimate_element_width(&cell.element);
                            let cell_total_w = content_w + (pad_x_px * 2);
                            if cell_total_w > max_cell_w {
                                max_cell_w = cell_total_w;
                            }
                        }
                    }

                    let min_constraint = col_def.min_w.unwrap_or(40);
                    let mut final_w = (max_cell_w + 4).max(min_constraint);
                    if let Some(max) = col_def.max_w {
                        final_w = final_w.min(max);
                    }
                    resolved_widths.push(Length::Fixed(final_w));
                }
                other => {
                    resolved_widths.push(other);
                }
            }
        }

        let mut sections = Vec::with_capacity(2);

        // 1. Sticky Header Bar (`<thead>`)
        if !self.hide_header && !columns.is_empty() {
            let col_count = columns.len();
            let mut header_cells = Vec::with_capacity(col_count + if is_scrolling { 1 } else { 0 });
            for (c_idx, col_def) in columns.iter().enumerate() {
                let mut cell_txt = text(&col_def.title).bold().small().nowrap();
                cell_txt = match col_def.align {
                    TableAlign::Left => cell_txt,
                    TableAlign::Center => cell_txt.text_center(),
                    TableAlign::Right => cell_txt.text_right(),
                };

                let mut cell_el = col([cell_txt.into_element()])
                    .py(row_pad_y)
                    .px(row_pad_x)
                    .justify_center();

                cell_el.style.width = resolved_widths[c_idx];
                if self.column_dividers && c_idx + 1 < col_count {
                    cell_el = cell_el.border_right();
                }
                header_cells.push(cell_el);
            }

            // Compensate for body scrollbar width so header columns align with body columns
            if is_scrolling {
                header_cells.push(empty().w(12));
            }

            let mut header_row = row(header_cells)
                .bg_header()
                .border_bottom()
                .w_full()
                .items_center();
            header_row.style.min_h = row_min_h;
            sections.push(header_row);
        }

        // 2. Body Rows (`<tbody>`)
        let body_content = if rows.is_empty() {
            col([
                text(&self.empty_text)
                    .secondary()
                    .small()
                    .text_center()
                    .w_full()
                    .into_element(),
            ])
            .py(4)
            .into_element()
        } else {
            let visible_rows: Vec<TableRow<M>> = if is_scrolling {
                rows.into_iter().skip(offset).take(page_size).collect()
            } else {
                rows
            };

            let mut rendered_rows = Vec::with_capacity(visible_rows.len());
            let visible_count = visible_rows.len();

            for (idx, row_item) in visible_rows.into_iter().enumerate() {
                let actual_idx = offset + idx;
                let is_even = actual_idx % 2 == 0;

                let row_bg = if row_item.is_selected {
                    BgStyle::TabActive
                } else if self.striped && !is_even {
                    BgStyle::CardHeader
                } else {
                    BgStyle::None
                };

                let col_count = columns.len();
                let mut row_cells = Vec::with_capacity(row_item.cells.len());
                for (c_idx, cell) in row_item.cells.into_iter().enumerate() {
                    let col_def = columns.get(c_idx);
                    let col_width = resolved_widths.get(c_idx).copied().unwrap_or(Length::Flex(1));
                    let col_align = col_def.map(|c| c.align).unwrap_or(TableAlign::Left);
                    let align = cell.align.unwrap_or(col_align);

                    let mut cell_el = col([cell.element])
                        .py(row_pad_y)
                        .px(row_pad_x)
                        .justify_center();

                    cell_el.style.width = col_width;
                    if align == TableAlign::Right {
                        cell_el = cell_el.items_end();
                    } else if align == TableAlign::Center {
                        cell_el = cell_el.items_center();
                    }

                    if self.column_dividers && c_idx + 1 < col_count {
                        cell_el = cell_el.border_right();
                    }

                    row_cells.push(cell_el);
                }

                let mut row_el = row(row_cells).bg(row_bg).w_full().items_center();
                row_el.style.min_h = row_min_h;

                if row_item.is_selected {
                    row_el = row_el.rounded_md();
                }

                if !self.borderless && idx + 1 < visible_count {
                    row_el = row_el.border_bottom();
                }

                if let Some(msg) = row_item.on_click {
                    row_el = row_el.on_click(msg);
                }

                rendered_rows.push(row_el);
            }

            if is_scrolling {
                // Auto scrollbar calculation
                let track_h = (rendered_rows.len() as i32 * row_min_h).max(120);
                let thumb_ratio = (page_size as f32 / total_rows as f32).clamp(0.15, 0.85);
                let thumb_h = ((track_h as f32 * thumb_ratio) as i32).max(24);
                let travel = (track_h - thumb_h).max(0);
                let top_space = if max_offset > 0 {
                    ((travel as f32 * offset as f32) / max_offset as f32) as i32
                } else {
                    0
                };
                let bottom_space = travel - top_space;

                let on_scroll = self.on_scroll.clone();

                let mut top_track = col([]).w(12);
                if top_space > 0 {
                    top_track = top_track.h(top_space);
                    if let Some(ref cb) = on_scroll {
                        top_track = top_track.on_click(cb(-(page_size as i32)));
                    }
                }

                let mut bottom_track = col([]).w(12);
                if bottom_space > 0 {
                    bottom_track = bottom_track.h(bottom_space);
                    if let Some(ref cb) = on_scroll {
                        bottom_track = bottom_track.on_click(cb(page_size as i32));
                    }
                }

                let thumb = col([])
                    .w(5)
                    .h(thumb_h)
                    .bg(BgStyle::TabHover)
                    .rounded_pill();

                let scrollbar = col([
                    if top_space > 0 { top_track.into_element() } else { empty().into_element() },
                    thumb.into_element(),
                    if bottom_space > 0 { bottom_track.into_element() } else { empty().into_element() },
                ])
                .w(12)
                .items_center()
                .py(1);

                row([col(rendered_rows).flex_1().into_element(), scrollbar.into_element()])
                    .w_full()
                    .items_center()
                    .into_element()
            } else {
                col(rendered_rows).w_full().into_element()
            }
        };

        sections.push(body_content);

        let mut outer = if sections.len() == 1 {
            sections.pop().unwrap().bg_card().w_full()
        } else {
            col(sections).bg_card().w_full()
        };
        if self.bordered {
            outer = outer.border().rounded_md();
        }
        if self.style.width != Length::Auto {
            outer.style.width = self.style.width;
        }
        if self.style.height != Length::Auto {
            outer.style.height = self.style.height;
        }
        if self.style.min_h > 0 {
            outer.style.min_h = self.style.min_h;
        }
        if self.style.min_w > 0 {
            outer.style.min_w = self.style.min_w;
        }
        outer
    }
}

impl<M: Clone + Send + Sync + 'static> Styleable for Table<M> {
    #[inline]
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

impl<M: Clone + Send + Sync + 'static> IntoElement<M> for Table<M> {
    #[inline]
    fn into_element(self) -> Element<M> {
        self.build()
    }
}

impl<M: Clone + Send + Sync + 'static> From<Table<M>> for Element<M> {
    #[inline]
    fn from(t: Table<M>) -> Self {
        t.into_element()
    }
}
