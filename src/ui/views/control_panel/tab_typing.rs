//! Tab 0: Typing & Input Method Configuration (Bộ gõ & Phím tắt)
//! Matches mockup_profiles.html 1:1 with 3-Card Fluent container layout.

use crate::engine::EngineConfig;
use crate::engine::config::InputMethod;
use crate::engine::profile::{AppProfile, LanguageMode, MethodMode};
use crate::language::LanguageStrings;
use crate::ui::win32::{
    GetDC, GetTextExtentPoint32W, InvalidateRect, ReleaseDC, SelectObject, SendMessageW, SetBkColor, SetTextColor,
    SetWindowPos, ShowWindow, RECT, SIZE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SW_HIDE, SW_SHOW, WM_GETFONT,
};
use crate::ui::components::*;
use crate::ui::tokens::id::*;

/// Switch-key badge geometry (label inside the pill painted by paint.rs)
const BADGE_X: i32 = 392;
const BADGE_Y: i32 = 270;
const BADGE_H: i32 = 18;
const BADGE_MIN_W: i32 = 116;
const BADGE_MAX_W: i32 = 690 - BADGE_X; // keep clear of the "Đổi phím" button at x=700
const BADGE_PAD_X: i32 = 16;

/// The single "Simple Telex" combo entry maps to both SimpleTelex1/2: keep whichever is stored.
fn simple_telex_keep_variant(current: InputMethod) -> InputMethod {
    match current {
        InputMethod::SimpleTelex2 => InputMethod::SimpleTelex2,
        _ => InputMethod::SimpleTelex1,
    }
}

pub struct TabTyping {
    parent: isize,
    // Card 1: Thiết lập Bộ gõ tiếng Việt
    pub lbl_group_typing: Label,
    pub lbl_mode: Label,
    pub combo_mode: ComboBox,
    pub lbl_method: Label,
    pub combo_method: ComboBox,

    // Card 2: Phím chuyển đổi E ⇄ V (Checkbox title toggles switch shortcut)
    pub check_switch_key: CheckBox,
    pub lbl_switch_prompt: Label,
    pub lbl_switch_badge: Label,
    pub btn_switch_key: PushButton,

    // Card 3: Tính năng hỗ trợ gõ & kiểm tra chính tả
    pub lbl_group_features: Label,
    pub check_restore_wrong: CheckBox,
    pub check_auto_upper: CheckBox,
}

