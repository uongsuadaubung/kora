//! Tab 0: Typing & Input Method Configuration (Bộ gõ)
//! Uses CardLayout (1-line row builders, auto-HWND & auto-i18n), Label::as_pill_badge,
//! and semantic component event matchers.
//!
//! Card 4 (Target key injection method) strictly matches the visual mockup in mockups/mockup_typing.html:
//! dedicated Inspect Tool Box container with real-time status and trigger pill,
//! followed by rich Target Rule Item cards with index badges, pill badges, danger delete buttons,
//! strategy dropdowns, and dynamic explanation blocks.

use crate::engine::EngineConfig;
use crate::engine::config::InputMethod;
use crate::engine::profile::{
    AppProfile, Child, KeyInjectionStrategy, LanguageMode, MethodMode, TargetRule,
};
use crate::language::LanguageStrings;
use crate::ui::colors::ThemePalette;
use crate::ui::prelude::*;
use crate::ui_framework::tokens::{borders, layout, radii};
use crate::ui_framework::win32::*;
use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

/// The single "Simple Telex" combo entry maps to both SimpleTelex1/2: keep whichever is stored.
fn simple_telex_keep_variant(current: InputMethod) -> InputMethod {
    match current {
        InputMethod::SimpleTelex2 => InputMethod::SimpleTelex2,
        _ => InputMethod::SimpleTelex1,
    }
}

fn decode_input_method(idx: usize, current: InputMethod) -> Option<InputMethod> {
    match idx {
        0 => Some(InputMethod::Telex),
        1 => Some(InputMethod::Vni),
        2 => Some(simple_telex_keep_variant(current)),
        _ => None,
    }
}

fn strategy_to_index(strategy: KeyInjectionStrategy) -> usize {
    match strategy {
        KeyInjectionStrategy::AutocompleteGuard => 0,
        KeyInjectionStrategy::ClipboardPaste => 1,
    }
}

fn index_to_strategy(idx: usize) -> KeyInjectionStrategy {
    match idx {
        1 => KeyInjectionStrategy::ClipboardPaste,
        _ => KeyInjectionStrategy::AutocompleteGuard,
    }
}

fn strategy_description(strategy: KeyInjectionStrategy, strings: &LanguageStrings) -> &'static str {
    match strategy {
        KeyInjectionStrategy::AutocompleteGuard => strings.strategy_desc_autocomplete_guard,
        KeyInjectionStrategy::ClipboardPaste => strings.strategy_desc_clipboard_paste,
    }
}

fn app_strategy_to_index(strategy: Option<KeyInjectionStrategy>) -> usize {
    match strategy {
        None => 0,
        Some(KeyInjectionStrategy::AutocompleteGuard) => 1,
        Some(KeyInjectionStrategy::ClipboardPaste) => 2,
    }
}

fn index_to_app_strategy(idx: usize) -> Option<KeyInjectionStrategy> {
    match idx {
        1 => Some(KeyInjectionStrategy::AutocompleteGuard),
        2 => Some(KeyInjectionStrategy::ClipboardPaste),
        _ => None,
    }
}

fn app_strategy_description(
    strategy: Option<KeyInjectionStrategy>,
    strings: &LanguageStrings,
) -> &'static str {
    match strategy {
        None => strings.typing_app_strategy_desc_inherit,
        Some(KeyInjectionStrategy::AutocompleteGuard) => strings.strategy_desc_autocomplete_guard,
        Some(KeyInjectionStrategy::ClipboardPaste) => strings.strategy_desc_clipboard_paste,
    }
}

#[derive(Clone, Copy)]
enum SubBoxKind {
    AppStrategyBox,
    SectionDivider,
    InspectBox,
    EmptyBox,
    RuleItem,
    RuleControls,
}

#[derive(Clone, Copy)]
struct SubBoxRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    kind: SubBoxKind,
}

struct SubBoxesState {
    is_child: bool,
    boxes: Vec<SubBoxRect>,
}

pub struct TargetRuleItemUi {
    pub target_id: String,
    pub lbl_index: Label<StaticText>,
    pub lbl_badge: Label<PillBadge>,
    pub btn_delete: PushButton,
    pub lbl_strategy: Label<StaticText>,
    pub combo_strategy: ComboBox,
    pub lbl_desc: Label<StaticText>,
}

impl TargetRuleItemUi {
    pub fn destroy(&self) {
        unsafe {
            DestroyWindow(self.lbl_index.hwnd());
            DestroyWindow(self.lbl_badge.hwnd());
            DestroyWindow(self.btn_delete.hwnd());
            DestroyWindow(self.lbl_strategy.hwnd());
            DestroyWindow(self.combo_strategy.hwnd());
            DestroyWindow(self.lbl_desc.hwnd());
        }
    }

    pub fn set_visible(&self, visible: bool) {
        self.lbl_index.set_visible(visible);
        self.lbl_badge.set_visible(visible);
        self.btn_delete.set_visible(visible);
        self.lbl_strategy.set_visible(visible);
        self.combo_strategy.set_visible(visible);
        self.lbl_desc.set_visible(visible);
    }
}

