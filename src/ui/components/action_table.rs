//! Modern Universal ActionTable Component (Web-style Canvas / Owner-Draw Architecture)
//! Features:
//! - Fixed Header with crisp vertical column separators and horizontal divider line.
//! - Smooth double-buffered rendering (BitBlt from Memory DC) - zero flicker.
//! - Data-driven TableCell model: Text, SecondaryText, Badge (with BadgeConfig), Actions, and Custom.
//! - Pixel-perfect row hover and row selection highlighting.
//! - Integrated vertical scrolling with native Win32 scrollbar.
//! - Dynamic in-cell action buttons (arbitrary labels, variants, callbacks) with hand cursor (IDC_HAND) and hover states.

use crate::ui::colors::ThemePalette;
use crate::ui::tokens::borders;
use crate::ui::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, Once};

const HEADER_HEIGHT: i32 = 30;
const ROW_HEIGHT: i32 = 32;
const IDC_HAND_ID: usize = 32649;
const IDC_ARROW_ID: usize = 32512;
const DT_END_ELLIPSIS: u32 = 0x00004000;
const DT_RIGHT: u32 = 0x00000002;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_SIZE: u32 = 0x0005;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SETCURSOR: u32 = 0x0020;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_MOUSELEAVE: u32 = 0x02A3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    KeyPill,
    Danger,
}

#[derive(Clone, Debug)]
pub struct BadgeConfig {
    pub text: String,
    pub variant: BadgeVariant,
    pub suffix: Option<String>,
}

impl BadgeConfig {
    pub fn key(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::KeyPill,
            suffix: None,
        }
    }

    pub fn key_with_arrow(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::KeyPill,
            suffix: Some("➜".to_string()),
        }
    }

    pub fn danger_key(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Danger,
            suffix: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionVariant {
    Primary,   // Accent / Blue (Edit, Run)
    Danger,    // Danger / Red (Delete, Block)
    Secondary, // Neutral / Gray
}

#[derive(Clone)]
pub struct TableAction {
    pub label: String,
    pub variant: ActionVariant,
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl TableAction {
    pub fn new(label: impl Into<String>, variant: ActionVariant) -> Self {
        Self {
            label: label.into(),
            variant,
            on_click: None,
        }
    }

    pub fn with_callback<F: Fn() + Send + Sync + 'static>(mut self, cb: F) -> Self {
        self.on_click = Some(Arc::new(cb));
        self
    }

    pub fn edit<F: Fn() + Send + Sync + 'static>(label: impl Into<String>, cb: F) -> Self {
        Self::new(label, ActionVariant::Primary).with_callback(cb)
    }

    pub fn delete<F: Fn() + Send + Sync + 'static>(label: impl Into<String>, cb: F) -> Self {
        Self::new(label, ActionVariant::Danger).with_callback(cb)
    }

    pub fn secondary<F: Fn() + Send + Sync + 'static>(label: impl Into<String>, cb: F) -> Self {
        Self::new(label, ActionVariant::Secondary).with_callback(cb)
    }
}

impl std::fmt::Debug for TableAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TableAction(label={:?}, variant={:?}, has_cb={})",
            self.label,
            self.variant,
            self.on_click.is_some()
        )
    }
}

pub type CustomCellRenderer = Arc<dyn Fn(isize, &RECT, &ThemePalette) + Send + Sync>;

#[derive(Clone)]
pub enum TableCell {
    Text(String),
    SecondaryText(String),
    Badge(BadgeConfig),
    Actions(Vec<TableAction>),
    Custom(CustomCellRenderer),
}

impl std::fmt::Debug for TableCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "Text({:?})", s),
            Self::SecondaryText(s) => write!(f, "SecondaryText({:?})", s),
            Self::Badge(b) => write!(f, "Badge({:?})", b),
            Self::Actions(acts) => write!(f, "Actions({:?})", acts),
            Self::Custom(_) => write!(f, "Custom(<closure>)"),
        }
    }
}

#[derive(Clone, Default)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

