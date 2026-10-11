//! Modal Dialog for Adding / Editing Macro Shorthands (Gõ tắt)
//! Supports both "Add new shorthand" and "Edit existing shorthand"

use crate::engine::macro_table::MacroType;
use crate::ui::colors::ThemePalette;
use crate::ui::components::*;
use crate::ui::theme::is_current_dark;
use crate::ui::tokens::id::*;
use crate::ui::win32::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicIsize, Ordering};

#[derive(Debug, Clone)]
pub struct MacroEditData {
    pub original_key: String,
    pub key: String,
    pub value: String,
    pub macro_type: MacroType,
}

static DIALOG_PARENT_HWND: AtomicIsize = AtomicIsize::new(0);
static CURRENT_EDIT_DATA: Mutex<Option<MacroEditData>> = Mutex::new(None);

pub fn show_macro_dialog(parent: isize, edit_data: Option<MacroEditData>) {
    let class_name = to_wide("KoraMacroDialogClass");

    unsafe {
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfn_wnd_proc = Some(macro_dialog_wnd_proc);
        wc.h_instance = 0;
        wc.h_cursor = LoadCursorW(0, 32512);
        wc.lpsz_class_name = class_name.as_ptr();
        RegisterClassExW(&wc);

        DIALOG_PARENT_HWND.store(parent, Ordering::SeqCst);
        if let Ok(mut guard) = CURRENT_EDIT_DATA.lock() {
            *guard = edit_data.clone();
        }

        let width = 450;
        let height = 240;

        let mut rc: RECT = std::mem::zeroed();
        GetWindowRect(parent, &mut rc);
        let x = rc.left + ((rc.right - rc.left) - width) / 2;
        let y = rc.top + ((rc.bottom - rc.top) - height) / 2;

        EnableWindow(parent, 0);

        let strings = crate::language::current();
        let title = if edit_data.is_some() {
            strings.dlg_macro_title_edit
        } else {
            strings.dlg_macro_title_add
        };

        let hwnd = CreateWindowExW(
            0x00010000 | 0x00000001,
            class_name.as_ptr(),
            to_wide(title).as_ptr(),
            0x00080000 | 0x00020000 | 0x10000000 | 0x00C00000,
            x,
            y,
            width,
            height,
            parent,
            0,
            0,
            std::ptr::null_mut(),
        );

        if hwnd != 0 {
            apply_modern_window_styling(hwnd, is_current_dark());
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        } else {
            EnableWindow(parent, 1);
        }
    }
}

