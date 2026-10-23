//! Modern Segmented Pill TabBar Control
//! Backed by Declarative Single-HWND UI Engine (`Surface<usize>` + `core::tabs`).

use crate::ui_framework::core::*;
use crate::ui_framework::win32::{SendMessageW, WM_COMMAND, set_hwnds_visible};
use std::sync::{Arc, Mutex};

pub struct TabBarState {
    pub tabs: Vec<String>,
    pub cur_sel: usize,
    pub parent: isize,
    pub control_id: u32,
}

#[derive(Clone)]
pub struct TabBar {
    surface: Surface<usize>,
    state: Arc<Mutex<TabBarState>>,
}

impl TabBar {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        parent: isize,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        _hfont_normal: isize,
        _hfont_bold: isize,
        tabs_list: &[impl AsRef<str>],
    ) -> Option<Self> {
        let surface = Surface::create(parent, x, y, width, height)?;
        surface.set_content_width(width);

        let tabs_vec: Vec<String> = tabs_list.iter().map(|s| s.as_ref().to_string()).collect();
        let state = Arc::new(Mutex::new(TabBarState {
            tabs: tabs_vec.clone(),
            cur_sel: 0,
            parent,
            control_id: id,
        }));

        surface.render(tab_group(tabs_vec, 0, |idx| idx));

        let state_clone = Arc::clone(&state);
        let surface_hwnd = surface.hwnd();
        let surface_weak = surface.clone();
        surface.set_on_msg(move |new_sel| {
            let (parent_hwnd, ctrl_id, tabs_current, changed) = {
                let mut g = state_clone.lock().unwrap();
                let old = g.cur_sel;
                g.cur_sel = new_sel;
                (g.parent, g.control_id, g.tabs.clone(), old != new_sel)
            };

            if changed {
                surface_weak.render(tab_group(tabs_current, new_sel, |idx| idx));
                if parent_hwnd != 0 {
                    unsafe {
                        SendMessageW(
                            parent_hwnd,
                            WM_COMMAND,
                            (ctrl_id & 0xFFFF) as usize,
                            surface_hwnd,
                        );
                    }
                }
            }
        });

        Some(Self { surface, state })
    }

    #[inline]
    pub fn hwnd(&self) -> isize {
        self.surface.hwnd()
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.state.lock().unwrap().control_id
    }

    #[inline]
    pub fn is_changed(&self, ctrl_id: u32, _notify_code: u32) -> bool {
        ctrl_id == self.id()
    }

    #[inline]
    pub fn get_cur_sel(&self) -> usize {
        self.state.lock().unwrap().cur_sel
    }

    pub fn set_cur_sel(&self, idx: usize) {
        let tabs_current = {
            let mut g = self.state.lock().unwrap();
            g.cur_sel = idx;
            g.tabs.clone()
        };
        self.surface.render(tab_group(tabs_current, idx, |i| i));
    }

    pub fn set_tabs(&self, new_tabs: &[impl AsRef<str>]) {
        let (tabs_vec, cur_sel) = {
            let mut g = self.state.lock().unwrap();
            g.tabs = new_tabs.iter().map(|s| s.as_ref().to_string()).collect();
            if g.cur_sel >= g.tabs.len() && !g.tabs.is_empty() {
                g.cur_sel = g.tabs.len() - 1;
            }
            (g.tabs.clone(), g.cur_sel)
        };
        self.surface.render(tab_group(tabs_vec, cur_sel, |i| i));
    }

    #[inline]
    pub fn set_visible(&self, visible: bool) {
        set_hwnds_visible(&[self.surface.hwnd()], visible);
    }
}
