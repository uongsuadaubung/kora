//! Modal Dialog for Key Macro Setup & Editing (Thiết Lập Phím Macro)
//!
//! Full native Win32 implementation matching the approved design in `mockup_macro.html`.
//! Supports:
//! - Physical key capture for trigger key
//! - Macro name / description
//! - Action sequence table with move-up, move-down, delete
//! - Inline step builder: Type dropdown (press, down, up, delay, text) + input box + [+ Thêm bước]
//! - Pipeline DSL live preview

use crate::engine::config::hotkey::vk_to_name;
use crate::engine::profile::{KeyMacroRule, MacroAction};
use crate::platform::win32::{
    get_captured_display_text, get_captured_hotkey, start_combo_capture, start_single_key_capture,
    stop_combo_capture, stop_single_key_capture, ENGINE_INSTANCE, WM_COMBO_CAPTURED,
    WM_SINGLE_KEY_CAPTURED,
};
use crate::ui::colors::ThemePalette;
use crate::ui::components::action_table::{
    ActionTable, BadgeConfig, BadgeVariant, ColumnStyle, TableAction, TableRow,
};
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::components::{ComboBox, TextBox};
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::tokens::id::*;
use crate::ui::win32::*;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveCaptureTarget {
    None,
    TriggerKey,
    StepKey,
}

struct KeyMacroDialogState {
    editing_trigger_vk: Option<u32>,
    trigger_vk: Option<u32>,
    trigger_display: String,
    name: String,
    actions: Vec<MacroAction>,
    active_capture: ActiveCaptureTarget,
    current_action_type: usize, // 0: press, 1: down, 2: up, 3: delay, 4: text
    step_key_str: String,
    hwnd_dlg: isize,
    hwnd_table: isize,
    txt_name: Option<TextBox>,
    table_actions: Option<Arc<ActionTable>>,
    cbo_action_type: Option<ComboBox>,
    btn_trigger: Option<PushButton>,
    btn_step_key: Option<PushButton>,
    txt_step_delay: Option<TextBox>,
    txt_step_text: Option<TextBox>,
    dsl_text: String,
    step_count_text: String,
}

static DIALOG_PARENT_HWND: AtomicIsize = AtomicIsize::new(0);
static DISPLAY_FONT: AtomicIsize = AtomicIsize::new(0);
static STATE: Mutex<Option<KeyMacroDialogState>> = Mutex::new(None);

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraKeyMacroDialogClass";

/// Displays the modal Key Macro Dialog for creating a new macro
pub fn show_key_macro_dialog(parent: isize) {
    show_key_macro_dialog_with_key(parent, None);
}

/// Displays the modal Key Macro Dialog for creating or editing a macro
pub fn show_key_macro_dialog_with_key(parent: isize, initial_trigger: Option<u32>) {
    let strings = crate::language::current();

    let mut name = String::new();
    let mut actions = Vec::new();
    let mut trigger_display = String::new();
    let trigger_vk = initial_trigger;

    if let Some(vk) = initial_trigger {
        trigger_display = vk_to_name(vk).to_string();
        if let Ok(guard) = ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            let pm = &engine.profile_manager;
            let sel_id = &pm.selected_id;
            let resolved = pm.resolve_key_macro_items_for_ui(sel_id);
            if let Some(item) = resolved.into_iter().find(|m| m.trigger_vk == vk) {
                name = item.name;
                actions = item.actions;
                trigger_display = item.trigger_name;
            }
        }
    }

    let dsl_text = MacroAction::actions_to_dsl(&actions);
    let step_count_text = format!("{} bước", actions.len());

    if let Ok(mut guard) = STATE.lock() {
        *guard = Some(KeyMacroDialogState {
            editing_trigger_vk: initial_trigger,
            trigger_vk,
            trigger_display,
            name,
            actions,
            active_capture: ActiveCaptureTarget::None,
            current_action_type: 0,
            step_key_str: String::new(),
            hwnd_dlg: 0,
            hwnd_table: 0,
            txt_name: None,
            table_actions: None,
            cbo_action_type: None,
            btn_trigger: None,
            btn_step_key: None,
            txt_step_delay: None,
            txt_step_text: None,
            dsl_text,
            step_count_text,
        });
    }

    DIALOG_PARENT_HWND.store(parent, Ordering::SeqCst);

    let title_str = if let Some(vk) = initial_trigger {
        format!("{} [{}]", strings.dlg_key_macro_title_edit, vk_to_name(vk))
    } else {
        strings.dlg_key_macro_title_add.to_string()
    };

    super::base::create_modal_dialog(
        parent,
        &title_str,
        600,
        475,
        CLASS_NAME,
        key_macro_dialog_wnd_proc,
        &REGISTERED,
    );
}

