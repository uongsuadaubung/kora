//! Universal ActionTable Component (Web-style Canvas / Owner-Draw Architecture)

pub mod render;
pub mod types;
pub mod wnd_proc;

pub use render::*;
pub use types::*;
pub use wnd_proc::*;

use crate::ui_framework::theme::get_app_font_bold;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, Once};

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
        let id = crate::ui_framework::components::resolve_id(id);
        ensure_class_registered();

        let font_bold = get_app_font_bold();

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

    pub fn set_rows(&self, rows: impl IntoIterator<Item = TableRow>) {
        let cells: Vec<Vec<TableCell>> = rows.into_iter().map(|r| r.cells).collect();
        self.set_raw_cells(cells);
    }

    pub fn set_raw_cells(&self, rows: Vec<Vec<TableCell>>) {
        if let Ok(mut g) = self.state.lock() {
            g.rows = rows;
            let total_rows = g.rows.len();
            if let Some(r) = g.selected_row {
                g.selected_row = if total_rows == 0 {
                    None
                } else {
                    Some(r.min(total_rows - 1))
                };
            }
            if let Some(r) = g.hover_row {
                g.hover_row = if total_rows == 0 {
                    None
                } else {
                    Some(r.min(total_rows - 1))
                };
            }
            g.hover_action = ActionHover::None;
            let mut rc = RECT::default();
            unsafe {
                GetClientRect(self.hwnd, &mut rc);
            }
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
                            let labels: Vec<&str> =
                                actions.iter().map(|a| a.label.as_str()).collect();
                            labels.join("   ")
                        }
                        _ => String::new(),
                    };
                }
            }
        }
        String::new()
    }

    pub fn set_columns(&self, columns: &[(&str, i32, ColumnStyle)]) {
        if let Ok(mut g) = self.state.lock() {
            let mut defs = Vec::with_capacity(columns.len());
            for &(title, w, style) in columns {
                defs.push(ColumnDef::fixed(title, w, style));
            }
            g.columns = defs;
        }
        unsafe {
            InvalidateRect(self.hwnd, null_mut(), 0);
        }
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

    /// Updates all column header titles at once and refreshes the table header.
    pub fn set_column_titles(&self, titles: &[&str]) {
        Self::set_column_titles_by_hwnd(self.hwnd, titles);
    }

    /// Updates all column header titles on an `ActionTable` `hwnd` and invalidates the window.
    pub fn set_column_titles_by_hwnd(hwnd: isize, titles: &[&str]) {
        if hwnd == 0 {
            return;
        }
        unsafe {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Mutex<ActionTableState>;
            if !ptr.is_null()
                && let Ok(mut g) = (*ptr).lock()
            {
                for (col, &title) in g.columns.iter_mut().zip(titles.iter()) {
                    col.title = title.to_string();
                }
            }
            InvalidateRect(hwnd, null_mut(), 0);
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

