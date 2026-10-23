//! Internal Surface Types, Overlays, and Geometry Helpers

use crate::ui_framework::core::layout_engine::LayoutNode;
use crate::ui_framework::core::style::BgStyle;
use crate::ui_framework::win32::types::RECT;
use std::sync::Arc;

pub const SURFACE_CLASS_NAME: &str = "KoraDeclarativeSurface";
pub const WM_APP_FOCUS_NEXT: u32 = 0x8001;
pub const SUBCLASS_ID_INPUT: usize = 0x501;
pub const SELECT_ITEM_H: i32 = 28;

pub struct OpenSelectOverlay<M> {
    pub key: u32,
    pub rect_doc: RECT,
    pub selected: usize,
    pub options: Vec<String>,
    pub hover_idx: Option<usize>,
    pub on_select: Option<Arc<dyn Fn(usize) -> M + Send + Sync>>,
}

pub struct ActiveSliderDrag<M> {
    pub node_id: usize,
    pub rect_doc: RECT,
    pub min: i32,
    pub max: i32,
    pub last_val: i32,
    pub on_change: Option<Arc<dyn Fn(i32, bool) -> M + Send + Sync>>,
}

pub struct ActiveInput<M> {
    pub slot_key: u32,
    pub node_id: usize,
    pub hwnd: isize,
    pub rect: RECT,
    pub last_val: String,
    pub last_placeholder: String,
    pub is_disabled: bool,
    pub is_password: bool,
    pub on_change: Option<Arc<dyn Fn(String) -> M + Send + Sync>>,
}

pub struct SurfaceInner<M> {
    pub hwnd: isize,
    pub height: i32,
    pub content_w: i32,
    pub content_h: i32,
    pub scroll_y: i32,
    pub max_scroll: i32,
    pub is_dragging_scroll: bool,
    pub drag_start_y: i32,
    pub drag_start_scroll: i32,
    pub hover_node_id: Option<usize>,
    pub tracking_mouse: bool,
    pub layout_root: Option<LayoutNode<M>>,
    pub open_select: Option<OpenSelectOverlay<M>>,
    pub active_slider: Option<ActiveSliderDrag<M>>,
    pub inputs: Vec<ActiveInput<M>>,
    pub focused_node_id: Option<usize>,
    pub edit_bg_brush: isize,
    pub edit_bg_color: u32,
    pub on_msg: Option<Arc<dyn Fn(M) + Send + Sync>>,
    pub bg: BgStyle,
}

#[inline]
pub fn pt_in_rect(x: i32, y: i32, rc: &RECT) -> bool {
    x >= rc.left && x < rc.right && y >= rc.top && y < rc.bottom
}

#[inline]
pub fn offset_rect(rc: &RECT, dy: i32) -> RECT {
    RECT {
        left: rc.left,
        top: rc.top + dy,
        right: rc.right,
        bottom: rc.bottom + dy,
    }
}

pub fn compute_dropdown_menu_rect<M>(
    sel: &OpenSelectOverlay<M>,
    scroll_y: i32,
    viewport_h: i32,
) -> RECT {
    let trig = offset_rect(&sel.rect_doc, -scroll_y);
    let menu_h = (sel.options.len() as i32 * SELECT_ITEM_H + 4).clamp(SELECT_ITEM_H + 4, 240);
    let below_bottom = trig.bottom + 2 + menu_h;
    let (top, bottom) = if below_bottom > viewport_h && trig.top - 2 - menu_h >= 0 {
        (trig.top - 2 - menu_h, trig.top - 2)
    } else {
        (trig.bottom + 2, below_bottom)
    };
    RECT {
        left: trig.left,
        top,
        right: trig.right,
        bottom,
    }
}

pub fn slider_value_from_x(x: i32, rc: &RECT, min: i32, max: i32) -> i32 {
    let pad = 8;
    let track_l = rc.left + pad;
    let track_r = (rc.right - pad).max(track_l + 1);
    let ratio = ((x - track_l) as f32 / (track_r - track_l) as f32).clamp(0.0, 1.0);
    (min as f32 + ratio * (max - min) as f32).round() as i32
}