fn move_step(index: usize, delta: isize) {
    let (changed, actions_copy, table_opt, hwnd_dlg) = {
        if let Ok(mut guard) = STATE.lock()
            && let Some(ref mut s) = *guard
        {
            let new_index = index as isize + delta;
            if new_index >= 0 && (new_index as usize) < s.actions.len() {
                let item = s.actions.remove(index);
                s.actions.insert(new_index as usize, item);
                s.dsl_text = if s.actions.is_empty() {
                    "(Trống)".to_string()
                } else {
                    MacroAction::actions_to_dsl(&s.actions)
                };
                s.step_count_text = format!("{} bước", s.actions.len());
                (true, s.actions.clone(), s.table_actions.clone(), s.hwnd_dlg)
            } else {
                (false, Vec::new(), None, 0)
            }
        } else {
            (false, Vec::new(), None, 0)
        }
    };

    if changed {
        update_table_and_dialog(table_opt.as_deref(), hwnd_dlg, &actions_copy);
    }
}

fn remove_step(index: usize) {
    let (changed, actions_copy, table_opt, hwnd_dlg) = {
        if let Ok(mut guard) = STATE.lock()
            && let Some(ref mut s) = *guard
        {
            if index < s.actions.len() {
                s.actions.remove(index);
                s.dsl_text = if s.actions.is_empty() {
                    "(Trống)".to_string()
                } else {
                    MacroAction::actions_to_dsl(&s.actions)
                };
                s.step_count_text = format!("{} bước", s.actions.len());
                (true, s.actions.clone(), s.table_actions.clone(), s.hwnd_dlg)
            } else {
                (false, Vec::new(), None, 0)
            }
        } else {
            (false, Vec::new(), None, 0)
        }
    };

    if changed {
        update_table_and_dialog(table_opt.as_deref(), hwnd_dlg, &actions_copy);
    }
}

fn update_table_and_dialog(table: Option<&ActionTable>, hwnd_dlg: isize, actions: &[MacroAction]) {
    if let Some(t) = table {
        populate_action_table_rows(t, actions);
    }
    if hwnd_dlg != 0 {
        unsafe {
            InvalidateRect(hwnd_dlg, null_mut(), 0);
        }
    }
}

