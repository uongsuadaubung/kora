//! Modal Dialog for Key Macro Setup & Editing (Thiết Lập Phím Macro)
//!
//! Full native Win32 implementation matching the approved design in `mockup_macro.html`.
//! Supports:
//! - Physical key capture for trigger key
//! - Macro name / description
//! - Action sequence table with move-up, move-down, delete
//! - Inline step builder: Type dropdown (press, down, up, delay, text) + input box + [+ Thêm bước]
//! - Pipeline DSL live preview

use super::{
    attach_state, close_modal, detach_state, open_client_modal, reflect_dialog_messages, state,
};
use crate::engine::config::hotkey::vk_to_name;
use crate::engine::profile::MacroAction;
use crate::platform::win32::{
    ENGINE_INSTANCE, WM_COMBO_CAPTURED, WM_SINGLE_KEY_CAPTURED, get_captured_display_text,
    get_captured_hotkey, start_combo_capture, start_single_key_capture, stop_combo_capture,
    stop_single_key_capture,
};
use crate::ui::colors::ThemePalette;
use crate::ui::components::action_table::{
    ActionTable, BadgeConfig, BadgeVariant, ColumnStyle, TableAction, TableRow,
};
use crate::ui::components::button::{ButtonVariant, PushButton};
use crate::ui::components::{ComboBox, Label, LabelTone, SurfaceRole, TextBox};
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::win32::*;
use std::ptr::null_mut;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveCaptureTarget {
    None,
    TriggerKey,
    StepKey,
}

struct KeyMacroDialogState {
    parent: isize,
    display_font: isize,
    editing_trigger_vk: Option<u32>,
    trigger_vk: Option<u32>,
    trigger_display: String,
    name: String,
    actions: Vec<MacroAction>,
    active_capture: ActiveCaptureTarget,
    current_action_type: usize, // 0: press, 1: down, 2: up, 3: delay, 4: text
    step_key_str: String,
    txt_name: Option<TextBox>,
    table_actions: Option<Arc<ActionTable>>,
    cbo_action_type: Option<ComboBox>,
    btn_trigger: Option<PushButton>,
    btn_step_key: Option<PushButton>,
    txt_step_delay: Option<TextBox>,
    txt_step_text: Option<TextBox>,
    lbl_step_count: Option<Label>,
    dsl_text: String,
}

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

    let title_str = if let Some(vk) = initial_trigger {
        format!("{} [{}]", strings.dlg_key_macro_title_edit, vk_to_name(vk))
    } else {
        strings.dlg_key_macro_title_add.to_string()
    };

    open_client_modal(
        parent,
        &title_str,
        600,
        475,
        CLASS_NAME,
        key_macro_dialog_wnd_proc,
        &REGISTERED,
        Box::new(KeyMacroDialogState {
            parent,
            display_font: 0,
            editing_trigger_vk: initial_trigger,
            trigger_vk,
            trigger_display,
            name,
            actions,
            active_capture: ActiveCaptureTarget::None,
            current_action_type: 0,
            step_key_str: String::new(),
            txt_name: None,
            table_actions: None,
            cbo_action_type: None,
            btn_trigger: None,
            btn_step_key: None,
            txt_step_delay: None,
            txt_step_text: None,
            lbl_step_count: None,
            dsl_text,
        }),
    );
}

fn move_step(hwnd: isize, index: usize, delta: isize) {
    let (changed, actions_copy, table_opt) = {
        if let Some(s) = unsafe { state::<KeyMacroDialogState>(hwnd) } {
            let new_index = index as isize + delta;
            if new_index >= 0 && (new_index as usize) < s.actions.len() {
                let item = s.actions.remove(index);
                s.actions.insert(new_index as usize, item);
                s.dsl_text = if s.actions.is_empty() {
                    "(Trống)".to_string()
                } else {
                    MacroAction::actions_to_dsl(&s.actions)
                };
                if let Some(ref lbl) = s.lbl_step_count {
                    lbl.set_text(&format!("{} bước", s.actions.len()));
                }
                (true, s.actions.clone(), s.table_actions.clone())
            } else {
                (false, Vec::new(), None)
            }
        } else {
            (false, Vec::new(), None)
        }
    };

    if changed {
        update_table_and_dialog(table_opt.as_deref(), hwnd, &actions_copy);
    }
}

