//! Modal Dialog for Adding / Editing Macro Shorthands (Gõ tắt)
//! Supports both "Add new shorthand" and "Edit existing shorthand"

use super::{
    attach_state, close_modal, detach_state, open_client_modal, reflect_dialog_messages, state,
};
use crate::engine::macro_table::MacroType;
use crate::ui::colors::ThemePalette;
use crate::ui::components::*;
use crate::ui::theme::is_current_dark;
use crate::ui::win32::*;

#[derive(Debug, Clone)]
pub struct MacroEditData {
    pub original_key: String,
    pub key: String,
    pub value: String,
    pub macro_type: MacroType,
}

struct MacroDialogState {
    parent: isize,
    edit_data: Option<MacroEditData>,
    txt_key: Option<TextBox>,
    cmb_type: Option<ComboBox>,
    txt_val: Option<TextBox>,
}

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraMacroDialogClass";

pub fn show_macro_dialog(parent: isize, edit_data: Option<MacroEditData>) {
    let strings = crate::language::current();
    let title = if edit_data.is_some() {
        strings.dlg_macro_title_edit
    } else {
        strings.dlg_macro_title_add
    };

    open_client_modal(
        parent,
        title,
        434,
        196,
        CLASS_NAME,
        macro_dialog_wnd_proc,
        &REGISTERED,
        Box::new(MacroDialogState {
            parent,
            edit_data,
            txt_key: None,
            cmb_type: None,
            txt_val: None,
        }),
    );
}

unsafe fn submit_macro(hwnd: isize) {
    let Some(st) = (unsafe { state::<MacroDialogState>(hwnd) }) else {
        return;
    };
    let key = st
        .txt_key
        .as_ref()
        .map(|t| t.get_text().trim().to_string())
        .unwrap_or_default();
    let val = st
        .txt_val
        .as_ref()
        .map(|t| t.get_text().trim().to_string())
        .unwrap_or_default();
    let type_idx = st
        .cmb_type
        .as_ref()
        .and_then(|c| c.get_selected())
        .unwrap_or(0);
    let original_key = st.edit_data.as_ref().map(|d| d.original_key.clone());
    let focus_hwnd = if key.is_empty() {
        st.txt_key.as_ref().map_or(0, |t| t.hwnd())
    } else {
        st.txt_val.as_ref().map_or(0, |t| t.hwnd())
    };

    if !key.is_empty() && !val.is_empty() {
        let mtype = match type_idx {
            1 => MacroType::StartConsonant,
            2 => MacroType::EndConsonant,
            _ => MacroType::Normal,
        };
        crate::ui::controller::save_macro(original_key.as_deref(), &key, &val, mtype);
        unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    } else if focus_hwnd != 0 {
        unsafe {
            SetFocus(focus_hwnd);
        }
    }
}

/// Standard Win32 Window Procedure for Macro Editor Dialog.
///
/// # Safety
/// Must be invoked by Windows message dispatcher with valid window handle and parameters.
pub unsafe extern "system" fn macro_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_NCDESTROY: u32 = 0x0082;
    unsafe {
        if let Some(res) = reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                attach_state::<MacroDialogState>(hwnd, lparam);
                let Some(st) = state::<MacroDialogState>(hwnd) else {
                    return -1;
                };
                let font_bold = crate::ui::theme::cached_font(10, FW_SEMIBOLD);
                let font_normal = crate::ui::theme::get_app_font_normal();
                let strings = crate::language::current();

                // 1. Shortcut Key
                Label::create(hwnd, strings.dlg_macro_lbl_key, 24, 16, 175, 18, font_bold)
                    .map(|l| l.on_surface(SurfaceRole::Window));
                let txt_key = TextBox::create_with_placeholder(
                    hwnd,
                    0,
                    strings.edit_macro_key_placeholder,
                    24,
                    38,
                    175,
                    28,
                    font_normal,
                );

                // 2. Macro Rule Type
                Label::create(
                    hwnd,
                    strings.dlg_macro_lbl_type,
                    215,
                    16,
                    195,
                    18,
                    font_bold,
                )
                .map(|l| l.on_surface(SurfaceRole::Window));
                let cmb_type = ComboBox::create_with_items(
                    hwnd,
                    0,
                    215,
                    38,
                    195,
                    120,
                    font_normal,
                    &[
                        strings.macro_type_normal,
                        strings.macro_type_start,
                        strings.macro_type_end,
                    ],
                    0,
                );

                // 3. Replacement Value
                Label::create(hwnd, strings.dlg_macro_lbl_val, 24, 76, 386, 18, font_bold)
                    .map(|l| l.on_surface(SurfaceRole::Window));
                let txt_val = TextBox::create_with_placeholder(
                    hwnd,
                    0,
                    strings.edit_macro_val_placeholder,
                    24,
                    98,
                    386,
                    28,
                    font_normal,
                );

                // Prefill if in Edit Mode
                if let Some(ref data) = st.edit_data {
                    if let Some(ref k) = txt_key {
                        k.set_text(&data.key);
                    }
                    if let Some(ref v) = txt_val {
                        v.set_text(&data.value);
                    }
                    if let Some(ref c) = cmb_type {
                        let idx = match data.macro_type {
                            MacroType::Normal => 0,
                            MacroType::StartConsonant => 1,
                            MacroType::EndConsonant => 2,
                        };
                        c.set_selected(idx);
                    }
                }

                // 4. Action Buttons
                super::create_dialog_footer_buttons(
                    hwnd,
                    (strings.btn_cancel, 215, 90),
                    (strings.btn_save, 311, 99, ButtonVariant::Primary),
                    144,
                    30,
                    None,
                    move || submit_macro(hwnd),
                );

                st.txt_key = txt_key;
                st.cmb_type = cmb_type;
                st.txt_val = txt_val;

                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let palette = ThemePalette::get(is_current_dark());
                    super::base::fill_dialog_bg(hwnd, hdc, &palette);
                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_COMMAND => {
                match (wparam & 0xFFFF) as u32 {
                    IDCANCEL => {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    IDOK => submit_macro(hwnd),
                    _ => {}
                }
                0
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let vk = wparam as u32;
                if vk == 0x1B {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    return 0;
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_CLOSE => {
                let parent = state::<MacroDialogState>(hwnd).map_or(0, |s| s.parent);
                close_modal(hwnd, parent);
                0
            }
            WM_NCDESTROY => {
                if let Some(st) = detach_state::<MacroDialogState>(hwnd) {
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