impl TabTyping {
    pub fn create(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Option<Self> {
        let strings = crate::language::current();

        // --- Card 1: Thiết lập Bộ gõ tiếng Việt ---
        let lbl_group_typing = Label::create(
            parent,
            strings.typing_group_main,
            260,
            124,
            260,
            20,
            hfont_bold,
        )?;

        let lbl_mode = Label::create(parent, strings.typing_lbl_mode, 260, 148, 248, 18, hfont_bold)?;
        let combo_mode = ComboBox::create_with_items(
            parent,
            IDC_COMBO_TYPING_MODE,
            260,
            170,
            248,
            150,
            hfont_normal,
            &[strings.mode_vietnamese, strings.mode_english],
            0,
        )?;

        let lbl_method = Label::create(parent, strings.typing_lbl_method, 532, 148, 248, 18, hfont_bold)?;
        let combo_method = ComboBox::create_with_items(
            parent,
            IDC_COMBO_TYPING_METHOD,
            532,
            170,
            248,
            150,
            hfont_normal,
            &[strings.method_telex, strings.method_vni, strings.method_simple_telex],
            0,
        )?;

        // --- Card 2: Phím chuyển đổi E ⇄ V ---
        let check_switch_key = CheckBox::create_with_state(
            parent,
            IDC_CHECK_SWITCH_KEY,
            strings.typing_group_switch,
            260,
            236,
            380,
            24,
            hfont_bold,
            true,
        )?;

        let lbl_switch_prompt =
            Label::create(parent, strings.typing_lbl_current_key, 260, 270, 124, 20, hfont_bold)?;

        let lbl_switch_badge = Label::create_centered_with_id(
            parent,
            IDC_LABEL_SWITCH_KEY_BADGE,
            "Ctrl + Shift",
            BADGE_X,
            BADGE_Y,
            BADGE_MIN_W,
            BADGE_H,
            hfont_bold,
        )?;

        let btn_switch_key = PushButton::create_with_variant(
            parent,
            IDC_BTN_SWITCH_KEY,
            strings.typing_btn_change_key,
            700,
            264,
            86,
            30,
            hfont_bold,
            ButtonVariant::Primary,
            None,
        )?;

        // --- Card 3: Tính năng hỗ trợ gõ & kiểm tra chính tả ---
        let lbl_group_features = Label::create(
            parent,
            strings.typing_group_features,
            260,
            330,
            350,
            20,
            hfont_bold,
        )?;

        let check_restore_wrong = CheckBox::create_with_state(
            parent,
            IDC_CHECK_RESTORE_WRONG,
            strings.check_restore_wrong,
            260,
            356,
            520,
            22,
            hfont_normal,
            true,
        )?;

        let check_auto_upper = CheckBox::create_with_state(
            parent,
            IDC_CHECK_AUTO_UPPER,
            strings.check_auto_upper,
            260,
            382,
            520,
            22,
            hfont_normal,
            false,
        )?;

        let tab = Self {
            parent,
            lbl_group_typing,
            lbl_mode,
            combo_mode,
            lbl_method,
            combo_method,
            check_switch_key,
            lbl_switch_prompt,
            lbl_switch_badge,
            btn_switch_key,
            lbl_group_features,
            check_restore_wrong,
            check_auto_upper,
        };
        tab.update_switch_controls_state();
        Some(tab)
    }

    /// Dynamically enables or disables switch-key controls based on `check_switch_key`
    pub fn update_switch_controls_state(&self) {
        let is_enabled = self.check_switch_key.is_checked();
        self.btn_switch_key.set_enabled(is_enabled);
        unsafe {
            let rc = RECT {
                left: 260,
                top: BADGE_Y - 8,
                right: BADGE_X + BADGE_MAX_W + 4,
                bottom: BADGE_Y + BADGE_H + 8,
            };
            InvalidateRect(self.parent, &rc, 0);
            InvalidateRect(self.lbl_switch_prompt.hwnd(), std::ptr::null(), 0);
            InvalidateRect(self.lbl_switch_badge.hwnd(), std::ptr::null(), 0);
        }
    }

    /// Sets the switch-key badge text and fits the label (and its painted pill) to the content.
    fn set_badge_text(&self, text: &str) {
        self.lbl_switch_badge.set_text(text);
        let hwnd = self.lbl_switch_badge.hwnd();
        let wide: Vec<u16> = text.encode_utf16().collect();
        unsafe {
            let hdc = GetDC(hwnd);
            let old = SelectObject(hdc, SendMessageW(hwnd, WM_GETFONT, 0, 0));
            let mut sz = SIZE::default();
            GetTextExtentPoint32W(hdc, wide.as_ptr(), wide.len() as i32, &mut sz);
            SelectObject(hdc, old);
            ReleaseDC(hwnd, hdc);

            let w = (sz.cx + BADGE_PAD_X).clamp(BADGE_MIN_W, BADGE_MAX_W);
            SetWindowPos(hwnd, 0, 0, 0, w, BADGE_H, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
            // Repaint the pill border drawn by the parent around the label
            let rc = RECT { left: BADGE_X - 4, top: BADGE_Y - 8, right: BADGE_X + BADGE_MAX_W + 4, bottom: BADGE_Y + BADGE_H + 8 };
            InvalidateRect(self.parent, &rc, 0);
            InvalidateRect(hwnd, std::ptr::null(), 0);
        }
    }

    /// Loads global EngineConfig into controls
    pub fn load_config(&self, config: &EngineConfig) {
        let strings = crate::language::current();
        self.combo_mode
            .reset_items(&[strings.mode_vietnamese, strings.mode_english], Some(if config.enabled { 0 } else { 1 }));

        let method_idx = match config.method {
            InputMethod::Telex => 0,
            InputMethod::Vni => 1,
            InputMethod::SimpleTelex1 | InputMethod::SimpleTelex2 => 2,
        };
        self.combo_method
            .reset_items(&[strings.method_telex, strings.method_vni, strings.method_simple_telex], Some(method_idx));

        self.check_switch_key.set_checked(config.switch_key_enabled);
        self.update_switch_controls_state();

        self.set_badge_text(&config.switch_key.display_text());
        self.btn_switch_key.set_text(strings.typing_btn_change_key);

        self.check_restore_wrong
            .set_checked(config.restore_on_wrong_spelling);
        self.check_auto_upper
            .set_checked(config.auto_uppercase_first_char);
    }

    /// Loads Profile settings (handling Inherit options)
    pub fn load_profile(&self, profile: &AppProfile, global_config: &EngineConfig) {
        if profile.is_root {
            self.load_config(global_config);
        } else {
            let strings = crate::language::current();
            // Mode options for child profile
            let mode_items = [strings.typing_inherit_global, strings.mode_vietnamese, strings.mode_english];
            let mode_idx = match profile.mode {
                LanguageMode::Inherit => 0,
                LanguageMode::Vietnamese => 1,
                LanguageMode::English => 2,
            };
            self.combo_mode.reset_items(&mode_items, Some(mode_idx));

            // Method options for child profile
            let method_items = [
                strings.typing_inherit_global,
                strings.method_telex,
                strings.method_vni,
                strings.method_simple_telex,
            ];
            let method_idx = match profile.method {
                MethodMode::Inherit => 0,
                MethodMode::Specific(InputMethod::Telex) => 1,
                MethodMode::Specific(InputMethod::Vni) => 2,
                MethodMode::Specific(InputMethod::SimpleTelex1 | InputMethod::SimpleTelex2) => 3,
            };
            self.combo_method
                .reset_items(&method_items, Some(method_idx));

            // Switch key enabled checkbox (tri-state)
            self.check_switch_key.set_tri_state(
                profile.switch_key_enabled,
                global_config.switch_key_enabled,
            );
            self.update_switch_controls_state();

            // Hotkey switch badge
            let badge_text = match profile.switch_key {
                Some(ref k) => k.display_text(),
                None => format!("{} ({})", strings.typing_inherit_prefix, global_config.switch_key.display_text()),
            };
            self.set_badge_text(&badge_text);
            self.btn_switch_key.set_text(strings.typing_btn_change_key);

            self.check_restore_wrong.set_tri_state(
                profile.restore_on_wrong_spelling,
                global_config.restore_on_wrong_spelling,
            );
            self.check_auto_upper.set_tri_state(
                profile.auto_uppercase_first_char,
                global_config.auto_uppercase_first_char,
            );
        }
    }

    /// Saves controls back into an EngineConfig
    pub fn read_config(&self, config: &mut EngineConfig) {
        if let Some(mode_sel) = self.combo_mode.get_selected() {
            match mode_sel {
                0 => config.enabled = true,
                1 => config.enabled = false,
                _ => {}
            }
        }
        if let Some(method_sel) = self.combo_method.get_selected() {
            match method_sel {
                0 => config.method = InputMethod::Telex,
                1 => config.method = InputMethod::Vni,
                2 => config.method = simple_telex_keep_variant(config.method),
                _ => {}
            }
        }
        config.switch_key_enabled = self.check_switch_key.is_checked();
        config.restore_on_wrong_spelling = self.check_restore_wrong.is_checked();
        config.auto_uppercase_first_char = self.check_auto_upper.is_checked();
    }

    /// Saves controls back into an AppProfile
    pub fn read_profile(&self, profile: &mut AppProfile) {
        if profile.is_root {
            return;
        }
        if let Some(mode_sel) = self.combo_mode.get_selected() {
            match mode_sel {
                0 => profile.mode = LanguageMode::Inherit,
                1 => profile.mode = LanguageMode::Vietnamese,
                2 => profile.mode = LanguageMode::English,
                _ => {}
            }
        }
        if let Some(method_sel) = self.combo_method.get_selected() {
            match method_sel {
                0 => profile.method = MethodMode::Inherit,
                1 => profile.method = MethodMode::Specific(InputMethod::Telex),
                2 => profile.method = MethodMode::Specific(InputMethod::Vni),
                3 => {
                    let current = match profile.method {
                        MethodMode::Specific(m) => m,
                        MethodMode::Inherit => InputMethod::SimpleTelex1,
                    };
                    profile.method = MethodMode::Specific(simple_telex_keep_variant(current));
                }
                _ => {}
            }
        }
        profile.switch_key_enabled = self.check_switch_key.tri_state();
        profile.restore_on_wrong_spelling = self.check_restore_wrong.tri_state();
        profile.auto_uppercase_first_char = self.check_auto_upper.tri_state();
    }

    pub fn update_language(&self, strings: &LanguageStrings) {
        self.lbl_group_typing.set_text(strings.typing_group_main);
        self.lbl_mode.set_text(strings.typing_lbl_mode);
        self.lbl_method.set_text(strings.typing_lbl_method);
        self.check_switch_key.set_text(strings.typing_group_switch);
        self.lbl_switch_prompt.set_text(strings.typing_lbl_current_key);
        self.lbl_group_features.set_text(strings.typing_group_features);
        self.check_restore_wrong.set_text(strings.check_restore_wrong);
        self.check_auto_upper.set_text(strings.check_auto_upper);

        // Combo items are localized: reselect current values (engine lock released first).
        let snap = crate::ui::controller::with_engine(|e| {
            let pm = &e.profile_manager;
            (pm.get_profile(&pm.selected_id).cloned(), e.config().clone())
        });
        if let Some((Some(profile), cfg)) = snap {
            self.load_profile(&profile, &cfg);
        }
    }


    pub fn on_ctlcolor_static(&self, hdc: isize, ctrl_id: u32, is_dark: bool) -> Option<isize> {
        if ctrl_id == IDC_LABEL_SWITCH_KEY_BADGE {
            let palette = crate::ui::colors::ThemePalette::get(is_dark);
            unsafe {
                SetBkColor(hdc, palette.bg_input);
                SetTextColor(hdc, palette.accent);
            }
            let in_br = crate::ui::theme::get_input_bg_brush();
            if in_br != 0 {
                return Some(in_br);
            }
        }
        None
    }

    pub fn on_command(&self, hwnd: isize, ctrl_id: u32, notify_code: u32) -> bool {
        use crate::ui::controller::EditTarget;
        if ctrl_id == IDC_BTN_SWITCH_KEY {
            if self.btn_switch_key.is_enabled() {
                crate::ui::views::dialogs::show_hotkey_dialog(hwnd);
            }
            return true;
        }

        let is_combo = (ctrl_id == IDC_COMBO_TYPING_MODE || ctrl_id == IDC_COMBO_TYPING_METHOD)
            && notify_code == crate::ui::win32::CBN_SELCHANGE;
        let is_check = (ctrl_id == IDC_CHECK_RESTORE_WRONG
            || ctrl_id == IDC_CHECK_AUTO_UPPER
            || ctrl_id == IDC_CHECK_SWITCH_KEY)
            && notify_code == 0;

        if is_combo || is_check {
            let sw_on = self.check_switch_key.is_checked();
            if ctrl_id == IDC_CHECK_SWITCH_KEY {
                self.update_switch_controls_state();
            }
            crate::ui::controller::edit_selected_then(
                |target| match target {
                    EditTarget::Global(cfg) => self.read_config(cfg),
                    EditTarget::Child(child) => self.read_profile(child),
                },
                |engine| {
                    let pm = &mut engine.profile_manager;
                    if pm.selected_id == crate::ui::GLOBAL_PROFILE_ID {
                        pm.global_profile.switch_key_enabled = Some(sw_on);
                    }
                },
            );
            return true;
        }

        false
    }

    pub fn update_visibility(&self, visible: bool) {
        let cmd = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.lbl_group_typing.hwnd(), cmd);
            ShowWindow(self.lbl_mode.hwnd(), cmd);
            ShowWindow(self.combo_mode.hwnd(), cmd);
            ShowWindow(self.lbl_method.hwnd(), cmd);
            ShowWindow(self.combo_method.hwnd(), cmd);

            ShowWindow(self.check_switch_key.hwnd(), cmd);
            ShowWindow(self.lbl_switch_prompt.hwnd(), cmd);
            ShowWindow(self.lbl_switch_badge.hwnd(), cmd);
            ShowWindow(self.btn_switch_key.hwnd(), cmd);

            ShowWindow(self.lbl_group_features.hwnd(), cmd);
            ShowWindow(self.check_restore_wrong.hwnd(), cmd);
            ShowWindow(self.check_auto_upper.hwnd(), cmd);
        }
    }
}