pub struct TabTyping {
    pub scroll_view: ScrollView,

    layout: CardLayout,
    // Card 1: Thiết lập Bộ gõ tiếng Việt
    pub combo_mode: ComboBox,
    pub combo_method: ComboBox,

    // Card 2: Phím chuyển đổi E ⇄ V (Checkbox title toggles switch shortcut)
    pub check_switch_key: CheckBox<Inheritable>,
    pub lbl_switch_badge: Label<PillBadge>,
    pub btn_switch_key: PushButton,

    // Card 3: Tính năng hỗ trợ gõ & kiểm tra chính tả
    pub check_restore_wrong: CheckBox<Inheritable>,
    pub check_auto_upper: CheckBox<Inheritable>,

    // Card 4: 🎯 Phương pháp gửi phím theo ô text (Chỉ hiện khi ở Child Profile)
    h_content: isize,
    hfont_normal: isize,
    hfont_bold: isize,
    y_card4: i32,
    pub lbl_target_title: Label<StaticText>,
    pub lbl_target_desc: Label<StaticText>,
    pub lbl_app_strat_title: Label<StaticText>,
    pub lbl_app_strat_sub: Label<StaticText>,
    pub combo_app_strategy: ComboBox,
    pub lbl_app_strat_desc: Label<StaticText>,
    pub lbl_exceptions_title: Label<StaticText>,
    pub lbl_exceptions_desc: Label<StaticText>,
    pub lbl_capture_title: Label<StaticText>,
    pub lbl_capture_badge: Label<PillBadge>,
    pub btn_capture_change: PushButton,
    pub lbl_capture_guide: Label<StaticText>,
    pub lbl_target_subtitle: Label<PillBadge>,
    pub lbl_empty_state: Label<StaticText>,
    pub rule_items: RefCell<Vec<TargetRuleItemUi>>,
    sub_boxes: Arc<Mutex<SubBoxesState>>,
    pub is_child_profile: Cell<bool>,
    pub is_tab_visible: Cell<bool>,
}

impl TabTyping {
    #[inline]
    fn card4_y_app_box(y_card4: i32) -> i32 {
        y_card4 + 82
    }

    #[inline]
    fn card4_y_exc(y_card4: i32) -> i32 {
        Self::card4_y_app_box(y_card4) + 130
    }

    #[inline]
    fn card4_y_box(y_card4: i32) -> i32 {
        Self::card4_y_exc(y_card4) + 70
    }

    #[inline]
    fn card4_y_list(y_card4: i32) -> i32 {
        Self::card4_y_box(y_card4) + 94
    }
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let viewport_x = layout::CARD_X;
        let viewport_y = layout::CARD_START_Y;
        let viewport_w = layout::CARD_WIDTH + 8;
        let viewport_h = layout::CARD_BOTTOM_Y - layout::CARD_START_Y;
        let scroll_view = ScrollView::create(parent, viewport_x, viewport_y, viewport_w, viewport_h)?;
        let h_content = scroll_view.content_hwnd();

        let mut ly = CardLayout::with_parent(h_content, hfont_normal, hfont_bold)
            .at(4)
            .with_bounds(16, 530);

        // --- Card 1: Thiết lập Bộ gõ tiếng Việt ---
        let (combo_mode, combo_method) = ly.card_with_pad(10, 12, |card| {
            card.section_title(|s| s.typing_group_main, 4)?;
            let ((_, combo_mode), (_, combo_method)) = card.two_col_combos(
                20,
                0,
                true,
                (|s| s.typing_lbl_mode, |s| vec![s.mode_vietnamese, s.mode_english]),
                (
                    |s| s.typing_lbl_method,
                    |s| vec![s.method_telex, s.method_vni, s.method_simple_telex],
                ),
            )?;
            Some((combo_mode, combo_method))
        })?;

        // --- Card 2: Phím chuyển đổi E ⇄ V ---
        let (check_switch_key, lbl_switch_badge, btn_switch_key) =
            ly.card_with_pad(6, 10, |card| {
                let check_switch_key =
                    card.inheritable_checkbox_row(|s| s.typing_group_switch, 24, 4, true)?;

                let y_sw_row = card.row(28, 0);
                card.label_at(
                    |s| s.typing_lbl_current_key,
                    card.x(),
                    y_sw_row + 4,
                    124,
                    20,
                    true,
                )?;

                let btn_w = 86;
                let btn_x = card.right() - btn_w - 4;
                let badge_x = card.x() + 130;
                let badge_max_w = btn_x - badge_x - 10;

                let lbl_switch_badge = card.track(
                    Label::create_centered(
                        h_content,
                        "Ctrl + Shift",
                        badge_x,
                        y_sw_row,
                        120,
                        28,
                        hfont_bold,
                    )?
                    .as_pill_badge(120, badge_max_w),
                );

                let btn_switch_key = card.button_at(
                    |s| s.typing_btn_change_key,
                    btn_x,
                    y_sw_row,
                    btn_w,
                    28,
                    true,
                    ButtonVariant::Primary,
                )?;
                Some((check_switch_key, lbl_switch_badge, btn_switch_key))
            })?;