fn remove_step(hwnd: isize, index: usize) {
    let (changed, actions_copy, table_opt) = {
        if let Some(s) = unsafe { state::<KeyMacroDialogState>(hwnd) } {
            if index < s.actions.len() {
                s.actions.remove(index);
                s.dsl_text = if s.actions.is_empty() {
                    "(Trống)".to_string()
                } else {
                    MacroAction::actions_to_dsl(&s.actions)
                };
                if let Some(ref lbl) = s.lbl_step_count {
                    lbl.set_text(&format!("{} bước", s.actions.len()));
                }
                (true, s.actions.clone(), s.table_actions.clone())
            } else {
                (false, Vec::new(), None)
            }
        } else {
            (false, Vec::new(), None)
        }
    };

    if changed {
        update_table_and_dialog(table_opt.as_deref(), hwnd, &actions_copy);
    }
}

fn update_table_and_dialog(table: Option<&ActionTable>, hwnd_dlg: isize, actions: &[MacroAction]) {
    if let Some(t) = table {
        populate_action_table_rows(hwnd_dlg, t, actions);
    }
    if hwnd_dlg != 0 {
        unsafe {
            InvalidateRect(hwnd_dlg, null_mut(), 0);
        }
    }
}

fn populate_action_table_rows(hwnd_dlg: isize, table: &ActionTable, actions: &[MacroAction]) {
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
                MacroAction::Text(txt) => {
                    ("🔤 Gõ", BadgeVariant::Secondary, format!("\"{}\"", txt))
                }
            };

            let mut order_actions = Vec::new();
            if i > 0 {
                order_actions.push(TableAction::secondary("↑", move || {
                    move_step(hwnd_dlg, i, -1);
                }));
            }
            if i + 1 < count {
                order_actions.push(TableAction::secondary("↓", move || {
                    move_step(hwnd_dlg, i, 1);
                }));
            }

            let mut delete_actions = Vec::new();
            delete_actions.push(TableAction::delete("✕", move || {
                remove_step(hwnd_dlg, i);
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
    const WM_COMMAND: u32 = 0x0111;
    const WM_PAINT: u32 = 0x000F;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_ACTIVATE: u32 = 0x0006;
    const WM_CLOSE: u32 = 0x0010;
    const WM_NCDESTROY: u32 = 0x0082;

    unsafe {
        if let Some(res) = reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                attach_state::<KeyMacroDialogState>(hwnd, lparam);
                let Some(st) = state::<KeyMacroDialogState>(hwnd) else {
                    return -1;
                };
                let hfont_large = super::base::create_display_font();
                st.display_font = hfont_large;

                let hfont_normal = get_app_font_normal();
                let hfont_bold = get_app_font_bold();
                let strings = crate::language::current();

                Label::create(
                    hwnd,
                    strings.dlg_key_macro_trigger_lbl,
                    16,
                    14,
                    154,
                    16,
                    hfont_bold,
                )
                .map(|l| {
                    l.with_tone(LabelTone::Secondary)
                        .on_surface(SurfaceRole::Window)
                });
                Label::create(
                    hwnd,
                    strings.dlg_key_macro_name_lbl,
                    176,
                    14,
                    404,
                    16,
                    hfont_bold,
                )
                .map(|l| {
                    l.with_tone(LabelTone::Secondary)
                        .on_surface(SurfaceRole::Window)
                });

                // 1. Trigger Key Box (Button)
                let btn_trigger =
                    PushButton::create_owner_draw(hwnd, 0, "", 16, 32, 150, 34, hfont_large);
                if let Some(ref btn) = btn_trigger {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let (is_listening, trig_vk, trig_disp, font) =
                            state::<KeyMacroDialogState>(hwnd)
                                .map(|s| {
                                    (
                                        s.active_capture == ActiveCaptureTarget::TriggerKey,
                                        s.trigger_vk,
                                        s.trigger_display.clone(),
                                        s.display_font,
                                    )
                                })
                                .unwrap_or((false, None, String::new(), 0));

                        let text = if is_listening {
                            "Bấm phím..."
                        } else if trig_vk.is_some() && !trig_disp.is_empty() {
                            &trig_disp
                        } else {
                            ""
                        };

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
                    btn.set_on_click(move || toggle_trigger_capture(hwnd));
                }

                // 2. Name TextBox
                let initial_name = st.name.clone();
                let txt_name = TextBox::create(
                    hwnd,
                    0,
                    &initial_name,
                    strings.dlg_key_macro_name_placeholder,
                    176,
                    32,
                    408,
                    34,
                    hfont_normal,
                );

                // 3. Action Table header & step count
                Label::create(
                    hwnd,
                    strings.dlg_key_macro_actions_lbl,
                    16,
                    76,
                    384,
                    18,
                    hfont_bold,
                )
                .map(|l| {
                    l.with_tone(LabelTone::Secondary)
                        .on_surface(SurfaceRole::Window)
                });
                let lbl_step_count = Label::create_right(
                    hwnd,
                    &format!("{} bước", st.actions.len()),
                    450,
                    76,
                    134,
                    18,
                    hfont_normal,
                )
                .map(|l| {
                    l.with_tone(LabelTone::Secondary)
                        .on_surface(SurfaceRole::Window)
                });

                let table = ActionTable::create_styled(
                    hwnd,
                    0,
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

                // 4. Inline builder title & Action Type ComboBox
                Label::create(
                    hwnd,
                    strings.dlg_key_macro_insert_title,
                    16,
                    266,
                    384,
                    18,
                    hfont_bold,
                )
                .map(|l| {
                    l.with_tone(LabelTone::Secondary)
                        .on_surface(SurfaceRole::Window)
                });

                let mut cbo_type = ComboBox::create(hwnd, 0, 24, 292, 145, 34, hfont_normal);
                if let Some(ref mut cbo) = cbo_type {
                    cbo.add_item("⚡ Nhấn (press)");
                    cbo.add_item("⬇️ Giữ (down)");
                    cbo.add_item("⬆️ Nhả (up)");
                    cbo.add_item("⏱️ Chờ (delay ms)");
                    cbo.add_item("🔤 Gõ chữ (text)");
                    cbo.set_selected(0);
                }

                // 5. Value input controls (Key Box, Delay Box, Text Box)
                let btn_step_key =
                    PushButton::create_owner_draw(hwnd, 0, "", 177, 292, 270, 34, hfont_bold);
                if let Some(ref btn) = btn_step_key {
                    btn.set_custom_drawer(move |dis, is_dark| {
                        let (is_listening, key_text) = state::<KeyMacroDialogState>(hwnd)
                            .map(|s| {
                                (
                                    s.active_capture == ActiveCaptureTarget::StepKey,
                                    s.step_key_str.clone(),
                                )
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
                        super::base::draw_key_box(
                            dis,
                            is_dark,
                            is_listening,
                            text,
                            key_text.is_empty(),
                            false,
                            font,
                        );
                        true
                    });
                    btn.set_on_click(move || toggle_step_key_capture(hwnd));
                }

                let txt_step_delay = TextBox::create(
                    hwnd,
                    0,
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
                    0,
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
                if let Some(btn) = PushButton::create_with_variant(
                    hwnd,
                    0,
                    strings.dlg_key_macro_btn_add_step,
                    455,
                    292,
                    120,
                    34,
                    hfont_bold,
                    ButtonVariant::Primary,
                    None,
                ) {
                    btn.on_click(move || add_step_from_inputs(hwnd));
                }

                // DSL title & Footer hint labels
                Label::create(hwnd, "DSL:", 26, 357, 34, 20, hfont_bold).map(|l| {
                    l.with_tone(LabelTone::Secondary)
                        .on_surface(SurfaceRole::Card)
                });
                Label::create(
                    hwnd,
                    strings.dlg_key_macro_trigger_hint,
                    16,
                    429,
                    344,
                    24,
                    hfont_normal,
                )
                .map(|l| {
                    l.with_tone(LabelTone::Secondary)
                        .on_surface(SurfaceRole::Window)
                });

                // 7. Footer buttons
                super::create_dialog_footer_layout(
                    hwnd,
                    strings.dlg_btn_cancel,
                    90,
                    strings.btn_save,
                    104,
                    ButtonVariant::Primary,
                    422,
                    32,
                    16,
                    None,
                    move || submit_key_macro(hwnd),
                );

                let initial_actions = st.actions.clone();
                st.txt_name = txt_name;
                st.cbo_action_type = cbo_type;
                st.btn_trigger = btn_trigger;
                st.btn_step_key = btn_step_key;
                st.txt_step_delay = txt_step_delay;
                st.txt_step_text = txt_step_text;
                st.lbl_step_count = lbl_step_count;
                st.table_actions = arc_table.clone();

                if let Some(ref t) = arc_table {
                    populate_action_table_rows(hwnd, t, &initial_actions);
                }

                0
            }

            WM_ERASEBKGND => 1,

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                let palette = ThemePalette::get(is_current_dark());
                let hfont_normal = get_app_font_normal();

                let client_rc = super::base::fill_dialog_bg(hwnd, hdc, &palette);

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
                RoundRect(
                    hdc,
                    rc_builder.left,
                    rc_builder.top,
                    rc_builder.right,
                    rc_builder.bottom,
                    6,
                    6,
                );
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
                RoundRect(
                    hdc,
                    rc_dsl.left,
                    rc_dsl.top,
                    rc_dsl.right,
                    rc_dsl.bottom,
                    6,
                    6,
                );
                SelectObject(hdc, ob);
                SelectObject(hdc, op);
                DeleteObject(br_dsl);
                DeleteObject(pen_dsl);

                // DSL text content
                let dsl_content = state::<KeyMacroDialogState>(hwnd)
                    .map(|s| s.dsl_text.clone())
                    .unwrap_or_else(|| "(Trống)".to_string());
                let old_font = SelectObject(hdc, hfont_normal);
                SetTextColor(hdc, 0x00B0C94E); // Teal accent for DSL
                let mut rc_dsl_txt = RECT {
                    left: 64,
                    top: 357,
                    right: 574,
                    bottom: 377,
                };
                let w_dsl_txt = to_wide(&dsl_content);
                DrawTextW(hdc, w_dsl_txt.as_ptr(), -1, &mut rc_dsl_txt, 0x00004000); // DT_END_ELLIPSIS
                SelectObject(hdc, old_font);

                // Footer divider line
                draw_line(hdc, 0, 406, client_rc.right, 406, palette.border_separator);

                EndPaint(hwnd, &ps);
                0
            }

            WM_COMMAND => {
                let ctrl_id = (wparam & 0xFFFF) as u32;
                let notify_code = ((wparam >> 16) & 0xFFFF) as u32;

                let is_combo_changed = state::<KeyMacroDialogState>(hwnd)
                    .and_then(|s| {
                        s.cbo_action_type
                            .as_ref()
                            .map(|c| c.is_changed(ctrl_id, notify_code))
                    })
                    .unwrap_or(false);

                if is_combo_changed {
                    stop_single_key_capture();
                    stop_combo_capture();

                    let (sel_idx, btn_hwnd, delay_hwnd, text_hwnd) = {
                        if let Some(s) = state::<KeyMacroDialogState>(hwnd) {
                            let idx = s
                                .cbo_action_type
                                .as_ref()
                                .and_then(|c| c.get_selected())
                                .unwrap_or(0);
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
                    IDOK => submit_key_macro(hwnd),
                    IDCANCEL => cancel_key_macro(hwnd),
                    _ => {}
                }
                0
            }

            WM_SINGLE_KEY_CAPTURED => {
                let vk = wparam as u32;
                let mut h_target = 0;

                if let Some(s) = state::<KeyMacroDialogState>(hwnd) {
                    match s.active_capture {
                        ActiveCaptureTarget::TriggerKey => {
                            stop_single_key_capture();
                            s.trigger_vk = Some(vk);
                            s.trigger_display = vk_to_name(vk).to_string();
                            s.active_capture = ActiveCaptureTarget::None;
                            h_target = s.btn_trigger.as_ref().map_or(0, |b| b.hwnd());
                        }
                        ActiveCaptureTarget::StepKey => {
                            stop_single_key_capture();
                            s.step_key_str = vk_to_name(vk).to_string();
                            s.active_capture = ActiveCaptureTarget::None;
                            h_target = s.btn_step_key.as_ref().map_or(0, |b| b.hwnd());
                        }
                        ActiveCaptureTarget::None => {}
                    }
                }

                if h_target != 0 {
                    InvalidateRect(h_target, null_mut(), 0);
                    UpdateWindow(h_target);
                }
                0
            }

            WM_COMBO_CAPTURED => {
                let display = get_captured_display_text();
                let hotkey = get_captured_hotkey();
                let is_final = hotkey.is_some();
                let mut h_step = 0;

                if let Some(s) = state::<KeyMacroDialogState>(hwnd)
                    && s.active_capture == ActiveCaptureTarget::StepKey
                {
                    if !display.is_empty() {
                        s.step_key_str = display;
                    }
                    if is_final {
                        s.active_capture = ActiveCaptureTarget::None;
                    }
                    h_step = s.btn_step_key.as_ref().map_or(0, |b| b.hwnd());
                }

                if is_final {
                    stop_combo_capture();
                }

                if h_step != 0 {
                    InvalidateRect(h_step, null_mut(), 0);
                    UpdateWindow(h_step);
                }
                0
            }

            WM_ACTIVATE => {
                if (wparam & 0xFFFF) == 0 {
                    stop_single_key_capture();
                    stop_combo_capture();
                    let (was_capturing, btn_step_key_hwnd, btn_trigger_hwnd) = {
                        if let Some(s) = state::<KeyMacroDialogState>(hwnd) {
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
                let parent = state::<KeyMacroDialogState>(hwnd).map_or(0, |s| s.parent);
                close_modal(hwnd, parent);
                0
            }

            WM_NCDESTROY => {
                stop_single_key_capture();
                stop_combo_capture();
                if let Some(st) = detach_state::<KeyMacroDialogState>(hwnd) {
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn toggle_trigger_capture(hwnd: isize) {
    let mut btn_hwnd = 0;
    if let Some(s) = unsafe { state::<KeyMacroDialogState>(hwnd) } {
        btn_hwnd = s.btn_trigger.as_ref().map_or(0, |b| b.hwnd());
        if s.active_capture == ActiveCaptureTarget::TriggerKey {
            stop_single_key_capture();
            s.active_capture = ActiveCaptureTarget::None;
        } else {
            stop_combo_capture();
            stop_single_key_capture();
            s.active_capture = ActiveCaptureTarget::TriggerKey;
            start_single_key_capture(hwnd);
        }
    }
    unsafe {
        if btn_hwnd != 0 {
            InvalidateRect(btn_hwnd, null_mut(), 0);
            UpdateWindow(btn_hwnd);
        }
        InvalidateRect(hwnd, null_mut(), 0);
    }
}

unsafe fn toggle_step_key_capture(hwnd: isize) {
    let mut btn_hwnd = 0;
    if let Some(s) = unsafe { state::<KeyMacroDialogState>(hwnd) } {
        btn_hwnd = s.btn_step_key.as_ref().map_or(0, |b| b.hwnd());
        if s.active_capture == ActiveCaptureTarget::StepKey {
            stop_single_key_capture();
            stop_combo_capture();
            s.active_capture = ActiveCaptureTarget::None;
        } else {
            stop_single_key_capture();
            stop_combo_capture();
            s.active_capture = ActiveCaptureTarget::StepKey;
            if s.current_action_type == 0 {
                start_combo_capture(hwnd, true);
            } else {
                start_single_key_capture(hwnd);
            }
        }
    }
    unsafe {
        if btn_hwnd != 0 {
            InvalidateRect(btn_hwnd, null_mut(), 0);
            UpdateWindow(btn_hwnd);
        }
        InvalidateRect(hwnd, null_mut(), 0);
    }
}

unsafe fn add_step_from_inputs(hwnd: isize) {
    stop_single_key_capture();
    stop_combo_capture();

    let (action_added, actions_copy, table_opt, btn_step_key_hwnd) = {
        if let Some(s) = unsafe { state::<KeyMacroDialogState>(hwnd) } {
            let mut action_opt = None;
            match s.current_action_type {
                0 => {
                    let key = s.step_key_str.replace(" + ", "+").trim().to_string();
                    if !key.is_empty() && !key.contains("Bấm phím") && !key.contains("Cần thêm")
                    {
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

            if let Some(act) = action_opt {
                s.actions.push(act);
                s.active_capture = ActiveCaptureTarget::None;
                s.step_key_str.clear();
                s.dsl_text = if s.actions.is_empty() {
                    "(Trống)".to_string()
                } else {
                    MacroAction::actions_to_dsl(&s.actions)
                };
                if let Some(ref lbl) = s.lbl_step_count {
                    lbl.set_text(&format!("{} bước", s.actions.len()));
                }
                if let Some(ref txt) = s.txt_step_text {
                    txt.set_text("");
                }
                (
                    true,
                    s.actions.clone(),
                    s.table_actions.clone(),
                    s.btn_step_key.as_ref().map(|b| b.hwnd()),
                )
            } else {
                (false, Vec::new(), None, None)
            }
        } else {
            (false, Vec::new(), None, None)
        }
    };

    if action_added {
        if let Some(h_btn) = btn_step_key_hwnd {
            unsafe {
                InvalidateRect(h_btn, null_mut(), 0);
                UpdateWindow(h_btn);
            }
        }
        update_table_and_dialog(table_opt.as_deref(), hwnd, &actions_copy);
    }
}

unsafe fn submit_key_macro(hwnd: isize) {
    stop_single_key_capture();
    stop_combo_capture();

    let (trigger_vk, name, actions, orig_vk) = {
        if let Some(s) = unsafe { state::<KeyMacroDialogState>(hwnd) } {
            let mut n = s
                .txt_name
                .as_ref()
                .map(|t| t.get_text())
                .unwrap_or_default();
            if n.trim().is_empty() {
                n = format!("Macro {}", s.trigger_display);
            }
            (s.trigger_vk, n, s.actions.clone(), s.editing_trigger_vk)
        } else {
            (None, String::new(), Vec::new(), None)
        }
    };

    let Some(trig_vk) = trigger_vk else {
        return;
    };
    if actions.is_empty() {
        return;
    }

    crate::ui::controller::save_key_macro(orig_vk, trig_vk, name, actions);
    unsafe {
        PostMessageW(hwnd, WM_CLOSE, 0, 0);
    }
}

unsafe fn cancel_key_macro(hwnd: isize) {
    stop_single_key_capture();
    stop_combo_capture();
    unsafe {
        PostMessageW(hwnd, WM_CLOSE, 0, 0);
    }
}
