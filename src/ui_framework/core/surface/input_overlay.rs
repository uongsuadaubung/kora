//! Native Win32 EDIT Input Overlay and Synchronization
//!
//! Manages native single-line child `EDIT` controls positioned seamlessly inside
//! GDI Fluent Design rounded containers with zero flickering, Vietnamese IME typing,
//! clipboard operations, and Tab / Shift+Tab keyboard focus cycling.

use super::types::*;
use crate::ui_framework::core::element::*;
use crate::ui_framework::core::layout_engine::*;
use crate::ui_framework::win32::*;
use std::ptr::null_mut;
use std::sync::Arc;

pub(crate) unsafe extern "system" fn input_subclass_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
    uid_subclass: usize,
    _ref_data: usize,
) -> isize {
    match msg {
        WM_KEYDOWN => {
            if wparam == 0x09
            /* VK_TAB */
            {
                let shift_down = unsafe {
                    (GetKeyState(0x10 /* VK_SHIFT */) as i16) < 0
                };
                let parent = unsafe { GetParent(hwnd) };
                if parent != 0 {
                    unsafe {
                        PostMessageW(
                            parent,
                            WM_APP_FOCUS_NEXT,
                            if shift_down { 1 } else { 0 },
                            hwnd,
                        );
                    }
                }
                return 0;
            }
            if wparam == 0x0D
            /* VK_RETURN */
            {
                let parent = unsafe { GetParent(hwnd) };
                if parent != 0 {
                    let modal = unsafe { GetParent(parent) };
                    if modal != 0 {
                        unsafe {
                            PostMessageW(modal, WM_KEYDOWN, 0x0D, 0);
                        }
                    }
                }
                return 0;
            }
            if wparam == 0x1B
            /* VK_ESCAPE */
            {
                let parent = unsafe { GetParent(hwnd) };
                if parent != 0 {
                    let modal = unsafe { GetParent(parent) };
                    if modal != 0 {
                        unsafe {
                            PostMessageW(modal, WM_KEYDOWN, 0x1B, 0);
                        }
                    }
                }
                return 0;
            }
        }
        WM_CHAR => {
            if wparam == 0x0D || wparam == 0x09 || wparam == 0x1B {
                return 0;
            }
        }
        WM_NCDESTROY => unsafe {
            RemoveWindowSubclass(hwnd, input_subclass_proc, uid_subclass);
        },
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub(crate) struct CollectedInput<M> {
    pub slot_key: u32,
    pub node_id: usize,
    pub rect: RECT,
    pub z_index: i32,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub password: bool,
    pub auto_focus: bool,
    pub on_change: Option<Arc<dyn Fn(String) -> M + Send + Sync>>,
}

pub(crate) fn collect_inputs<M: Clone>(
    node: &LayoutNode<M>,
    parent_z: i32,
    slot_counter: &mut u32,
    out: &mut Vec<CollectedInput<M>>,
) {
    let my_z = if node.element.style.z_index != 0 {
        node.element.style.z_index
    } else {
        parent_z
    };
    if let WidgetKind::Input {
        key,
        value,
        placeholder,
        disabled,
        password,
        auto_focus,
        on_change,
    } = &node.element.kind
    {
        let slot = if *key != 0 { *key } else { *slot_counter };
        *slot_counter += 1;
        out.push(CollectedInput {
            slot_key: slot,
            node_id: node.id,
            rect: node.rect,
            z_index: my_z,
            value: value.clone(),
            placeholder: placeholder.clone(),
            disabled: *disabled,
            password: *password,
            auto_focus: *auto_focus,
            on_change: on_change.clone(),
        });
    }
    for child in &node.children {
        collect_inputs(child, my_z, slot_counter, out);
    }
}

pub fn is_node_occluding_rect<M>(
    node: &LayoutNode<M>,
    target_rc: &RECT,
    target_z: i32,
    scroll_y: i32,
    parent_z: i32,
) -> bool {
    let my_z = if node.element.style.z_index != 0 {
        node.element.style.z_index
    } else {
        parent_z
    };

    if my_z > target_z {
        let node_vp = offset_rect(&node.rect, -scroll_y);
        if rect_intersects(target_rc, &node_vp) {
            return true;
        }
    }

    for child in &node.children {
        if is_node_occluding_rect(child, target_rc, target_z, scroll_y, my_z) {
            return true;
        }
    }

    false
}

impl<M: Clone + Send + Sync + 'static> SurfaceInner<M> {
    pub(crate) fn sync_inputs(&mut self) -> Option<isize> {
        if self.hwnd == 0 {
            return None;
        }

        let mut collected = Vec::new();
        if let Some(ref root) = self.layout_root {
            let mut slot_counter = 1u32;
            collect_inputs(root, 0, &mut slot_counter, &mut collected);
        }

        let font_norm = resolve_font_handle(FontScale::Normal);
        let mut active_slots = Vec::new();
        let mut focus_target: Option<isize> = None;
        let menu_rc = self.open_select.as_ref().map(|sel| {
            compute_dropdown_menu_rect(sel, self.scroll_y, self.height)
        });

        for item in collected {
            active_slots.push(item.slot_key);

            let inner_h = 20;
            let box_h = item.rect.bottom - item.rect.top;
            let inner_y = item.rect.top + (box_h - inner_h) / 2 - self.scroll_y;
            let inner_x = item.rect.left + 10;
            let inner_w = (item.rect.right - item.rect.left - 20).max(10);
            let in_viewport = inner_y + inner_h > 0 && inner_y < self.height;

            let container_rc = RECT {
                left: item.rect.left,
                top: item.rect.top - self.scroll_y,
                right: item.rect.right,
                bottom: item.rect.bottom - self.scroll_y,
            };

            let is_occluded_by_node = self.layout_root.as_ref().map_or(false, |root| {
                is_node_occluding_rect(root, &container_rc, item.z_index, self.scroll_y, 0)
            });
            let is_occluded_by_select = if let Some(ref m_rc) = menu_rc {
                rect_intersects(&container_rc, m_rc)
            } else {
                false
            };

            let is_visible = in_viewport && !is_occluded_by_node && !is_occluded_by_select;

            if let Some(existing) = self
                .inputs
                .iter_mut()
                .find(|inp| inp.slot_key == item.slot_key)
            {
                existing.node_id = item.node_id;
                existing.rect = item.rect;
                existing.z_index = item.z_index;
                existing.on_change = item.on_change;

                unsafe {
                    SetWindowPos(
                        existing.hwnd,
                        0,
                        inner_x,
                        inner_y,
                        inner_w,
                        inner_h,
                        SWP_NOZORDER | SWP_NOACTIVATE | if is_visible { SWP_SHOWWINDOW } else { 0 },
                    );
                    if is_visible {
                        ShowWindow(existing.hwnd, SW_SHOW);
                        InvalidateRect(existing.hwnd, null_mut(), 1);
                    } else {
                        ShowWindow(existing.hwnd, SW_HIDE);
                    }
                }

                // Check text
                let cur_len = unsafe { GetWindowTextLengthW(existing.hwnd) };
                let mut buf = vec![0u16; (cur_len + 1) as usize];
                unsafe { GetWindowTextW(existing.hwnd, buf.as_mut_ptr(), buf.len() as i32) };
                let cur_text = String::from_utf16_lossy(&buf[..cur_len as usize]);
                if cur_text != item.value {
                    unsafe {
                        SetWindowTextW(existing.hwnd, to_wide(&item.value).as_ptr());
                    }
                    existing.last_val = item.value;
                }

                // Check placeholder
                if existing.last_placeholder != item.placeholder {
                    const EM_SETCUEBANNER: u32 = 0x1501;
                    unsafe {
                        SendMessageW(
                            existing.hwnd,
                            EM_SETCUEBANNER,
                            1,
                            to_wide(&item.placeholder).as_ptr() as isize,
                        );
                    }
                    existing.last_placeholder = item.placeholder;
                }

                // Check disabled
                if existing.is_disabled != item.disabled {
                    unsafe {
                        EnableWindow(existing.hwnd, if item.disabled { 0 } else { 1 });
                    }
                    existing.is_disabled = item.disabled;
                }

                // Check password mode toggle
                if existing.is_password != item.password {
                    const EM_SETPASSWORDCHAR: u32 = 0x00CC;
                    let ch = if item.password { 0x25CF as usize } else { 0 };
                    unsafe {
                        SendMessageW(existing.hwnd, EM_SETPASSWORDCHAR, ch, 0);
                        InvalidateRect(existing.hwnd, null_mut(), 1);
                    }
                    existing.is_password = item.password;
                }
            } else {
                // Create new Win32 EDIT control
                const WS_CHILD: u32 = 0x40000000;
                const WS_TABSTOP: u32 = 0x00010000;
                const ES_AUTOHSCROLL: u32 = 0x0080;
                const ES_PASSWORD: u32 = 0x0020;
                const WM_SETFONT: u32 = 0x0030;
                const EM_SETCUEBANNER: u32 = 0x1501;

                let mut style = WS_CHILD | WS_TABSTOP | ES_AUTOHSCROLL;
                if item.password {
                    style |= ES_PASSWORD;
                }

                let edit_hwnd = unsafe {
                    CreateWindowExW(
                        0,
                        to_wide("EDIT").as_ptr(),
                        to_wide(&item.value).as_ptr(),
                        style,
                        inner_x,
                        inner_y,
                        inner_w,
                        inner_h,
                        self.hwnd,
                        (1000 + item.slot_key) as isize,
                        GetModuleHandleW(null_mut()),
                        null_mut(),
                    )
                };

                if edit_hwnd != 0 {
                    unsafe {
                        if font_norm != 0 {
                            SendMessageW(edit_hwnd, WM_SETFONT, font_norm as usize, 0);
                        }
                        if !item.placeholder.is_empty() {
                            SendMessageW(
                                edit_hwnd,
                                EM_SETCUEBANNER,
                                1,
                                to_wide(&item.placeholder).as_ptr() as isize,
                            );
                        }
                        if item.disabled {
                            EnableWindow(edit_hwnd, 0);
                        }
                        SetWindowSubclass(edit_hwnd, input_subclass_proc, SUBCLASS_ID_INPUT, 0);
                        if is_visible {
                            ShowWindow(edit_hwnd, SW_SHOW);
                        }
                        if item.auto_focus {
                            focus_target = Some(edit_hwnd);
                            self.focused_node_id = Some(item.node_id);
                        }
                    }

                    self.inputs.push(ActiveInput {
                        slot_key: item.slot_key,
                        node_id: item.node_id,
                        hwnd: edit_hwnd,
                        rect: item.rect,
                        z_index: item.z_index,
                        last_val: item.value,
                        last_placeholder: item.placeholder,
                        is_disabled: item.disabled,
                        is_password: item.password,
                        on_change: item.on_change,
                    });
                }
            }
        }

        // Clean up removed inputs
        self.inputs.retain(|inp| {
            if !active_slots.contains(&inp.slot_key) {
                unsafe {
                    DestroyWindow(inp.hwnd);
                }
                false
            } else {
                true
            }
        });

        focus_target
    }

    pub(crate) fn update_input_scroll(&self) {
        let menu_rc = self.open_select.as_ref().map(|sel| {
            compute_dropdown_menu_rect(sel, self.scroll_y, self.height)
        });

        for inp in &self.inputs {
            let inner_h = 20;
            let box_h = inp.rect.bottom - inp.rect.top;
            let inner_y = inp.rect.top + (box_h - inner_h) / 2 - self.scroll_y;
            let inner_x = inp.rect.left + 10;
            let in_viewport = inner_y + inner_h > 0 && inner_y < self.height;

            let container_rc = RECT {
                left: inp.rect.left,
                top: inp.rect.top - self.scroll_y,
                right: inp.rect.right,
                bottom: inp.rect.bottom - self.scroll_y,
            };

            let is_occluded_by_node = self.layout_root.as_ref().map_or(false, |root| {
                is_node_occluding_rect(root, &container_rc, inp.z_index, self.scroll_y, 0)
            });
            let is_occluded_by_select = if let Some(ref m_rc) = menu_rc {
                rect_intersects(&container_rc, m_rc)
            } else {
                false
            };

            let is_visible = in_viewport && !is_occluded_by_node && !is_occluded_by_select;

            unsafe {
                if is_visible {
                    SetWindowPos(
                        inp.hwnd,
                        0,
                        inner_x,
                        inner_y,
                        0,
                        0,
                        SWP_NOSIZE
                            | SWP_NOZORDER
                            | SWP_NOACTIVATE
                            | SWP_SHOWWINDOW,
                    );
                    ShowWindow(inp.hwnd, SW_SHOW);
                    InvalidateRect(inp.hwnd, null_mut(), 1);
                } else {
                    ShowWindow(inp.hwnd, SW_HIDE);
                }
            }
        }
    }
}