        // --- Card 3: Tính năng hỗ trợ gõ & kiểm tra chính tả ---
        let (check_restore_wrong, check_auto_upper) = ly.card_with_pad(6, 10, |card| {
            card.section_title(|s| s.typing_group_features, 4)?;
            let check_restore_wrong =
                card.inheritable_checkbox_row(|s| s.check_restore_wrong, 20, 2, false)?;
            let check_auto_upper =
                card.inheritable_checkbox_row(|s| s.check_auto_upper, 20, 0, false)?;
            Some((check_restore_wrong, check_auto_upper))
        })?;

        // --- Card 4: ⚡ Cơ chế gửi phím vào ứng dụng (Chỉ hiện khi ở Child Profile) ---
        let y_card4 = ly.y();
        let strings = crate::language::current();

        let lbl_target_title = Label::create(
            h_content,
            strings.typing_group_target,
            28,
            y_card4 + 14,
            506,
            22,
            hfont_bold,
        )?
        .with_tone(LabelTone::Accent);

        let lbl_target_desc = Label::create(
            h_content,
            strings.typing_target_desc,
            28,
            y_card4 + 38,
            506,
            36,
            hfont_normal,
        )?
        .with_tone(LabelTone::Secondary);

        // Khối 1: App-level Strategy Box (y_card4 + 82, h: 110)
        let y_app_box = Self::card4_y_app_box(y_card4);
        let lbl_app_strat_title = Label::create(
            h_content,
            strings.typing_app_strategy_label,
            28,
            y_app_box + 12,
            220,
            22,
            hfont_bold,
        )?
        .on_surface(SurfaceRole::Window);

        let combo_app_strategy = ComboBox::create(
            h_content,
            0,
            254,
            y_app_box + 10,
            280,
            160,
            hfont_normal,
        )?;

        let lbl_app_strat_sub = Label::create(
            h_content,
            "",
            28,
            y_app_box + 38,
            506,
            18,
            hfont_normal,
        )?
        .with_tone(LabelTone::Secondary)
        .on_surface(SurfaceRole::Window);

        let lbl_app_strat_desc = Label::create(
            h_content,
            strings.typing_app_strategy_desc_inherit,
            28,
            y_app_box + 66,
            506,
            36,
            hfont_normal,
        )?
        .with_tone(LabelTone::Secondary)
        .on_surface(SurfaceRole::Window);

        // Khối 2: Ngoại lệ theo từng ô text
        let y_exc = Self::card4_y_exc(y_card4);
        let lbl_exceptions_title = Label::create(
            h_content,
            strings.typing_exceptions_title,
            28,
            y_exc,
            410,
            22,
            hfont_bold,
        )?
        .with_tone(LabelTone::Accent);

        // Pill badge trên tiêu đề: "{count} ngoại lệ"
        let lbl_target_subtitle = Label::create_centered(
            h_content,
            "",
            446,
            y_exc - 2,
            88,
            24,
            hfont_bold,
        )?
        .as_pill_badge(70, 96);

        let lbl_exceptions_desc = Label::create(
            h_content,
            strings.typing_exceptions_desc,
            28,
            y_exc + 26,
            506,
            36,
            hfont_normal,
        )?
        .with_tone(LabelTone::Secondary);

        // Capture Hotkey Box (left: 16, top: y_box, right: 546, bottom: y_box + 80)
        let y_box = Self::card4_y_box(y_card4);
        let lbl_capture_title = Label::create(
            h_content,
            strings.typing_group_target_hotkey,
            28,
            y_box + 12,
            200,
            24,
            hfont_bold,
        )?
        .on_surface(SurfaceRole::Window);

        let lbl_capture_badge = Label::create_centered(
            h_content,
            "",
            236,
            y_box + 10,
            198,
            26,
            hfont_bold,
        )?
        .as_pill_badge(80, 200)
        .on_surface(SurfaceRole::Window);

        let btn_capture_change = PushButton::create_with_variant(
            h_content,
            0,
            strings.typing_btn_change_key,
            446,
            y_box + 10,
            88,
            26,
            hfont_bold,
            ButtonVariant::Primary,
            None,
        )?;

        let lbl_capture_guide = Label::create(
            h_content,
            strings.typing_target_step_guide,
            32,
            y_box + 46,
            498,
            20,
            hfont_normal,
        )?
        .with_tone(LabelTone::Secondary)
        .on_surface(SurfaceRole::Card);

        // Empty state & List start
        let y_list = Self::card4_y_list(y_card4);
        let lbl_empty_state = Label::create(
            h_content,
            strings.typing_target_empty_hint,
            28,
            y_list + 14,
            506,
            36,
            hfont_normal,
        )?
        .with_tone(LabelTone::Secondary)
        .on_surface(SurfaceRole::Window);

        let sub_boxes = Arc::new(Mutex::new(SubBoxesState {
            is_child: false,
            boxes: Vec::new(),
        }));