impl TableRow {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn cell(mut self, cell: TableCell) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::Text(text.into()));
        self
    }

    pub fn secondary(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::SecondaryText(text.into()));
        self
    }

    pub fn key(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::Badge(BadgeConfig::key(text)));
        self
    }

    pub fn key_with_arrow(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::Badge(BadgeConfig::key_with_arrow(text)));
        self
    }

    pub fn danger_key(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::Badge(BadgeConfig::danger_key(text)));
        self
    }

    pub fn badge(mut self, badge: BadgeConfig) -> Self {
        self.cells.push(TableCell::Badge(badge));
        self
    }

    pub fn actions(mut self, actions: Vec<TableAction>) -> Self {
        self.cells.push(TableCell::Actions(actions));
        self
    }
}

impl From<TableRow> for Vec<TableCell> {
    fn from(r: TableRow) -> Self {
        r.cells
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnWidth {
    Fixed(i32),
    Flex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CellAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColumnStyle {
    #[default]
    Default,
    KeyBadge,
    KeyBadgeWithArrow,
    DangerKeyBadge,
    CenterSecondary,
    OriginBadge,
    Action,
}

#[derive(Clone, Debug)]
pub struct ColumnDef {
    pub title: String,
    pub width: ColumnWidth,
    pub align: CellAlign,
    pub style: ColumnStyle,
}

impl ColumnDef {
    pub fn fixed(title: impl Into<String>, width: i32, style: ColumnStyle) -> Self {
        Self {
            title: title.into(),
            width: ColumnWidth::Fixed(width),
            align: match style {
                ColumnStyle::CenterSecondary | ColumnStyle::Action => CellAlign::Center,
                _ => CellAlign::Left,
            },
            style,
        }
    }

    pub fn flex(title: impl Into<String>, style: ColumnStyle) -> Self {
        Self {
            title: title.into(),
            width: ColumnWidth::Flex,
            align: match style {
                ColumnStyle::CenterSecondary | ColumnStyle::Action => CellAlign::Center,
                _ => CellAlign::Left,
            },
            style,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionHover {
    None,
    Action { row: usize, action_idx: usize },
}

pub type ActionCallback = Arc<dyn Fn(i32) + Send + Sync>;

pub struct ActionTableState {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<TableCell>>,
    pub font: isize,
    pub font_bold: isize,
    pub hover_row: Option<usize>,
    pub hover_action: ActionHover,
    pub selected_row: Option<usize>,
    pub scroll_y: i32,
    pub on_select: Option<ActionCallback>,
    pub is_dragging_scroll: bool,
    pub scroll_drag_start_y: i32,
    pub scroll_drag_start_val: i32,
    pub is_hover_scrollbar: bool,
    /// Header height in px; 0 hides the header row
    pub header_h: i32,
}

pub struct ActionTable {
    hwnd: isize,
    state: Arc<Mutex<ActionTableState>>,
}

static REGISTER_CLASS_ONCE: Once = Once::new();
const CLASS_NAME: &str = "Kora_ActionTable";

fn ensure_class_registered() {
    REGISTER_CLASS_ONCE.call_once(|| unsafe {
        let class_name_w = to_wide(CLASS_NAME);
        let wnd_class = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfn_wnd_proc: Some(action_table_wndproc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: GetModuleHandleW(null_mut()),
            h_icon: 0,
            h_cursor: LoadCursorW(0, IDC_ARROW_ID),
            h_br_background: 0,
            lpsz_menu_name: null_mut(),
            lpsz_class_name: class_name_w.as_ptr(),
            h_icon_sm: 0,
        };
        RegisterClassExW(&wnd_class);
    });
}

fn pt_in_rect(x: i32, y: i32, rc: &RECT) -> bool {
    x >= rc.left && x <= rc.right && y >= rc.top && y <= rc.bottom
}

fn get_dynamic_action_button_rects(rc: &RECT, count: usize) -> Vec<RECT> {
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

fn compute_column_rects(columns: &[ColumnDef], client_width: i32) -> Vec<(i32, i32)> {
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

fn compute_scroll_thumb_rect(
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

unsafe extern "system" fn action_table_wndproc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<ActionTableState> };

    if msg == WM_CREATE {
        return 0;
    }

    if msg == WM_DESTROY {
        if !state_ptr.is_null() {
            unsafe {
                drop(Arc::from_raw(state_ptr));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
        }
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    if state_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
    let state_mutex = unsafe { &*state_ptr };
    // try_lock: some calls made while holding the lock (e.g. ReleaseCapture) re-enter this proc synchronously.
    let header_h = state_mutex.try_lock().map(|g| g.header_h).unwrap_or(HEADER_HEIGHT);

    if msg == WM_ERASEBKGND {
        return 1;
    }

    if msg == WM_SIZE {
        let h = ((lparam >> 16) & 0xFFFF) as i32;
        if let Ok(mut g) = state_mutex.lock() {
            let body_h = (h - header_h).max(0);
            let total_h = g.rows.len() as i32 * ROW_HEIGHT;
            let max_scroll = (total_h - body_h).max(0);
            if g.scroll_y > max_scroll {
                g.scroll_y = max_scroll;
            }
        }
        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }
        return 0;
    }

    if msg == WM_MOUSEWHEEL {
        let delta = ((wparam >> 16) as i16) as i32;
        let mut client_rc = RECT::default();
        unsafe { GetClientRect(hwnd, &mut client_rc); }
        let body_h = (client_rc.bottom - header_h).max(0);

        let changed = if let Ok(mut g) = state_mutex.lock() {
            let total_h = g.rows.len() as i32 * ROW_HEIGHT;
            let max_scroll = (total_h - body_h).max(0);
            let scroll_lines = (delta / 120) * (ROW_HEIGHT * 3);
            let old_scroll = g.scroll_y;
            g.scroll_y = (g.scroll_y - scroll_lines).clamp(0, max_scroll);
            old_scroll != g.scroll_y
        } else {
            false
        };

        if changed {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
        return 0;
    }

    if msg == WM_MOUSEMOVE {
        let x = (lparam as i16) as i32;
        let y = ((lparam >> 16) as i16) as i32;

        let mut client_rc = RECT::default();
        unsafe { GetClientRect(hwnd, &mut client_rc); }
        let width = client_rc.right - client_rc.left;
        let height = client_rc.bottom - client_rc.top;
        let body_h = (height - header_h).max(0);

        let (need_redraw, is_hand_cursor, dirty_row1, dirty_row2, sb_changed, is_drag) = if let Ok(mut g) = state_mutex.lock() {
            if g.is_dragging_scroll {
                let total_h = g.rows.len() as i32 * ROW_HEIGHT;
                let max_scroll = (total_h - body_h).max(0);
                let changed = if max_scroll > 0 {
                    let track_top = header_h + 3;
                    let track_bottom = height - 3;
                    let track_h = (track_bottom - track_top).max(10);
                    let thumb_h = ((body_h as f32 / total_h as f32) * track_h as f32).clamp(18.0, track_h as f32) as i32;
                    let travel = (track_h - thumb_h).max(1);
                    let delta_y = y - g.scroll_drag_start_y;
                    let scroll_delta = ((delta_y as f32 / travel as f32) * max_scroll as f32) as i32;
                    let new_scroll = (g.scroll_drag_start_val + scroll_delta).clamp(0, max_scroll);
                    let changed = g.scroll_y != new_scroll;
                    g.scroll_y = new_scroll;
                    changed
                } else {
                    false
                };
                (changed, false, None, None, false, true)
            } else {
                let old_hot_row = g.hover_row;
                let old_action = g.hover_action;
                let old_hover_scroll = g.is_hover_scrollbar;

                let total_h = g.rows.len() as i32 * ROW_HEIGHT;
                let has_scroll = total_h > body_h;
                let hover_scrollbar = has_scroll && x >= width - 14 && y >= header_h;
                g.is_hover_scrollbar = hover_scrollbar;

                if !hover_scrollbar && y >= header_h && y < client_rc.bottom {
                    let row_idx = ((y - header_h + g.scroll_y) / ROW_HEIGHT) as usize;
                    if row_idx < g.rows.len() {
                        g.hover_row = Some(row_idx);
                        let mut action_hover = ActionHover::None;

                        let col_rects = compute_column_rects(&g.columns, width);
                        let row_y = header_h + (row_idx as i32 * ROW_HEIGHT) - g.scroll_y;

                        if let Some(row_cells) = g.rows.get(row_idx) {
                            for (col_idx, &(col_x, col_w)) in col_rects.iter().enumerate() {
                                if x >= col_x && x < col_x + col_w {
                                    if let Some(TableCell::Actions(actions)) = row_cells.get(col_idx) {
                                        let cell_rc = RECT {
                                            left: col_x,
                                            top: row_y,
                                            right: col_x + col_w,
                                            bottom: row_y + ROW_HEIGHT - 1,
                                        };
                                        let btn_rects = get_dynamic_action_button_rects(&cell_rc, actions.len());
                                        for (act_idx, btn_rc) in btn_rects.iter().enumerate() {
                                            if pt_in_rect(x, y, btn_rc) {
                                                action_hover = ActionHover::Action { row: row_idx, action_idx: act_idx };
                                                break;
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                        g.hover_action = action_hover;
                    } else {
                        g.hover_row = None;
                        g.hover_action = ActionHover::None;
                    }
                } else {
                    g.hover_row = None;
                    g.hover_action = ActionHover::None;
                }

                let row_changed = old_hot_row != g.hover_row || old_action != g.hover_action;
                let sb_changed = old_hover_scroll != g.is_hover_scrollbar;
                let hand = g.hover_action != ActionHover::None;
                (row_changed || sb_changed, hand, old_hot_row, g.hover_row, sb_changed, false)
            }
        } else {
            (false, false, None, None, false, false)
        };

        if need_redraw {
            unsafe {
                if is_drag {
                    InvalidateRect(hwnd, null_mut(), 0);
                } else {
                    let scroll_y = state_mutex.lock().map(|g| g.scroll_y).unwrap_or(0);
                    let invalidate_row = |r_opt: Option<usize>| {
                        if let Some(r) = r_opt {
                            let ry = header_h + (r as i32 * ROW_HEIGHT) - scroll_y;
                            if ry < height && ry + ROW_HEIGHT > header_h {
                                let r_rc = RECT {
                                    left: 0,
                                    top: ry.max(header_h),
                                    right: width,
                                    bottom: (ry + ROW_HEIGHT).min(height),
                                };
                                InvalidateRect(hwnd, &r_rc, 0);
                            }
                        }
                    };
                    invalidate_row(dirty_row1);
                    if dirty_row1 != dirty_row2 {
                        invalidate_row(dirty_row2);
                    }
                    if sb_changed {
                        let sb_rc = RECT {
                            left: width - 14,
                            top: header_h,
                            right: width,
                            bottom: height,
                        };
                        InvalidateRect(hwnd, &sb_rc, 0);
                    }
                }
            }
        }

        unsafe {
            if is_hand_cursor {
                SetCursor(LoadCursorW(0, IDC_HAND_ID));
            } else {
                SetCursor(LoadCursorW(0, IDC_ARROW_ID));
            }
            let mut tme = TRACKMOUSEEVENT {
                cb_size: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dw_flags: TME_LEAVE,
                hwnd_track: hwnd,
                dw_hover_time: 0,
            };
            TrackMouseEvent(&mut tme);
        }
        return 0;
    }

    if msg == WM_SETCURSOR
        && let Ok(g) = state_mutex.lock()
        && g.hover_action != ActionHover::None
    {
        unsafe {
            SetCursor(LoadCursorW(0, IDC_HAND_ID));
        }
        return 1;
    }

    if msg == WM_MOUSELEAVE {
        let changed = if let Ok(mut g) = state_mutex.lock() {
            let was = g.hover_row.is_some() || g.hover_action != ActionHover::None || g.is_hover_scrollbar;
            g.hover_row = None;
            g.hover_action = ActionHover::None;
            g.is_hover_scrollbar = false;
            was
        } else {
            false
        };
        if changed {
            unsafe {
                InvalidateRect(hwnd, null_mut(), 0);
            }
        }
        return 0;
    }

    if msg == WM_LBUTTONDOWN {
        let x = (lparam as i16) as i32;
        let y = ((lparam >> 16) as i16) as i32;

        let mut client_rc = RECT::default();
        unsafe { GetClientRect(hwnd, &mut client_rc); }
        let width = client_rc.right - client_rc.left;
        let height = client_rc.bottom - client_rc.top;
        let body_h = (height - header_h).max(0);

        let (action, hot_row, cb_sel, clicked_scroll) = if let Ok(mut g) = state_mutex.lock() {
            let total_rows = g.rows.len();
            let total_h = total_rows as i32 * ROW_HEIGHT;
            let has_scroll = total_h > body_h;

            if has_scroll && x >= width - 14 && y >= header_h {
                if let Some((thumb_rc, max_scroll)) = compute_scroll_thumb_rect(width, height, header_h, total_rows, g.scroll_y, true) {
                    if pt_in_rect(x, y, &thumb_rc) {
                        g.is_dragging_scroll = true;
                        g.scroll_drag_start_y = y;
                        g.scroll_drag_start_val = g.scroll_y;
                        unsafe { SetCapture(hwnd); }
                    } else if y < thumb_rc.top {
                        g.scroll_y = (g.scroll_y - body_h).max(0);
                    } else if y > thumb_rc.bottom {
                        g.scroll_y = (g.scroll_y + body_h).min(max_scroll);
                    }
                }
                (ActionHover::None, None, None, true)
            } else {
                let act = g.hover_action;
                let hr = g.hover_row;
                if hr.is_some() {
                    g.selected_row = hr;
                }
                (act, hr, g.on_select.clone(), false)
            }
        } else {
            (ActionHover::None, None, None, false)
        };

        unsafe {
            InvalidateRect(hwnd, null_mut(), 0);
        }

        if clicked_scroll {
            return 0;
        }

        match action {
            ActionHover::Action { row, action_idx } => {
                let cb = if let Ok(g) = state_mutex.lock() {
                    if let Some(row_cells) = g.rows.get(row) {
                        let mut found_cb = None;
                        for cell in row_cells {
                            if let TableCell::Actions(actions) = cell
                                && let Some(act) = actions.get(action_idx)
                            {
                                found_cb = act.on_click.clone();
                                break;
                            }
                        }
                        found_cb
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(f) = cb {
                    f();
                }
                return 0;
            }
            ActionHover::None => {
                if let (Some(row), Some(cb)) = (hot_row, cb_sel) {
                    cb(row as i32);
                }
            }
        }
        return 0;
    }

    if msg == WM_LBUTTONUP {
        let changed = if let Ok(mut g) = state_mutex.lock() {
            if g.is_dragging_scroll {
                g.is_dragging_scroll = false;
                unsafe { ReleaseCapture(); }
                true
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            unsafe { InvalidateRect(hwnd, null_mut(), 0); }
        }
        return 0;
    }

    if msg == WM_LBUTTONDBLCLK {
        let cb = if let Ok(g) = state_mutex.lock() {
            if matches!(g.hover_action, ActionHover::Action { .. }) {
                None
            } else if let Some(row_idx) = g.hover_row {
                if let Some(row_cells) = g.rows.get(row_idx) {
                    row_cells.iter().find_map(|cell| {
                        if let TableCell::Actions(actions) = cell {
                            actions.iter().find_map(|act| {
                                if act.variant == ActionVariant::Primary {
                                    act.on_click.clone()
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        if let Some(f) = cb {
            f();
        }
        return 0;
    }

    if msg == WM_PAINT {
        let mut ps = PAINTSTRUCT {
            hdc: 0,
            f_erase: 0,
            rc_paint: RECT::default(),
            f_restore: 0,
            f_inc_update: 0,
            rgb_reserved: [0; 32],
        };
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        if hdc != 0 {
            if let Ok(g) = state_mutex.lock() {
                render_table(hwnd, hdc, &g);
            }
            unsafe { EndPaint(hwnd, &ps) };
        }
        return 0;
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn render_table(hwnd: isize, hdc: isize, state: &ActionTableState) {
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

        // 5. BitBlt entire buffer to window DC
        BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

        SelectObject(mem_dc, old_bmp);
        DeleteObject(mem_bmp);
        DeleteDC(mem_dc);
    }
}

impl ActionTable {
    #[allow(clippy::too_many_arguments)]
    pub fn create_styled(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        font: isize,
        columns: &[(&str, i32, ColumnStyle)],
    ) -> Option<Self> {
        let mut defs = Vec::with_capacity(columns.len());
        for &(title, w, style) in columns {
            defs.push(ColumnDef::fixed(title, w, style));
        }
        Self::create_with_defs(parent, id, x, y, width, height, font, defs)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_with_defs(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        font: isize,
        columns: Vec<ColumnDef>,
    ) -> Option<Self> {
        ensure_class_registered();

        let font_bold = crate::ui::theme::get_app_font_bold();

        let state = Arc::new(Mutex::new(ActionTableState {
            columns,
            rows: Vec::new(),
            font,
            font_bold,
            hover_row: None,
            hover_action: ActionHover::None,
            selected_row: None,
            scroll_y: 0,
            on_select: None,
            is_dragging_scroll: false,
            scroll_drag_start_y: 0,
            scroll_drag_start_val: 0,
            is_hover_scrollbar: false,
            header_h: HEADER_HEIGHT,
        }));

        let raw_state = Arc::into_raw(state.clone());
        let class_name_w = to_wide(CLASS_NAME);

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name_w.as_ptr(),
                null_mut(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                x,
                y,
                width,
                height,
                parent,
                id as isize,
                GetModuleHandleW(null_mut()),
                null_mut(),
            )
        };

        if hwnd == 0 {
            unsafe {
                drop(Arc::from_raw(raw_state));
            }
            return None;
        }

        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw_state as isize);
        }

        Some(Self { hwnd, state })
    }

    /// Data-driven setter for rows using TableRow builder
    pub fn set_rows(&self, rows: impl IntoIterator<Item = TableRow>) {
        let cells: Vec<Vec<TableCell>> = rows.into_iter().map(|r| r.cells).collect();
        self.set_raw_cells(cells);
    }

    /// Low-level raw cells setter
    pub fn set_raw_cells(&self, rows: Vec<Vec<TableCell>>) {
        if let Ok(mut g) = self.state.lock() {
            g.rows = rows;
            let total_rows = g.rows.len();
            if let Some(r) = g.selected_row {
                g.selected_row = if total_rows == 0 { None } else { Some(r.min(total_rows - 1)) };
            }
            if let Some(r) = g.hover_row {
                g.hover_row = if total_rows == 0 { None } else { Some(r.min(total_rows - 1)) };
            }
            g.hover_action = ActionHover::None;
            let mut rc = RECT::default();
            unsafe { GetClientRect(self.hwnd, &mut rc); }
            let body_h = (rc.bottom - g.header_h).max(0);
            let total_h = total_rows as i32 * ROW_HEIGHT;
            let max_scroll = (total_h - body_h).max(0);
            if g.scroll_y > max_scroll {
                g.scroll_y = max_scroll;
            }
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    /// Hides the column header row (body rows start at y = 0)
    pub fn hide_header(&self) {
        if let Ok(mut g) = self.state.lock() {
            g.header_h = 0;
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.state.lock() {
            g.rows.clear();
            g.hover_row = None;
            g.hover_action = ActionHover::None;
            g.selected_row = None;
            g.scroll_y = 0;
            g.is_dragging_scroll = false;
            g.is_hover_scrollbar = false;
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn get_text(hwnd: isize, index: i32, sub_item: i32) -> String {
        unsafe {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<ActionTableState>;
            if !ptr.is_null() {
                let state_mutex = &*ptr;
                if let Ok(g) = state_mutex.lock()
                    && let Some(row) = g.rows.get(index as usize)
                    && let Some(cell) = row.get(sub_item as usize)
                {
                    return match cell {
                        TableCell::Text(s) => s.clone(),
                        TableCell::SecondaryText(s) => s.clone(),
                        TableCell::Badge(b) => b.text.clone(),
                        TableCell::Actions(actions) => {
                            let labels: Vec<&str> = actions.iter().map(|a| a.label.as_str()).collect();
                            labels.join("   ")
                        }
                        _ => String::new(),
                    };
                }
            }
        }
        String::new()
    }

    pub fn set_column_title(&self, idx: i32, title: &str) {
        if let Ok(mut g) = self.state.lock()
            && let Some(col) = g.columns.get_mut(idx as usize)
        {
            col.title = title.to_string();
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn refresh_header(&self) {
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn apply_theme(&self, _is_dark: bool) {
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
            UpdateWindow(self.hwnd);
        }
    }

    pub fn set_on_select<F: Fn(i32) + Send + Sync + 'static>(&self, f: F) {
        if let Ok(mut g) = self.state.lock() {
            g.on_select = Some(Arc::new(f));
        }
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { IsWindowEnabled(self.hwnd) != 0 }
    }

    pub fn set_enabled(&self, enabled: bool) {
        unsafe {
            EnableWindow(self.hwnd, enabled as i32);
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd) != 0 }
    }

    pub fn set_visible(&self, visible: bool) {
        unsafe {
            ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }
}