pub unsafe extern "system" fn macro_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_COMMAND: u32 = 0x0111;
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_CLOSE: u32 = 0x0010;
    const WM_DRAWITEM: u32 = 0x002B;
    const WM_CTLCOLORSTATIC: u32 = 0x0138;
    const WM_CTLCOLOREDIT: u32 = 0x0133;

    unsafe {
        match msg {
            WM_CREATE => {
                let font_bold = create_app_font(10, FW_SEMIBOLD);
                let font_normal = create_app_font(9, FW_NORMAL);
                let strings = crate::language::current();

                // 1. Shortcut Key
                Label::create(
                    hwnd,
                    strings.dlg_macro_lbl_key,
                    24,
                    16,
                    175,
                    18,
                    font_bold,
                );
                let txt_key = TextBox::create_with_placeholder(
                    hwnd,
                    IDC_MACRO_DLG_KEY,
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
                );
                let cmb_type = ComboBox::create_with_items(
                    hwnd,
                    IDC_MACRO_DLG_TYPE,
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
                Label::create(
                    hwnd,
                    strings.dlg_macro_lbl_val,
                    24,
                    76,
                    386,
                    18,
                    font_bold,
                );
                let txt_val = TextBox::create_with_placeholder(
                    hwnd,
                    IDC_MACRO_DLG_VAL,
                    strings.edit_macro_val_placeholder,
                    24,
                    98,
                    386,
                    28,
                    font_normal,
                );

                // Prefill if in Edit Mode
                if let Ok(guard) = CURRENT_EDIT_DATA.lock()
                    && let Some(ref data) = *guard
                {
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
                PushButton::create_with_variant(
                    hwnd,
                    IDC_MACRO_DLG_CANCEL,
                    strings.btn_cancel,
                    215,
                    144,
                    90,
                    30,
                    font_normal,
                    ButtonVariant::Secondary,
                    None,
                );
                PushButton::create_with_variant(
                    hwnd,
                    IDC_MACRO_DLG_SAVE,
                    strings.btn_save,
                    311,
                    144,
                    99,
                    30,
                    font_bold,
                    ButtonVariant::Primary,
                    None,
                );

                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc != 0 {
                    let is_dark = is_current_dark();
                    let palette = ThemePalette::get(is_dark);
                    let mut rc = RECT::default();
                    GetClientRect(hwnd, &mut rc);
                    let br = CreateSolidBrush(palette.bg_window);
                    FillRect(hdc, &rc, br);
                    DeleteObject(br);
                    EndPaint(hwnd, &ps);
                }
                0
            }
            WM_DRAWITEM => {
                let dis = &*(lparam as *const DRAWITEMSTRUCT);
                if dis.hwnd_item != 0 {
                    let res = SendMessageW(dis.hwnd_item, OCM_DRAWITEM, wparam, lparam);
                    if res != 0 {
                        return res;
                    }
                }
                0
            }
            WM_CTLCOLORSTATIC => {
                let is_dark = is_current_dark();
                let palette = ThemePalette::get(is_dark);
                let hdc = wparam as isize;
                SetTextColor(hdc, palette.text_primary);
                SetBkColor(hdc, palette.bg_window);
                let br = CreateSolidBrush(palette.bg_window);
                br
            }
            WM_CTLCOLOREDIT => {
                let is_dark = is_current_dark();
                let palette = ThemePalette::get(is_dark);
                let hdc = wparam as isize;
                SetTextColor(hdc, palette.text_primary);
                SetBkColor(hdc, palette.bg_input);
                let br = CreateSolidBrush(palette.bg_input);
                br
            }
            WM_COMMAND => {
                let ctl_id = (wparam & 0xFFFF) as u32;
                match ctl_id {
                    IDC_MACRO_DLG_CANCEL => {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    IDC_MACRO_DLG_SAVE => {
                        let mut key_buf = [0u16; 128];
                        let mut val_buf = [0u16; 1024];
                        let h_key = GetDlgItem(hwnd, IDC_MACRO_DLG_KEY as i32);
                        let h_val = GetDlgItem(hwnd, IDC_MACRO_DLG_VAL as i32);
                        let h_type = GetDlgItem(hwnd, IDC_MACRO_DLG_TYPE as i32);

                        GetWindowTextW(h_key, key_buf.as_mut_ptr(), 128);
                        GetWindowTextW(h_val, val_buf.as_mut_ptr(), 1024);

                        let key = String::from_utf16_lossy(
                            &key_buf[..key_buf.iter().position(|&c| c == 0).unwrap_or(0)],
                        )
                        .trim()
                        .to_string();

                        let val = String::from_utf16_lossy(
                            &val_buf[..val_buf.iter().position(|&c| c == 0).unwrap_or(0)],
                        )
                        .trim()
                        .to_string();

                        if !key.is_empty() && !val.is_empty() {
                            let type_idx = SendMessageW(h_type, 0x0147 /* CB_GETCURSEL */, 0, 0);
                            let mtype = match type_idx {
                                1 => MacroType::StartConsonant,
                                2 => MacroType::EndConsonant,
                                _ => MacroType::Normal,
                            };

                            let original_key = {
                                if let Ok(guard) = CURRENT_EDIT_DATA.lock()
                                    && let Some(ref data) = *guard
                                {
                                    Some(data.original_key.clone())
                                } else {
                                    None
                                }
                            };

                            if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                                && let Some(ref mut engine) = *guard
                            {
                                let sel_id = engine.profile_manager.selected_id.clone();
                                if sel_id == "global" {
                                    // If key was renamed in edit mode, remove the old key
                                    if let Some(ref orig) = original_key
                                        && orig != &key
                                    {
                                        engine.macro_table.remove(orig);
                                    }
                                    engine.macro_table.insert_typed(&key, &val, mtype);
                                } else if let Some(child) =
                                    engine.profile_manager.get_profile_mut(&sel_id)
                                {
                                    let is_global = engine.macro_table.get(&key).is_some();
                                    if let Some(ref orig) = original_key
                                        && orig != &key
                                    {
                                        child.app_macros.retain(|m| &m.key != orig);
                                        child.overridden_macros.remove(orig);
                                    }

                                    if is_global {
                                        child.muted_macros.retain(|k| k != &key);
                                        child
                                            .overridden_macros
                                            .insert(key.clone(), (val.clone(), mtype));
                                    } else {
                                        child.app_macros.retain(|m| m.key != key);
                                        child.app_macros.push(crate::engine::macro_table::MacroEntry {
                                            key: key.clone(),
                                            value: val.clone(),
                                            macro_type: mtype,
                                        });
                                    }
                                }

                                crate::engine::config_store::save_config_all_debounced(
                                    engine.config(),
                                    &engine.macro_table,
                                    &engine.profile_manager,
                                );
                            }

                            // Refresh ListView in parent UI
                            if let Ok(ui_guard) = crate::ui::UI_MANAGER.try_lock()
                                && let Some(ref ui) = *ui_guard
                            {
                                if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                                    && let Some(ref engine) = *guard
                                {
                                    let sel_id = engine.profile_manager.selected_id.clone();
                                    let items = engine
                                        .profile_manager
                                        .resolve_macro_items_for_ui(
                                            &sel_id,
                                            &engine.macro_table.get_sorted_entries(),
                                        );
                                    ui.controls.tab_macro.populate_macro_items(&items);
                                }
                            }
                        }

                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    _ => {}
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                    SetForegroundWindow(parent);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