        let sub_boxes_paint = sub_boxes.clone();
        scroll_view.set_on_paint(move |hdc, palette| {
            if let Ok(state) = sub_boxes_paint.lock() {
                if !state.is_child {
                    return;
                }
                for b in &state.boxes {
                    match b.kind {
                        SubBoxKind::AppStrategyBox => {
                            let br = unsafe { CreateSolidBrush(palette.bg_window) };
                            let pen = unsafe { CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border) };
                            let ob = unsafe { SelectObject(hdc, br) };
                            let op = unsafe { SelectObject(hdc, pen) };
                            unsafe {
                                RoundRect(
                                    hdc,
                                    b.left,
                                    b.top,
                                    b.right,
                                    b.bottom,
                                    radii::RADIUS_CARD,
                                    radii::RADIUS_CARD,
                                );
                                SelectObject(hdc, ob);
                                SelectObject(hdc, op);
                                DeleteObject(br);
                                DeleteObject(pen);

                                let line_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
                                let old_p = SelectObject(hdc, line_pen);
                                MoveToEx(hdc, b.left + 12, b.top + 60, null_mut());
                                LineTo(hdc, b.right - 12, b.top + 60);
                                SelectObject(hdc, old_p);
                                DeleteObject(line_pen);
                            }
                        }
                        SubBoxKind::SectionDivider => {
                            let pen_div = unsafe { CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border) };
                            let op_div = unsafe { SelectObject(hdc, pen_div) };
                            unsafe {
                                MoveToEx(hdc, b.left, b.top, null_mut());
                                LineTo(hdc, b.right, b.top);
                                SelectObject(hdc, op_div);
                                DeleteObject(pen_div);
                            }
                        }
                        SubBoxKind::InspectBox => {
                            let br = unsafe { CreateSolidBrush(palette.bg_window) };
                            let pen = unsafe { CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border) };
                            let ob = unsafe { SelectObject(hdc, br) };
                            let op = unsafe { SelectObject(hdc, pen) };
                            unsafe {
                                RoundRect(
                                    hdc,
                                    b.left,
                                    b.top,
                                    b.right,
                                    b.bottom,
                                    radii::RADIUS_CARD,
                                    radii::RADIUS_CARD,
                                );
                                SelectObject(hdc, ob);
                                SelectObject(hdc, op);
                                DeleteObject(br);
                                DeleteObject(pen);

                                let g_br = CreateSolidBrush(palette.bg_card);
                                let g_pen = CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border);
                                let g_ob = SelectObject(hdc, g_br);
                                let g_op = SelectObject(hdc, g_pen);
                                RoundRect(
                                    hdc,
                                    b.left + 8,
                                    b.top + 42,
                                    b.right - 8,
                                    b.bottom - 8,
                                    radii::RADIUS_MD,
                                    radii::RADIUS_MD,
                                );
                                SelectObject(hdc, g_ob);
                                SelectObject(hdc, g_op);
                                DeleteObject(g_br);
                                DeleteObject(g_pen);
                            }
                        }
                        SubBoxKind::EmptyBox | SubBoxKind::RuleItem => {
                            let br = unsafe { CreateSolidBrush(palette.bg_window) };
                            let pen = unsafe { CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border) };
                            let ob = unsafe { SelectObject(hdc, br) };
                            let op = unsafe { SelectObject(hdc, pen) };
                            unsafe {
                                RoundRect(
                                    hdc,
                                    b.left,
                                    b.top,
                                    b.right,
                                    b.bottom,
                                    radii::RADIUS_CARD,
                                    radii::RADIUS_CARD,
                                );
                                SelectObject(hdc, ob);
                                SelectObject(hdc, op);
                                DeleteObject(br);
                                DeleteObject(pen);
                            }
                        }
                        SubBoxKind::RuleControls => {
                            let br = unsafe { CreateSolidBrush(palette.bg_card) };
                            let pen = unsafe { CreatePen(PS_SOLID, borders::WIDTH_THIN, palette.border) };
                            let ob = unsafe { SelectObject(hdc, br) };
                            let op = unsafe { SelectObject(hdc, pen) };
                            unsafe {
                                RoundRect(
                                    hdc,
                                    b.left,
                                    b.top,
                                    b.right,
                                    b.bottom,
                                    radii::RADIUS_MD,
                                    radii::RADIUS_MD,
                                );
                                SelectObject(hdc, ob);
                                SelectObject(hdc, op);
                                DeleteObject(br);
                                DeleteObject(pen);
                            }
                        }
                    }
                }
            }
        });

        scroll_view.set_cards(ly.cards().to_vec());
        let tab = Self {
            scroll_view,
            layout: ly,
            combo_mode,
            combo_method,
            check_switch_key,
            lbl_switch_badge,
            btn_switch_key,
            check_restore_wrong,
            check_auto_upper,
            h_content,
            hfont_normal,
            hfont_bold,
            y_card4,
            lbl_target_title,
            lbl_target_desc,
            lbl_app_strat_title,
            lbl_app_strat_sub,
            combo_app_strategy,
            lbl_app_strat_desc,
            lbl_exceptions_title,
            lbl_exceptions_desc,
            lbl_capture_title,
            lbl_capture_badge,
            btn_capture_change,
            lbl_capture_guide,
            lbl_target_subtitle,
            lbl_empty_state,
            rule_items: RefCell::new(Vec::new()),
            sub_boxes,
            is_child_profile: Cell::new(false),
            is_tab_visible: Cell::new(true),
        };
        tab.set_target_ui_visible(false);
        tab.update_switch_controls_state();
        tab.update_capture_hotkey_badge();
        Some(tab)
    }

    /// Dynamically enables or disables switch-key controls based on `check_switch_key`
    pub fn update_switch_controls_state(&self) {
        let is_enabled = self.check_switch_key.is_checked();
        self.btn_switch_key.set_enabled(is_enabled);
        self.lbl_switch_badge.set_enabled(is_enabled);
    }

    /// Updates the target capture hotkey badge with latest configured hotkey
    pub fn update_capture_hotkey_badge(&self) {
        let strings = crate::language::current();
        let cap_text = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            let sel_id = &pm.selected_id;
            let g_hk = e.config().target_capture_hotkey;
            if sel_id == "global" {
                g_hk.display_text()
            } else if let Some(c) = pm.get_profile(sel_id) {
                match &c.target_capture_hotkey {
                    Some(k) => k.display_text(),
                    None => format!(
                        "{} ({})",
                        strings.typing_inherit_prefix,
                        g_hk.display_text()
                    ),
                }
            } else {
                g_hk.display_text()
            }
        })
        .unwrap_or_else(|| "Ctrl + ~".to_string());
        self.lbl_capture_badge.set_text(&cap_text);
    }

    /// Populates dynamic rule item cards for surgical target injection rules
    pub fn populate_target_rules(
        &self,
        rules: &[TargetRule],
        app_name: &str,
        app_strategy: Option<KeyInjectionStrategy>,
    ) {
        for item in self.rule_items.borrow().iter() {
            item.destroy();
        }
        self.rule_items.borrow_mut().clear();

        let strings = crate::language::current();
        let count_text = strings
            .typing_target_count_format
            .replace("{count}", &rules.len().to_string());
        self.lbl_target_subtitle.set_text(&count_text);

        let y_app_box = Self::card4_y_app_box(self.y_card4);
        let y_exc = Self::card4_y_exc(self.y_card4);
        let y_box = Self::card4_y_box(self.y_card4);
        let y_list = Self::card4_y_list(self.y_card4);

        let mut boxes = vec![
            SubBoxRect {
                left: 16,
                top: y_app_box,
                right: 546,
                bottom: y_app_box + 110,
                kind: SubBoxKind::AppStrategyBox,
            },
            SubBoxRect {
                left: 28,
                top: y_exc - 10,
                right: 534,
                bottom: y_exc - 10,
                kind: SubBoxKind::SectionDivider,
            },
            SubBoxRect {
                left: 16,
                top: y_box,
                right: 546,
                bottom: y_box + 80,
                kind: SubBoxKind::InspectBox,
            },
        ];

        let strategy_items = [
            strings.strategy_autocomplete_guard,
            strings.strategy_clipboard_paste,
        ];

        let is_tab_vis = self.is_tab_visible.get();
        let mut new_items = Vec::with_capacity(rules.len());

        if rules.is_empty() {
            boxes.push(SubBoxRect {
                left: 16,
                top: y_list,
                right: 546,
                bottom: y_list + 64,
                kind: SubBoxKind::EmptyBox,
            });

            let empty_text = match app_strategy {
                None => strings
                    .typing_target_empty_inherit_format
                    .replace("{app}", app_name),
                Some(KeyInjectionStrategy::AutocompleteGuard) => strings
                    .typing_target_empty_guard_format
                    .replace("{app}", app_name),
                Some(KeyInjectionStrategy::ClipboardPaste) => strings
                    .typing_target_empty_paste_format
                    .replace("{app}", app_name),
            };
            self.lbl_empty_state.set_text(&empty_text);
            self.lbl_empty_state.set_visible(is_tab_vis);
        } else {
            self.lbl_empty_state.set_text("");
            self.lbl_empty_state.set_visible(false);

            for (i, r) in rules.iter().enumerate() {
                let y_item = y_list + (i as i32) * 136;

                boxes.push(SubBoxRect {
                    left: 16,
                    top: y_item,
                    right: 546,
                    bottom: y_item + 124,
                    kind: SubBoxKind::RuleItem,
                });
                boxes.push(SubBoxRect {
                    left: 24,
                    top: y_item + 44,
                    right: 538,
                    bottom: y_item + 116,
                    kind: SubBoxKind::RuleControls,
                });

                let lbl_index = Label::create(
                    self.h_content,
                    &format!("#{}", i + 1),
                    28,
                    y_item + 10,
                    32,
                    24,
                    self.hfont_bold,
                )
                .map(|l| l.with_tone(LabelTone::Accent).on_surface(SurfaceRole::Window));

                let lbl_badge = Label::create_centered(
                    self.h_content,
                    &r.target_id,
                    64,
                    y_item + 8,
                    240,
                    28,
                    self.hfont_bold,
                )
                .map(|l| l.as_pill_badge(80, 240).on_surface(SurfaceRole::Window));

                let btn_delete = PushButton::create_with_variant(
                    self.h_content,
                    0,
                    "✕",
                    504,
                    y_item + 10,
                    30,
                    24,
                    self.hfont_normal,
                    ButtonVariant::Danger,
                    None,
                );

                let lbl_strategy = Label::create(
                    self.h_content,
                    strings.typing_target_method_label,
                    36,
                    y_item + 52,
                    110,
                    22,
                    self.hfont_normal,
                )
                .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Card));

                let combo_strategy = ComboBox::create(
                    self.h_content,
                    0,
                    150,
                    y_item + 50,
                    376,
                    160,
                    self.hfont_normal,
                );

                if let Some(ref combo) = combo_strategy {
                    let sel_idx = strategy_to_index(r.strategy);
                    combo.reset_items(&strategy_items, Some(sel_idx));
                }

                let desc_text = strategy_description(r.strategy, strings);
                let lbl_desc = Label::create(
                    self.h_content,
                    desc_text,
                    36,
                    y_item + 82,
                    492,
                    26,
                    self.hfont_normal,
                )
                .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Card));

                if let (
                    Some(lbl_index),
                    Some(lbl_badge),
                    Some(btn_delete),
                    Some(lbl_strategy),
                    Some(combo_strategy),
                    Some(lbl_desc),
                ) = (
                    lbl_index,
                    lbl_badge,
                    btn_delete,
                    lbl_strategy,
                    combo_strategy,
                    lbl_desc,
                ) {
                    if !is_tab_vis {
                        lbl_index.set_visible(false);
                        lbl_badge.set_visible(false);
                        btn_delete.set_visible(false);
                        lbl_strategy.set_visible(false);
                        combo_strategy.set_visible(false);
                        lbl_desc.set_visible(false);
                    }
                    new_items.push(TargetRuleItemUi {
                        target_id: r.target_id.clone(),
                        lbl_index,
                        lbl_badge,
                        btn_delete,
                        lbl_strategy,
                        combo_strategy,
                        lbl_desc,
                    });
                }
            }
        }

        *self.rule_items.borrow_mut() = new_items;

        if let Ok(mut sb) = self.sub_boxes.lock() {
            sb.boxes = boxes;
        }
    }

    pub fn viewport_hwnd(&self) -> isize {
        self.scroll_view.viewport_hwnd()
    }

    pub fn calculate_card4_bottom(&self) -> i32 {
        let n = self.rule_items.borrow().len();
        let y_list = Self::card4_y_list(self.y_card4);
        if n == 0 {
            y_list + 64 + 16
        } else {
            y_list + (n as i32) * 136 + 12
        }
    }

    pub fn content_height(&self) -> i32 {
        if self.is_child_profile.get() {
            self.calculate_card4_bottom() + 12
        } else {
            self.layout.cards().last().map(|&(_, b)| b).unwrap_or(0) + 8
        }
    }

    fn set_target_ui_visible(&self, visible: bool) {
        self.lbl_target_title.set_visible(visible);
        self.lbl_target_desc.set_visible(visible);
        self.lbl_app_strat_title.set_visible(visible);
        self.lbl_app_strat_sub.set_visible(visible);
        self.combo_app_strategy.set_visible(visible);
        self.lbl_app_strat_desc.set_visible(visible);
        self.lbl_exceptions_title.set_visible(visible);
        self.lbl_exceptions_desc.set_visible(visible);
        self.lbl_capture_title.set_visible(visible);
        self.lbl_capture_badge.set_visible(visible);
        self.btn_capture_change.set_visible(visible);
        self.lbl_capture_guide.set_visible(visible);
        self.lbl_target_subtitle.set_visible(visible);
        let empty = self.rule_items.borrow().is_empty();
        self.lbl_empty_state.set_visible(visible && empty);
        for item in self.rule_items.borrow().iter() {
            item.set_visible(visible);
        }
    }

    /// Loads profile settings (`None` = Root/Global, `Some(child)` = Child profile with Inherit options)
    pub fn load_profile(&self, child: Option<&AppProfile<Child>>, global_config: &EngineConfig) {
        let strings = crate::language::current();
        let is_root = child.is_none();
        let is_child = child.is_some();
        let offset = usize::from(is_root);
        self.is_child_profile.set(is_child);

        // Card 4 visibility: completely hidden if root/global profile!
        let is_tab_vis = self.is_tab_visible.get();
        if is_child {
            if let Some(c) = child {
                let app_strat_items = [
                    strings.typing_app_strategy_inherit,
                    strings.strategy_autocomplete_guard,
                    strings.strategy_clipboard_paste,
                ];
                let sel_app_idx = app_strategy_to_index(c.app_strategy);
                self.combo_app_strategy
                    .reset_items(&app_strat_items, Some(sel_app_idx));

                let sub_text = strings
                    .typing_app_strategy_sub_format
                    .replace("{app}", &c.name);
                self.lbl_app_strat_sub.set_text(&sub_text);

                let desc_text = app_strategy_description(c.app_strategy, strings);
                self.lbl_app_strat_desc.set_text(desc_text);

                self.populate_target_rules(&c.target_rules, &c.name, c.app_strategy);
            }
            if let Ok(mut sb) = self.sub_boxes.lock() {
                sb.is_child = true;
            }
            if is_tab_vis {
                self.set_target_ui_visible(true);
            }
            let card4_bottom = self.calculate_card4_bottom();
            let mut cards = self.layout.cards().to_vec();
            cards.push((self.y_card4, card4_bottom));
            self.scroll_view.set_cards(cards);
            self.scroll_view.set_content_height(card4_bottom + 12);
        } else {
            self.scroll_view.scroll_to(0);
            if let Ok(mut sb) = self.sub_boxes.lock() {
                sb.is_child = false;
            }
            self.set_target_ui_visible(false);
            self.scroll_view.set_cards(self.layout.cards().to_vec());
            self.scroll_view.set_content_height(self.content_height());
        }
        self.scroll_view.invalidate();

        let all_modes = [
            strings.typing_inherit_global,
            strings.mode_vietnamese,
            strings.mode_english,
        ];
        let mode_idx = match child.map(|c| c.mode) {
            None => usize::from(!global_config.enabled),
            Some(LanguageMode::Inherit) => 0,
            Some(LanguageMode::Vietnamese) => 1,
            Some(LanguageMode::English) => 2,
        };
        self.combo_mode
            .reset_items(&all_modes[offset..], Some(mode_idx));

        let all_methods = [
            strings.typing_inherit_global,
            strings.method_telex,
            strings.method_vni,
            strings.method_simple_telex,
        ];
        let method_idx_of = |m: InputMethod| match m {
            InputMethod::Telex => 0,
            InputMethod::Vni => 1,
            InputMethod::SimpleTelex1 | InputMethod::SimpleTelex2 => 2,
        };
        let method_idx = match child.map(|c| c.method) {
            None => method_idx_of(global_config.method),
            Some(MethodMode::Inherit) => 0,
            Some(MethodMode::Specific(m)) => 1 + method_idx_of(m),
        };
        self.combo_method
            .reset_items(&all_methods[offset..], Some(method_idx));

        self.check_switch_key.set_inherited(
            is_root,
            child.and_then(|c| c.switch_key_enabled),
            global_config.switch_key_enabled,
        );
        self.update_switch_controls_state();

        let badge_text = match child {
            None => global_config.switch_key.display_text(),
            Some(c) => match &c.switch_key {
                Some(k) => k.display_text(),
                None => format!(
                    "{} ({})",
                    strings.typing_inherit_prefix,
                    global_config.switch_key.display_text()
                ),
            },
        };
        self.lbl_switch_badge.set_text(&badge_text);
        self.btn_switch_key.set_text(strings.typing_btn_change_key);

        self.check_restore_wrong.set_inherited(
            is_root,
            child.and_then(|c| c.restore_on_wrong_spelling),
            global_config.restore_on_wrong_spelling,
        );
        self.check_auto_upper.set_inherited(
            is_root,
            child.and_then(|c| c.auto_uppercase_first_char),
            global_config.auto_uppercase_first_char,
        );

        let cap_badge_text = match child {
            None => global_config.target_capture_hotkey.display_text(),
            Some(c) => match &c.target_capture_hotkey {
                Some(k) => k.display_text(),
                None => format!(
                    "{} ({})",
                    strings.typing_inherit_prefix,
                    global_config.target_capture_hotkey.display_text()
                ),
            },
        };
        self.lbl_capture_badge.set_text(&cap_badge_text);
    }

    /// Saves controls back into the active target (`Global` config or `Child` profile).
    fn sync_to_target(&self, mut target: crate::ui::controller::EditTarget<'_>) {
        use crate::ui::controller::EditTarget;
        match &mut target {
            EditTarget::Global(config) => {
                if let Some(sel) = self.combo_mode.get_selected() {
                    config.enabled = sel == 0;
                }
                if let Some(m) = self
                    .combo_method
                    .get_selected()
                    .and_then(|sel| decode_input_method(sel, config.method))
                {
                    config.method = m;
                }
            }
            EditTarget::Child(profile) => {
                if let Some(sel) = self.combo_mode.get_selected() {
                    profile.mode = match sel {
                        1 => LanguageMode::Vietnamese,
                        2 => LanguageMode::English,
                        _ => LanguageMode::Inherit,
                    };
                }
                if let Some(sel) = self.combo_method.get_selected() {
                    let current = match profile.method {
                        MethodMode::Specific(m) => m,
                        MethodMode::Inherit => InputMethod::SimpleTelex1,
                    };
                    profile.method = sel
                        .checked_sub(1)
                        .and_then(|idx| decode_input_method(idx, current))
                        .map_or(MethodMode::Inherit, MethodMode::Specific);
                }
            }
        }
        target.sync_checkbox(
            &self.check_switch_key,
            |c| &mut c.switch_key_enabled,
            |p| &mut p.switch_key_enabled,
        );
        target.sync_checkbox(
            &self.check_restore_wrong,
            |c| &mut c.restore_on_wrong_spelling,
            |p| &mut p.restore_on_wrong_spelling,
        );
        target.sync_checkbox(
            &self.check_auto_upper,
            |c| &mut c.auto_uppercase_first_char,
            |p| &mut p.auto_uppercase_first_char,
        );
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.layout.update_language(strings);
        self.lbl_target_title.set_text(strings.typing_group_target);
        self.lbl_target_desc.set_text(strings.typing_target_desc);
        self.lbl_app_strat_title.set_text(strings.typing_app_strategy_label);
        self.lbl_exceptions_title.set_text(strings.typing_exceptions_title);
        self.lbl_exceptions_desc.set_text(strings.typing_exceptions_desc);
        self.lbl_capture_title.set_text(strings.typing_group_target_hotkey);
        self.btn_capture_change.set_text(strings.typing_btn_change_key);
        self.lbl_capture_guide.set_text(strings.typing_target_step_guide);
        self.lbl_empty_state.set_text(strings.typing_target_empty_hint);
        self.update_capture_hotkey_badge();

        let snap = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
        });
        if let Some((child, cfg)) = snap {
            self.load_profile(child.as_ref(), &cfg);
        }
        self.scroll_view.invalidate();
    }

    pub fn paint(&self, _hwnd: isize, _hdc: isize, _right_bound: i32, _palette: &ThemePalette) {
        self.scroll_view.invalidate();
    }

    pub fn on_command(&self, hwnd: isize, ctrl_id: u32, notify_code: u32) -> bool {
        if self.btn_switch_key.is_clicked(ctrl_id, notify_code) {
            if self.btn_switch_key.is_enabled() {
                crate::ui::views::dialogs::show_hotkey_dialog(hwnd);
            }
            return true;
        }

        if self.btn_capture_change.is_clicked(ctrl_id, notify_code) {
            if self.btn_capture_change.is_enabled() {
                crate::ui::views::dialogs::show_target_capture_hotkey_dialog(hwnd);
            }
            return true;
        }

        if self.is_child_profile.get() {
            if self.combo_app_strategy.is_changed(ctrl_id, notify_code) {
                if let Some(sel_idx) = self.combo_app_strategy.get_selected() {
                    let new_app_strat = index_to_app_strategy(sel_idx);
                    let strings = crate::language::current();
                    self.lbl_app_strat_desc
                        .set_text(app_strategy_description(new_app_strat, strings));
                    crate::ui::controller::update_app_key_injection_strategy(new_app_strat);
                    return true;
                }
            }

            let items = self.rule_items.borrow();
            for item in items.iter() {
                if item.btn_delete.is_clicked(ctrl_id, notify_code) {
                    let tid = item.target_id.clone();
                    drop(items);
                    crate::ui::controller::delete_target_rule(&tid);
                    return true;
                }
                if item.combo_strategy.is_changed(ctrl_id, notify_code) {
                    if let Some(sel_idx) = item.combo_strategy.get_selected() {
                        let new_strat = index_to_strategy(sel_idx);
                        let strings = crate::language::current();
                        item.lbl_desc.set_text(strategy_description(new_strat, strings));
                        let tid = item.target_id.clone();
                        drop(items);
                        crate::ui::controller::update_target_rule_strategy(&tid, new_strat);
                        return true;
                    }
                }
            }
        }

        let is_mode_changed = self.combo_mode.is_changed(ctrl_id, notify_code);
        let is_method_changed = self.combo_method.is_changed(ctrl_id, notify_code);
        let is_sw_toggled = self.check_switch_key.is_toggled(ctrl_id, notify_code);
        let is_feat_toggled = self.check_restore_wrong.is_toggled(ctrl_id, notify_code)
            || self.check_auto_upper.is_toggled(ctrl_id, notify_code);

        if is_mode_changed || is_method_changed || is_sw_toggled || is_feat_toggled {
            if is_sw_toggled {
                self.update_switch_controls_state();
            }
            crate::ui::controller::edit_selected(|target| self.sync_to_target(target));
            if is_mode_changed
                && let Some(is_viet) =
                    crate::ui::controller::with_engine(|e| e.effective_config().enabled)
            {
                crate::ui::update_tray_icon(is_viet);
            }
            if is_sw_toggled {
                crate::ui::controller::refresh_keys_overview();
            }
            return true;
        }

        false
    }

    pub fn update_visibility(&self, visible: bool) {
        self.is_tab_visible.set(visible);
        self.scroll_view.set_visible(visible);
        let show_card4 = visible && self.is_child_profile.get();
        self.set_target_ui_visible(show_card4);
    }
}