fn populate_action_table_rows(table: &ActionTable, actions: &[MacroAction]) {
    let count = actions.len();

    let rows: Vec<TableRow> = actions
        .iter()
        .enumerate()
        .map(|(i, act)| {
            let (type_name, badge_variant, val_str) = match act {
                MacroAction::Press(combo) => ("⚡ Nhấn", BadgeVariant::Info, combo.clone()),
                MacroAction::KeyDown(key) => ("⬇️ Giữ", BadgeVariant::Warning, key.clone()),
                MacroAction::KeyUp(key) => ("⬆️ Nhả", BadgeVariant::Success, key.clone()),
                MacroAction::Delay(ms) => ("⏱️ Chờ", BadgeVariant::Purple, format!("{} ms", ms)),
                MacroAction::Text(txt) => ("🔤 Gõ", BadgeVariant::Secondary, format!("\"{}\"", txt)),
            };

            let mut order_actions = Vec::new();
            if i > 0 {
                order_actions.push(TableAction::secondary("↑", move || {
                    move_step(i, -1);
                }));
            }
            if i + 1 < count {
                order_actions.push(TableAction::secondary("↓", move || {
                    move_step(i, 1);
                }));
            }

            let mut delete_actions = Vec::new();
            delete_actions.push(TableAction::delete("✕", move || {
                remove_step(i);
            }));

            let is_key_badge = matches!(
                act,
                MacroAction::Press(_) | MacroAction::KeyDown(_) | MacroAction::KeyUp(_)
            );

            let row = TableRow::new()
                .text((i + 1).to_string())
                .badge(BadgeConfig {
                    text: type_name.to_string(),
                    variant: badge_variant,
                    suffix: None,
                });

            let row = if is_key_badge {
                row.key(&val_str)
            } else {
                row.text(&val_str)
            };

            row.actions(order_actions).actions(delete_actions)
        })
        .collect();

    table.set_rows(rows);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe extern "system" fn key_macro_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_COMMAND: u32 = 0x0111;
    const WM_DRAWITEM: u32 = 0x002B;
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_CTLCOLOREDIT: u32 = 0x0133;
    const WM_CTLCOLORSTATIC: u32 = 0x0138;
    const WM_ACTIVATE: u32 = 0x0006;
    const WM_CLOSE: u32 = 0x0010;
    const CBN_SELCHANGE: usize = 1;

    unsafe {
        match msg {
            WM_CREATE => {
                let hfont_large = super::base::create_display_font();
                DISPLAY_FONT.store(hfont_large, Ordering::SeqCst);

                let hfont_normal = get_app_font_normal();
                let hfont_bold = get_app_font_bold();
                let strings = crate::language::current();

                // 1. Trigger Key Box (Button)
                let btn_trigger = PushButton::create_owner_draw(
                    hwnd,
                    IDC_KEY_MACRO_DLG_TRIGGER,
                    "",
                    16,
                    32,
                    150,
                    34,
                    hfont_large,
                );
                if let Some(ref btn) = btn_trigger {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let (is_listening, trig_vk, trig_disp) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| {
                                g.as_ref().map(|s| {
                                    (
                                        s.active_capture == ActiveCaptureTarget::TriggerKey,
                                        s.trigger_vk,
                                        s.trigger_display.clone(),
                                    )
                                })
                            })
                            .unwrap_or((false, None, String::new()));

                        let text = if is_listening {
                            "Bấm phím..."
                        } else if trig_vk.is_some() && !trig_disp.is_empty() {
                            &trig_disp
                        } else {
                            ""
                        };

                        let font = DISPLAY_FONT.load(Ordering::SeqCst);
                        super::base::draw_key_box(
                            dis,
                            is_dark,
                            is_listening,
                            text,
                            trig_vk.is_none(),
                            false,
                            font,
                        );
                        true
                    });
                }

                // 2. Name TextBox
                let initial_name = STATE
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|s| s.name.clone()))
                    .unwrap_or_default();
                let txt_name = TextBox::create(
                    hwnd,
                    IDC_KEY_MACRO_DLG_NAME,
                    &initial_name,
                    strings.dlg_key_macro_name_placeholder,
                    176,
                    32,
                    408,
                    34,
                    hfont_normal,
                );

                // 3. Action Table
                let table = ActionTable::create_styled(
                    hwnd,
                    IDC_KEY_MACRO_DLG_LIST_ACTIONS,
                    16,
                    96,
                    568,
                    160,
                    hfont_normal,
                    &[
                        (strings.dlg_key_macro_col_index, 36, ColumnStyle::Default),
                        (strings.dlg_key_macro_col_type, 110, ColumnStyle::Default),
                        (strings.dlg_key_macro_col_val, 274, ColumnStyle::Default),
                        (strings.dlg_key_macro_col_order, 84, ColumnStyle::Action),
                        (strings.dlg_key_macro_col_action, 64, ColumnStyle::Action),
                    ],
                );

                let arc_table = table.map(Arc::new);

                // 4. Action Type ComboBox
                let mut cbo_type = ComboBox::create(
                    hwnd,
                    IDC_KEY_MACRO_DLG_TYPE_COMBO,
                    24,
                    292,
                    145,
                    34,
                    hfont_normal,
                );
                if let Some(ref mut cbo) = cbo_type {
                    cbo.add_item("⚡ Nhấn (press)");
                    cbo.add_item("⬇️ Giữ (down)");
                    cbo.add_item("⬆️ Nhả (up)");
                    cbo.add_item("⏱️ Chờ (delay ms)");
                    cbo.add_item("🔤 Gõ chữ (text)");
                    cbo.set_selected(0);
                }

                // 5. Value input controls (Key Box, Delay Box, Text Box)
                let btn_step_key = PushButton::create_owner_draw(
                    hwnd,
                    IDC_KEY_MACRO_DLG_STEP_KEY,
                    "",
                    177,
                    292,
                    270,
                    34,
                    hfont_bold,
                );
                if let Some(ref btn) = btn_step_key {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let (is_listening, key_text) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| {
                                g.as_ref().map(|s| {
                                    (
                                        s.active_capture == ActiveCaptureTarget::StepKey,
                                        s.step_key_str.clone(),
                                    )
                                })
                            })
                            .unwrap_or((false, String::new()));

                        let text = if is_listening {
                            if key_text.is_empty() {
                                "Bấm phím / tổ hợp..."
                            } else {
                                &key_text
                            }
                        } else if key_text.is_empty() {
                            "Chọn phím"
                        } else {
                            &key_text
                        };

                        let font = get_app_font_bold();
                        super::base::draw_key_box(dis, is_dark, is_listening, text, key_text.is_empty(), false, font);
                        true
                    });
                }

                let txt_step_delay = TextBox::create(
                    hwnd,
                    IDC_KEY_MACRO_DLG_STEP_DELAY,
                    "150",
                    "mili-giây (ms)",
                    177,
                    292,
                    270,
                    34,
                    hfont_normal,
                );
                if let Some(ref t) = txt_step_delay {
                    ShowWindow(t.hwnd(), SW_HIDE);
                }

                let txt_step_text = TextBox::create(
                    hwnd,
                    IDC_KEY_MACRO_DLG_STEP_TEXT,
                    "",
                    "Nhập chuỗi văn bản cần gõ...",
                    177,
                    292,
                    270,
                    34,
                    hfont_normal,
                );
                if let Some(ref t) = txt_step_text {
                    ShowWindow(t.hwnd(), SW_HIDE);
                }

                // 6. Add Step Button
                let _btn_add_step = PushButton::create_with_variant(
                    hwnd,
                    IDC_KEY_MACRO_DLG_BTN_ADD_STEP,
                    strings.dlg_key_macro_btn_add_step,
                    455,
                    292,
                    120,
                    34,
                    hfont_bold,
                    ButtonVariant::Primary,
                    None,
                );

                // 7. Footer buttons
                let _btn_cancel = PushButton::create_with_variant(
                    hwnd,
                    IDC_KEY_MACRO_DLG_CANCEL,
                    strings.dlg_btn_cancel,
                    380,
                    422,
                    90,
                    32,
                    hfont_normal,
                    ButtonVariant::Secondary,
                    None,
                );
                let _btn_save = PushButton::create_with_variant(
                    hwnd,
                    IDC_KEY_MACRO_DLG_SAVE,
                    strings.btn_save,
                    480,
                    422,
                    104,
                    32,
                    hfont_bold,
                    ButtonVariant::Primary,
                    None,
                );

                // Save UI elements in state
                let mut initial_actions = Vec::new();
                if let Ok(mut guard) = STATE.lock()
                    && let Some(ref mut s) = *guard
                {
                    s.hwnd_dlg = hwnd;
                    s.txt_name = txt_name;
                    s.cbo_action_type = cbo_type;
                    s.btn_trigger = btn_trigger;
                    s.btn_step_key = btn_step_key;
                    s.txt_step_delay = txt_step_delay;
                    s.txt_step_text = txt_step_text;
                    if let Some(ref t) = arc_table {
                        s.hwnd_table = t.hwnd();
                        s.table_actions = Some(t.clone());
                    }
                    initial_actions = s.actions.clone();
                }

                if let Some(ref t) = arc_table {
                    populate_action_table_rows(t, &initial_actions);
                }

                0
            }

            WM_DRAWITEM => {
                super::forward_draw_item(wparam, lparam)
            }

            WM_CTLCOLORSTATIC => {
                let palette = ThemePalette::get(is_current_dark());
                let hdc = wparam as isize;
                SetTextColor(hdc, palette.text_primary);
                SetBkColor(hdc, palette.bg_window);
                crate::ui::theme::get_panel_bg_brush()
            }

            WM_CTLCOLOREDIT => {
                let palette = ThemePalette::get(is_current_dark());
                let hdc = wparam as isize;
                SetTextColor(hdc, palette.text_primary);
                SetBkColor(hdc, palette.bg_input);
                crate::ui::theme::get_input_bg_brush()
            }

            WM_ERASEBKGND => 1,

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                let is_dark = is_current_dark();
                let palette = ThemePalette::get(is_dark);
                let strings = crate::language::current();
                let hfont_normal = get_app_font_normal();
                let hfont_bold = get_app_font_bold();

                let mut client_rc = RECT::default();
                GetClientRect(hwnd, &mut client_rc);

                // Background
                let bg_brush = CreateSolidBrush(palette.bg_window);
                FillRect(hdc, &client_rc, bg_brush);
                DeleteObject(bg_brush);

                SetBkMode(hdc, TRANSPARENT);

                // Labels top
                SetTextColor(hdc, palette.text_secondary);
                let old_font = SelectObject(hdc, hfont_bold);

                let mut rc_trig_lbl = RECT {
                    left: 16,
                    top: 14,
                    right: 170,
                    bottom: 30,
                };
                let w_trig = to_wide(strings.dlg_key_macro_trigger_lbl);
                DrawTextW(hdc, w_trig.as_ptr(), -1, &mut rc_trig_lbl, 0);

                let mut rc_name_lbl = RECT {
                    left: 176,
                    top: 14,
                    right: 580,
                    bottom: 30,
                };
                let w_name = to_wide(strings.dlg_key_macro_name_lbl);
                DrawTextW(hdc, w_name.as_ptr(), -1, &mut rc_name_lbl, 0);

                // Action table header
                let mut rc_actions_lbl = RECT {
                    left: 16,
                    top: 76,
                    right: 400,
                    bottom: 94,
                };
                let w_actions = to_wide(strings.dlg_key_macro_actions_lbl);
                DrawTextW(hdc, w_actions.as_ptr(), -1, &mut rc_actions_lbl, 0);

                // Step count & DSL content
                let (count_str, dsl_content) = STATE
                    .lock()
                    .ok()
                    .and_then(|g| {
                        g.as_ref().map(|s| (s.step_count_text.clone(), s.dsl_text.clone()))
                    })
                    .unwrap_or_else(|| (String::new(), "(Trống)".to_string()));
                SelectObject(hdc, hfont_normal);
                let mut rc_count = RECT {
                    left: 450,
                    top: 76,
                    right: 584,
                    bottom: 94,
                };
                let w_count = to_wide(&count_str);
                DrawTextW(hdc, w_count.as_ptr(), -1, &mut rc_count, 0x0002); // DT_RIGHT

                // Inline builder label
                SelectObject(hdc, hfont_bold);
                let mut rc_insert_lbl = RECT {
                    left: 16,
                    top: 266,
                    right: 400,
                    bottom: 284,
                };
                let w_insert = to_wide(strings.dlg_key_macro_insert_title);
                DrawTextW(hdc, w_insert.as_ptr(), -1, &mut rc_insert_lbl, 0);

                // Builder container background
                let rc_builder = RECT {
                    left: 16,
                    top: 286,
                    right: 584,
                    bottom: 332,
                };
                let br_card = CreateSolidBrush(palette.bg_card);
                let pen_border = CreatePen(PS_SOLID, 1, palette.border_separator);
                let ob = SelectObject(hdc, br_card);
                let op = SelectObject(hdc, pen_border);
                RoundRect(hdc, rc_builder.left, rc_builder.top, rc_builder.right, rc_builder.bottom, 6, 6);
                SelectObject(hdc, ob);
                SelectObject(hdc, op);
                DeleteObject(br_card);
                DeleteObject(pen_border);

                // DSL Preview Container
                let rc_dsl = RECT {
                    left: 16,
                    top: 344,
                    right: 584,
                    bottom: 388,
                };
                let br_dsl = CreateSolidBrush(palette.bg_card);
                let pen_dsl = CreatePen(PS_SOLID, 1, palette.border);
                let ob = SelectObject(hdc, br_dsl);
                let op = SelectObject(hdc, pen_dsl);
                RoundRect(hdc, rc_dsl.left, rc_dsl.top, rc_dsl.right, rc_dsl.bottom, 6, 6);
                SelectObject(hdc, ob);
                SelectObject(hdc, op);
                DeleteObject(br_dsl);
                DeleteObject(pen_dsl);

                // DSL title
                SelectObject(hdc, hfont_bold);
                SetTextColor(hdc, palette.text_secondary);
                let mut rc_dsl_title = RECT {
                    left: 26,
                    top: 357,
                    right: 60,
                    bottom: 377,
                };
                let w_dsl_lbl = to_wide("DSL:");
                DrawTextW(hdc, w_dsl_lbl.as_ptr(), -1, &mut rc_dsl_title, 0);

                // DSL text content
                SelectObject(hdc, hfont_normal);
                SetTextColor(hdc, 0x00B0C94E); // Teal accent for DSL
                let mut rc_dsl_txt = RECT {
                    left: 64,
                    top: 357,
                    right: 574,
                    bottom: 377,
                };
                let w_dsl_txt = to_wide(&dsl_content);
                DrawTextW(hdc, w_dsl_txt.as_ptr(), -1, &mut rc_dsl_txt, 0x00004000); // DT_END_ELLIPSIS

                // Footer divider line
                let pen_div = CreatePen(PS_SOLID, 1, palette.border_separator);
                let op = SelectObject(hdc, pen_div);
                MoveToEx(hdc, 0, 406, null_mut());
                LineTo(hdc, client_rc.right, 406);
                SelectObject(hdc, op);
                DeleteObject(pen_div);

                // Footer Hint text
                SetTextColor(hdc, palette.text_secondary);
                let mut rc_hint = RECT {
                    left: 16,
                    top: 429,
                    right: 360,
                    bottom: 453,
                };
                let w_hint = to_wide(strings.dlg_key_macro_trigger_hint);
                DrawTextW(hdc, w_hint.as_ptr(), -1, &mut rc_hint, 0);

                SelectObject(hdc, old_font);
                EndPaint(hwnd, &ps);
                0
            }

            WM_COMMAND => {
                let ctrl_id = (wparam & 0xFFFF) as u32;
                let notify_code = (wparam >> 16) & 0xFFFF;

                if ctrl_id == IDC_KEY_MACRO_DLG_TYPE_COMBO && notify_code == CBN_SELCHANGE {
                    stop_single_key_capture();
                    stop_combo_capture();

                    let (sel_idx, btn_hwnd, delay_hwnd, text_hwnd) = {
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut s) = *guard
                        {
                            let idx = s.cbo_action_type.as_ref().and_then(|c| c.get_selected()).unwrap_or(0);
                            s.current_action_type = idx;
                            s.active_capture = ActiveCaptureTarget::None;
                            s.step_key_str.clear();
                            (
                                idx,
                                s.btn_step_key.as_ref().map(|b| b.hwnd()),
                                s.txt_step_delay.as_ref().map(|t| t.hwnd()),
                                s.txt_step_text.as_ref().map(|t| t.hwnd()),
                            )
                        } else {
                            (0, None, None, None)
                        }
                    };

                    let is_key = sel_idx <= 2;
                    let is_delay = sel_idx == 3;
                    let is_text = sel_idx == 4;

                    if let Some(h) = btn_hwnd {
                        ShowWindow(h, if is_key { SW_SHOW } else { SW_HIDE });
                        if is_key {
                            InvalidateRect(h, null_mut(), 0);
                            UpdateWindow(h);
                        }
                    }
                    if let Some(h) = delay_hwnd {
                        ShowWindow(h, if is_delay { SW_SHOW } else { SW_HIDE });
                    }
                    if let Some(h) = text_hwnd {
                        ShowWindow(h, if is_text { SW_SHOW } else { SW_HIDE });
                    }
                    return 0;
                }

                match ctrl_id {
                    IDC_KEY_MACRO_DLG_TRIGGER => {
                        let is_already = STATE
                            .lock()
                            .ok()
                            .and_then(|g| g.as_ref().map(|s| s.active_capture == ActiveCaptureTarget::TriggerKey))
                            .unwrap_or(false);

                        if is_already {
                            stop_single_key_capture();
                            if let Ok(mut guard) = STATE.lock()
                                && let Some(ref mut s) = *guard
                            {
                                s.active_capture = ActiveCaptureTarget::None;
                            }
                        } else {
                            stop_combo_capture();
                            stop_single_key_capture();
                            if let Ok(mut guard) = STATE.lock()
                                && let Some(ref mut s) = *guard
                            {
                                s.active_capture = ActiveCaptureTarget::TriggerKey;
                            }
                            start_single_key_capture(hwnd);
                        }
                        let btn = GetDlgItem(hwnd, IDC_KEY_MACRO_DLG_TRIGGER as i32);
                        if btn != 0 {
                            InvalidateRect(btn, null_mut(), 0);
                            UpdateWindow(btn);
                        }
                        InvalidateRect(hwnd, null_mut(), 0);
                    }

                    IDC_KEY_MACRO_DLG_STEP_KEY => {
                        let (is_already, act_type) = STATE
                            .lock()
                            .ok()
                            .and_then(|g| g.as_ref().map(|s| (s.active_capture == ActiveCaptureTarget::StepKey, s.current_action_type)))
                            .unwrap_or((false, 0));

                        if is_already {
                            stop_single_key_capture();
                            stop_combo_capture();
                            if let Ok(mut guard) = STATE.lock()
                                && let Some(ref mut s) = *guard
                            {
                                s.active_capture = ActiveCaptureTarget::None;
                            }
                        } else {
                            stop_single_key_capture();
                            stop_combo_capture();
                            if let Ok(mut guard) = STATE.lock()
                                && let Some(ref mut s) = *guard
                            {
                                s.active_capture = ActiveCaptureTarget::StepKey;
                            }
                            if act_type == 0 {
                                start_combo_capture(hwnd, true);
                            } else {
                                start_single_key_capture(hwnd);
                            }
                        }
                        let btn = GetDlgItem(hwnd, IDC_KEY_MACRO_DLG_STEP_KEY as i32);
                        if btn != 0 {
                            InvalidateRect(btn, null_mut(), 0);
                            UpdateWindow(btn);
                        }
                        InvalidateRect(hwnd, null_mut(), 0);
                    }

                    IDC_KEY_MACRO_DLG_BTN_ADD_STEP => {
                        stop_single_key_capture();
                        stop_combo_capture();

                        let mut action_opt = None;
                        if let Ok(guard) = STATE.lock()
                            && let Some(ref s) = *guard
                        {
                            match s.current_action_type {
                                0 => {
                                    let key = s.step_key_str.replace(" + ", "+").trim().to_string();
                                    if !key.is_empty() && !key.contains("Bấm phím") && !key.contains("Cần thêm") {
                                        action_opt = Some(MacroAction::Press(key));
                                    }
                                }
                                1 => {
                                    let key = s.step_key_str.trim().to_string();
                                    if !key.is_empty() && !key.contains("Bấm phím") {
                                        action_opt = Some(MacroAction::KeyDown(key));
                                    }
                                }
                                2 => {
                                    let key = s.step_key_str.trim().to_string();
                                    if !key.is_empty() && !key.contains("Bấm phím") {
                                        action_opt = Some(MacroAction::KeyUp(key));
                                    }
                                }
                                3 => {
                                    let ms = s
                                        .txt_step_delay
                                        .as_ref()
                                        .map(|t| t.get_text())
                                        .unwrap_or_default()
                                        .trim()
                                        .parse::<u32>()
                                        .unwrap_or(150);
                                    action_opt = Some(MacroAction::Delay(ms.clamp(10, 10000)));
                                }
                                4 => {
                                    let txt = s
                                        .txt_step_text
                                        .as_ref()
                                        .map(|t| t.get_text())
                                        .unwrap_or_default();
                                    if !txt.is_empty() {
                                        action_opt = Some(MacroAction::Text(txt));
                                    }
                                }
                                _ => {}
                            }
                        }

                        let (action_added, actions_copy, hwnd_dlg, table_opt, btn_step_key_hwnd, txt_step_text) = {
                            if let Ok(mut guard) = STATE.lock()
                                && let Some(ref mut s) = *guard
                            {
                                if let Some(act) = action_opt {
                                    s.actions.push(act);
                                    s.active_capture = ActiveCaptureTarget::None;
                                    s.step_key_str.clear();
                                    s.dsl_text = if s.actions.is_empty() {
                                        "(Trống)".to_string()
                                    } else {
                                        MacroAction::actions_to_dsl(&s.actions)
                                    };
                                    s.step_count_text = format!("{} bước", s.actions.len());
                                    (
                                        true,
                                        s.actions.clone(),
                                        s.hwnd_dlg,
                                        s.table_actions.clone(),
                                        s.btn_step_key.as_ref().map(|b| b.hwnd()),
                                        s.txt_step_text,
                                    )
                                } else {
                                    (false, Vec::new(), 0, None, None, None)
                                }
                            } else {
                                (false, Vec::new(), 0, None, None, None)
                            }
                        };

                        if action_added {
                            if let Some(h_btn) = btn_step_key_hwnd {
                                InvalidateRect(h_btn, null_mut(), 0);
                                UpdateWindow(h_btn);
                            }
                            if let Some(txt) = txt_step_text {
                                txt.set_text("");
                            }
                            update_table_and_dialog(table_opt.as_deref(), hwnd_dlg, &actions_copy);
                        }
                    }

                    IDC_KEY_MACRO_DLG_SAVE => {
                        stop_single_key_capture();
                        stop_combo_capture();

                        let (trigger_vk, name, actions, orig_vk) = {
                            if let Ok(mut guard) = STATE.lock() {
                                if let Some(ref mut s) = *guard {
                                    let mut n = s.txt_name.as_ref().map(|t| t.get_text()).unwrap_or_default();
                                    if n.trim().is_empty() {
                                        n = format!("Macro {}", s.trigger_display);
                                    }
                                    (s.trigger_vk, n, s.actions.clone(), s.editing_trigger_vk)
                                } else {
                                    (None, String::new(), Vec::new(), None)
                                }
                            } else {
                                (None, String::new(), Vec::new(), None)
                            }
                        };

                        let trig_vk = match trigger_vk {
                            Some(v) => v,
                            None => {
                                // No trigger key assigned
                                return 0;
                            }
                        };

                        if actions.is_empty() {
                            return 0;
                        }

                        let rule = KeyMacroRule::new(trig_vk, name.clone(), actions.clone());

                        crate::ui::controller::edit_selected_then(
                            |target| match target {
                                crate::ui::controller::EditTarget::Global(_) => {}
                                crate::ui::controller::EditTarget::Child(_) => {}
                            },
                            |engine| {
                                let pm = &mut engine.profile_manager;
                                let sel_id = pm.selected_id.clone();

                                if sel_id == crate::ui::GLOBAL_PROFILE_ID {
                                    if let Some(old_vk) = orig_vk {
                                        pm.global_profile.app_key_macros.retain(|m| m.trigger_vk != old_vk);
                                    }
                                    pm.global_profile.app_key_macros.retain(|m| m.trigger_vk != trig_vk);
                                    pm.global_profile.app_key_macros.push(rule);
                                } else {
                                    // If this trigger key exists in global macros, record as override
                                    let is_global = pm.global_profile.app_key_macros.iter().any(|m| m.trigger_vk == trig_vk);
                                    if let Some(child) = pm.get_profile_mut(&sel_id) {
                                        if is_global {
                                            child.muted_key_macros.retain(|&v| v != trig_vk);
                                            child.overridden_key_macros.insert(trig_vk, (name, actions));
                                        } else {
                                            if let Some(old_vk) = orig_vk {
                                                child.app_key_macros.retain(|m| m.trigger_vk != old_vk);
                                            }
                                            child.app_key_macros.retain(|m| m.trigger_vk != trig_vk);
                                            child.app_key_macros.push(rule);
                                        }
                                    }
                                }
                            },
                        );

                        let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                        let font = DISPLAY_FONT.swap(0, Ordering::SeqCst);
                        super::base::cleanup_modal_dialog(parent, font);
                        DestroyWindow(hwnd);
                        crate::ui::controller::refresh_key_macro_list();
                    }

                    IDC_KEY_MACRO_DLG_CANCEL | IDCANCEL => {
                        stop_single_key_capture();
                        stop_combo_capture();
                        let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                        let font = DISPLAY_FONT.swap(0, Ordering::SeqCst);
                        super::base::cleanup_modal_dialog(parent, font);
                        DestroyWindow(hwnd);
                    }

                    _ => {}
                }
                0
            }

            WM_SINGLE_KEY_CAPTURED => {
                let vk = wparam as u32;
                let active = STATE
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|s| s.active_capture))
                    .unwrap_or(ActiveCaptureTarget::None);

                match active {
                    ActiveCaptureTarget::TriggerKey => {
                        stop_single_key_capture();
                        let mut h_trig = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut s) = *guard
                        {
                            s.trigger_vk = Some(vk);
                            s.trigger_display = vk_to_name(vk).to_string();
                            s.active_capture = ActiveCaptureTarget::None;
                            if let Some(ref btn) = s.btn_trigger {
                                h_trig = btn.hwnd();
                            }
                        }
                        if h_trig == 0 {
                            h_trig = GetDlgItem(hwnd, IDC_KEY_MACRO_DLG_TRIGGER as i32);
                        }
                        if h_trig != 0 {
                            InvalidateRect(h_trig, null_mut(), 0);
                            UpdateWindow(h_trig);
                        }
                    }
                    ActiveCaptureTarget::StepKey => {
                        stop_single_key_capture();
                        let mut h_step = 0;
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut s) = *guard
                        {
                            s.step_key_str = vk_to_name(vk).to_string();
                            s.active_capture = ActiveCaptureTarget::None;
                            if let Some(ref btn) = s.btn_step_key {
                                h_step = btn.hwnd();
                            }
                        }
                        if h_step == 0 {
                            h_step = GetDlgItem(hwnd, IDC_KEY_MACRO_DLG_STEP_KEY as i32);
                        }
                        if h_step != 0 {
                            InvalidateRect(h_step, null_mut(), 0);
                            UpdateWindow(h_step);
                        }
                    }
                    ActiveCaptureTarget::None => {}
                }
                0
            }

            WM_COMBO_CAPTURED => {
                let active = STATE
                    .lock()
                    .ok()
                    .and_then(|g| g.as_ref().map(|s| s.active_capture))
                    .unwrap_or(ActiveCaptureTarget::None);

                if active == ActiveCaptureTarget::StepKey {
                    let display = get_captured_display_text();
                    let hotkey = get_captured_hotkey();
                    let is_final = hotkey.is_some();

                    let btn_hwnd = {
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut s) = *guard
                        {
                            if !display.is_empty() {
                                s.step_key_str = display;
                            }
                            if is_final {
                                s.active_capture = ActiveCaptureTarget::None;
                            }
                            s.btn_step_key.as_ref().map(|b| b.hwnd())
                        } else {
                            None
                        }
                    };

                    if is_final {
                        stop_combo_capture();
                    }

                    let h_step = btn_hwnd.unwrap_or_else(|| GetDlgItem(hwnd, IDC_KEY_MACRO_DLG_STEP_KEY as i32));
                    if h_step != 0 {
                        InvalidateRect(h_step, null_mut(), 0);
                        UpdateWindow(h_step);
                    }
                }
                0
            }

            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    stop_single_key_capture();
                    stop_combo_capture();
                    let (was_capturing, btn_step_key_hwnd, btn_trigger_hwnd) = {
                        if let Ok(mut guard) = STATE.lock()
                            && let Some(ref mut s) = *guard
                        {
                            let capturing = s.active_capture != ActiveCaptureTarget::None;
                            s.active_capture = ActiveCaptureTarget::None;
                            (
                                capturing,
                                s.btn_step_key.as_ref().map(|b| b.hwnd()),
                                s.btn_trigger.as_ref().map(|b| b.hwnd()),
                            )
                        } else {
                            (false, None, None)
                        }
                    };
                    if was_capturing {
                        if let Some(h) = btn_step_key_hwnd {
                            InvalidateRect(h, null_mut(), 0);
                            UpdateWindow(h);
                        }
                        if let Some(h) = btn_trigger_hwnd {
                            InvalidateRect(h, null_mut(), 0);
                            UpdateWindow(h);
                        }
                        InvalidateRect(hwnd, null_mut(), 0);
                    }
                }
                0
            }

            WM_CLOSE => {
                stop_single_key_capture();
                stop_combo_capture();
                let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                let font = DISPLAY_FONT.swap(0, Ordering::SeqCst);
                super::base::cleanup_modal_dialog(parent, font);
                DestroyWindow(hwnd);
                0
            }

            WM_DESTROY => {
                stop_single_key_capture();
                stop_combo_capture();
                if let Ok(mut guard) = STATE.lock() {
                    *guard = None;
                }
                0
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
