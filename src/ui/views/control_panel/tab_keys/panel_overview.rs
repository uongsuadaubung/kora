//! Sub-panel: Key Customization Overview & Conflict Diagnostics (Tổng quan & Chẩn đoán phím)
//!
//! Powered by the Declarative Single-HWND UI Engine (`Surface<OverviewMsg>`),
//! rendering grouped shortcut cards (`card_grouped`), automatic conflict alerts (`alert`),
//! inline rule mute/unmute checkboxes, and 1-click sub-tab/dialog navigation.

use crate::engine::config::Hotkey;
use crate::engine::config::hotkey::vk_to_name;
use crate::engine::profile::{
    ConflictFeature, KeyDiagnostics, ResolvedDisableItem, ResolvedKeyMacroItem, ResolvedRemapItem,
    RuleOrigin, check_profile_conflicts,
};
use crate::language::LanguageStrings;
use crate::ui::views::control_panel::components::{ShortcutRowProps, shortcut_row};
use crate::ui::views::control_panel::tab_keys::SUB_CARD_Y;
use crate::ui_framework::core::*;
use crate::ui_framework::tokens::layout;
use crate::ui_framework::win32::{InvalidateRect, SW_HIDE, SetWindowPos, ShowWindow, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER};
use std::ptr::null_mut;

#[derive(Clone, Debug)]
pub enum OverviewMsg {
    ToggleSwitchKey,
    OpenSwitchHotkeyDialog,
    ToggleRemapMuted(u32, bool),
    OpenRemapDialog(u32, u32),
    ToggleDisableMuted(u32, bool),
    OpenDisableDialog,
    ToggleKeyMacroMuted(u32, bool),
    OpenKeyMacroDialog(u32),
    SwitchSubTab(usize),
}

pub struct OverviewPanel {
    surface: Surface<OverviewMsg>,
    hwnd_slice: [isize; 1],
}

impl OverviewPanel {
    pub fn create(parent: isize, _hfont_normal: isize, _hfont_bold: isize) -> Option<Self> {
        let viewport_x = layout::CARD_X;
        let viewport_y = SUB_CARD_Y;
        let viewport_w = layout::CARD_WIDTH + 8;
        let viewport_h = layout::CARD_BOTTOM_Y - SUB_CARD_Y;

        let surface = Surface::create(parent, viewport_x, viewport_y, viewport_w, viewport_h)?;
        let hwnd = surface.hwnd();
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
            SetWindowPos(
                hwnd,
                0,
                viewport_x,
                viewport_y,
                viewport_w,
                viewport_h,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            );
        }

        surface.set_on_msg(move |msg| {
            Self::handle_message(parent, msg);
        });

        Some(Self {
            surface,
            hwnd_slice: [hwnd],
        })
    }

    pub fn hwnds(&self) -> &[isize] {
        &self.hwnd_slice
    }

    pub fn viewport_hwnd(&self) -> isize {
        self.surface.hwnd()
    }

    pub fn invalidate(&self) {
        self.surface.invalidate();
    }

    pub fn update_language(&self, _strings: &LanguageStrings) {
        self.surface.invalidate();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn populate(
        &self,
        remaps: &[ResolvedRemapItem],
        remap_enabled: bool,
        disabled: &[ResolvedDisableItem],
        disable_enabled: bool,
        key_macros: &[ResolvedKeyMacroItem],
        key_macro_enabled: bool,
        switch_key: Hotkey,
        switch_key_enabled: bool,
        switch_key_origin: RuleOrigin,
    ) -> KeyDiagnostics {
        let strings = crate::language::current();
        let diag = check_profile_conflicts(
            remaps,
            remap_enabled,
            disabled,
            disable_enabled,
            key_macros,
            key_macro_enabled,
            switch_key,
            switch_key_enabled,
        );

        let tree = Self::view(
            strings,
            &diag,
            remaps,
            remap_enabled,
            disabled,
            disable_enabled,
            key_macros,
            key_macro_enabled,
            switch_key,
            switch_key_enabled,
            switch_key_origin,
        );
        self.surface.render(tree);
        diag
    }

    #[allow(clippy::too_many_arguments)]
    fn view(
        strings: &LanguageStrings,
        diag: &KeyDiagnostics,
        remaps: &[ResolvedRemapItem],
        remap_enabled: bool,
        disabled: &[ResolvedDisableItem],
        disable_enabled: bool,
        key_macros: &[ResolvedKeyMacroItem],
        key_macro_enabled: bool,
        switch_key: Hotkey,
        switch_key_enabled: bool,
        switch_key_origin: RuleOrigin,
    ) -> Element<OverviewMsg> {
        let mut sections: Vec<Element<OverviewMsg>> = Vec::new();

        // 1. Conflict Alert Banner (automatically expands and pushes cards down when conflicts exist)
        if !diag.is_healthy() {
            let count = diag.conflicts.len();
            let title = strings
                .overview_conflicts_format
                .replace("{count}", &count.to_string());
            let details: Vec<String> = diag
                .conflicts
                .iter()
                .map(|c| {
                    let key_name = vk_to_name(c.vk);
                    let k = if key_name.is_empty() {
                        format!("VK {:#04X}", c.vk)
                    } else {
                        key_name.to_string()
                    };
                    format!(
                        "• [{}] {}: {}",
                        k,
                        c.shadowed.localized_name(strings),
                        strings
                            .status_shadowed_by
                            .replace("{feature}", c.winner.localized_name(strings))
                    )
                })
                .collect();
            sections.push(alert(Variant::Warning, title, details.join("\n")));
        }

        let origin_badge_for = |origin: RuleOrigin| -> (String, Variant) {
            match origin {
                RuleOrigin::Root => (strings.origin_root.to_string(), Variant::Secondary),
                RuleOrigin::InheritedGlobal => (
                    origin.localized_badge_label(strings).to_string(),
                    Variant::Info,
                ),
                RuleOrigin::AppSpecific | RuleOrigin::OverriddenGlobal => (
                    origin.localized_badge_label(strings).to_string(),
                    Variant::Purple,
                ),
                RuleOrigin::MutedGlobal => (
                    origin.localized_badge_label(strings).to_string(),
                    Variant::Secondary,
                ),
            }
        };

        // 2. Group 1: Mode Switch Hotkey (Phím tắt Bộ gõ)
        let switch_conflict = diag
            .conflicts
            .iter()
            .find(|c| c.shadowed == ConflictFeature::SwitchKey);
        let switch_status = if let Some(c) = switch_conflict {
            (
                strings
                    .status_shadowed_by
                    .replace("{feature}", c.winner.localized_name(strings)),
                Variant::Warning,
            )
        } else if switch_key_enabled {
            (strings.status_active.to_string(), Variant::Success)
        } else {
            (strings.status_disabled.to_string(), Variant::Secondary)
        };

        let group1_header = row(elements![
            row(elements![
                text(format!("⌨️ {}", strings.feature_switch_key)).bold(),
                badge("1", Variant::Primary),
            ])
            .items_center()
            .gap(2)
            .flex_1(),
            badge(
                if diag.is_healthy() {
                    strings.overview_healthy.to_string()
                } else {
                    strings
                        .overview_conflicts_format
                        .replace("{count}", &diag.conflicts.len().to_string())
                },
                if diag.is_healthy() {
                    Variant::Success
                } else {
                    Variant::Warning
                },
            ),
        ])
        .items_center()
        .justify_between();

        let switch_row = shortcut_row(ShortcutRowProps {
            title: strings.typing_group_switch.to_string(),
            desc: strings.action_toggle_vietnamese.to_string(),
            checked: Some((switch_key_enabled, OverviewMsg::ToggleSwitchKey)),
            status_badge: Some(switch_status),
            origin_badge: Some(origin_badge_for(switch_key_origin)),
            key_pill: switch_key.display_text(),
            key_variant: if switch_conflict.is_some() {
                Variant::Warning
            } else if switch_key_enabled {
                Variant::Primary
            } else {
                Variant::Secondary
            },
            is_conflict: switch_conflict.is_some(),
            action_btn: Some((
                strings.typing_btn_change_key.to_string(),
                Variant::Secondary,
                OverviewMsg::OpenSwitchHotkeyDialog,
            )),
        });
        sections.push(card_grouped(group1_header, vec![switch_row]));

        // 3. Group 2: Key Remap (Đổi phím)
        let group2_header = row(elements![
            row(elements![
                text(format!("🔄 {}", strings.feature_remap)).bold(),
                badge(remaps.len().to_string(), Variant::Primary),
            ])
            .items_center()
            .gap(2)
            .flex_1(),
            button(
                format!("{} ➜", strings.keys_sub_remap),
                Variant::Ghost,
                OverviewMsg::SwitchSubTab(1),
            )
            .size(Size::Sm),
        ])
        .items_center()
        .justify_between();

        let mut remap_rows: Vec<Element<OverviewMsg>> = Vec::new();
        if remaps.is_empty() {
            remap_rows.push(
                row(elements![text(strings.status_disabled).muted().size(Size::Sm)])
                    .px(3)
                    .py(2),
            );
        } else {
            for (idx, r) in remaps.iter().enumerate() {
                if idx > 0 {
                    remap_rows.push(divider());
                }
                let shadowed = diag.is_shadowed(r.from_vk, ConflictFeature::Remap);
                let status = if let Some(c) = shadowed {
                    (
                        strings
                            .status_shadowed_by
                            .replace("{feature}", c.winner.localized_name(strings)),
                        Variant::Warning,
                    )
                } else if r.is_active && remap_enabled {
                    (strings.status_active.to_string(), Variant::Success)
                } else {
                    (strings.status_disabled.to_string(), Variant::Secondary)
                };

                remap_rows.push(shortcut_row(ShortcutRowProps {
                    title: format!("{} ➜ {}", r.from_name, r.to_name),
                    desc: strings.feature_remap.to_string(),
                    checked: Some((
                        r.is_active && remap_enabled,
                        OverviewMsg::ToggleRemapMuted(r.from_vk, r.is_active),
                    )),
                    status_badge: Some(status),
                    origin_badge: Some(origin_badge_for(r.origin)),
                    key_pill: format!("{} ➜ {}", r.from_name, r.to_name),
                    key_variant: if shadowed.is_some() {
                        Variant::Warning
                    } else if r.is_active && remap_enabled {
                        Variant::Primary
                    } else {
                        Variant::Secondary
                    },
                    is_conflict: shadowed.is_some(),
                    action_btn: Some((
                        strings.typing_btn_change_key.to_string(),
                        Variant::Secondary,
                        OverviewMsg::OpenRemapDialog(r.from_vk, r.to_vk),
                    )),
                }));
            }
        }
        sections.push(card_grouped(group2_header, remap_rows));

        // 4. Group 3: Key Disabling (Khóa phím)
        let group3_header = row(elements![
            row(elements![
                text(format!("🚫 {}", strings.feature_disable)).bold(),
                badge(disabled.len().to_string(), Variant::Primary),
            ])
            .items_center()
            .gap(2)
            .flex_1(),
            button(
                format!("{} ➜", strings.keys_sub_disable),
                Variant::Ghost,
                OverviewMsg::SwitchSubTab(2),
            )
            .size(Size::Sm),
        ])
        .items_center()
        .justify_between();

        let mut disable_rows: Vec<Element<OverviewMsg>> = Vec::new();
        if disabled.is_empty() {
            disable_rows.push(
                row(elements![text(strings.status_disabled).muted().size(Size::Sm)])
                    .px(3)
                    .py(2),
            );
        } else {
            for (idx, d) in disabled.iter().enumerate() {
                if idx > 0 {
                    disable_rows.push(divider());
                }
                let is_on = d.is_active && disable_enabled;
                let status = if is_on {
                    (strings.status_active.to_string(), Variant::Success)
                } else {
                    (strings.status_disabled.to_string(), Variant::Secondary)
                };

                disable_rows.push(shortcut_row(ShortcutRowProps {
                    title: d.key_name.clone(),
                    desc: strings.status_blocked.to_string(),
                    checked: Some((is_on, OverviewMsg::ToggleDisableMuted(d.vk, d.is_active))),
                    status_badge: Some(status),
                    origin_badge: Some(origin_badge_for(d.origin)),
                    key_pill: format!("🚫 {}", d.key_name),
                    key_variant: if is_on {
                        Variant::Danger
                    } else {
                        Variant::Secondary
                    },
                    is_conflict: false,
                    action_btn: Some((
                        strings.keys_sub_disable.to_string(),
                        Variant::Secondary,
                        OverviewMsg::OpenDisableDialog,
                    )),
                }));
            }
        }
        sections.push(card_grouped(group3_header, disable_rows));

        // 5. Group 4: Key Macros (Phím Macro)
        let group4_header = row(elements![
            row(elements![
                text(format!("⚡ {}", strings.feature_key_macro)).bold(),
                badge(key_macros.len().to_string(), Variant::Primary),
            ])
            .items_center()
            .gap(2)
            .flex_1(),
            button(
                format!("{} ➜", strings.keys_sub_macro),
                Variant::Ghost,
                OverviewMsg::SwitchSubTab(3),
            )
            .size(Size::Sm),
        ])
        .items_center()
        .justify_between();

        let mut macro_rows: Vec<Element<OverviewMsg>> = Vec::new();
        if key_macros.is_empty() {
            macro_rows.push(
                row(elements![text(strings.status_disabled).muted().size(Size::Sm)])
                    .px(3)
                    .py(2),
            );
        } else {
            for (idx, m) in key_macros.iter().enumerate() {
                if idx > 0 {
                    macro_rows.push(divider());
                }
                let shadowed = diag.is_shadowed(m.trigger_vk, ConflictFeature::KeyMacro);
                let is_on = m.is_active && key_macro_enabled;
                let status = if let Some(c) = shadowed {
                    (
                        strings
                            .status_shadowed_by
                            .replace("{feature}", c.winner.localized_name(strings)),
                        Variant::Warning,
                    )
                } else if is_on {
                    (strings.status_active.to_string(), Variant::Success)
                } else {
                    (strings.status_disabled.to_string(), Variant::Secondary)
                };

                let trigger_display = if !m.trigger_name.is_empty() {
                    m.trigger_name.clone()
                } else {
                    let name = vk_to_name(m.trigger_vk);
                    if !name.is_empty() {
                        name.to_string()
                    } else {
                        format!("VK {:#04X}", m.trigger_vk)
                    }
                };

                let desc = if let Some(c) = shadowed {
                    strings
                        .status_shadowed_by
                        .replace("{feature}", c.winner.localized_name(strings))
                } else {
                    strings.feature_key_macro.to_string()
                };

                macro_rows.push(shortcut_row(ShortcutRowProps {
                    title: m.name.clone(),
                    desc,
                    checked: Some((
                        is_on,
                        OverviewMsg::ToggleKeyMacroMuted(m.trigger_vk, m.is_active),
                    )),
                    status_badge: Some(status),
                    origin_badge: Some(origin_badge_for(m.origin)),
                    key_pill: format!("⚡ {}", trigger_display),
                    key_variant: if shadowed.is_some() {
                        Variant::Warning
                    } else if is_on {
                        Variant::Primary
                    } else {
                        Variant::Secondary
                    },
                    is_conflict: shadowed.is_some(),
                    action_btn: Some((
                        strings.typing_btn_change_key.to_string(),
                        if shadowed.is_some() {
                            Variant::Warning
                        } else {
                            Variant::Secondary
                        },
                        OverviewMsg::OpenKeyMacroDialog(m.trigger_vk),
                    )),
                }));
            }
        }
        sections.push(card_grouped(group4_header, macro_rows));

        col(sections).gap(3).pb(3)
    }

    fn handle_message(parent_hwnd: isize, msg: OverviewMsg) {
        use crate::ui::controller::{self, EditTarget};

        match msg {
            OverviewMsg::ToggleSwitchKey => {
                controller::edit_selected(|target| match target {
                    EditTarget::Global(cfg) => {
                        cfg.switch_key_enabled = !cfg.switch_key_enabled;
                    }
                    EditTarget::Child(prof) => {
                        let cur = prof.switch_key_enabled.unwrap_or(true);
                        prof.switch_key_enabled = Some(!cur);
                    }
                });
                if let Some(c) = crate::ui::controls() {
                    c.tab_typing.refresh_from_engine();
                }
                controller::refresh_keys_overview();
            }
            OverviewMsg::OpenSwitchHotkeyDialog => {
                crate::ui::views::dialogs::show_hotkey_dialog(parent_hwnd);
            }
            OverviewMsg::ToggleRemapMuted(from_vk, currently_active) => {
                controller::set_remap_muted(from_vk, currently_active);
            }
            OverviewMsg::OpenRemapDialog(from_vk, to_vk) => {
                crate::ui::views::dialogs::show_remap_dialog_with_keys(
                    parent_hwnd,
                    Some(from_vk),
                    Some(to_vk),
                );
            }
            OverviewMsg::ToggleDisableMuted(vk, currently_active) => {
                controller::set_disable_muted(vk, currently_active);
            }
            OverviewMsg::OpenDisableDialog => {
                crate::ui::views::dialogs::show_disable_dialog(parent_hwnd);
            }
            OverviewMsg::ToggleKeyMacroMuted(trig_vk, currently_active) => {
                controller::set_key_macro_muted(trig_vk, currently_active);
            }
            OverviewMsg::OpenKeyMacroDialog(trig_vk) => {
                crate::ui::views::dialogs::show_key_macro_dialog_with_key(
                    parent_hwnd,
                    Some(trig_vk),
                );
            }
            OverviewMsg::SwitchSubTab(sub_idx) => {
                if let Some(c) = crate::ui::controls() {
                    c.tab_keys.sub_tabs.set_cur_sel(sub_idx);
                    c.tab_keys.update_visibility(true);
                    unsafe {
                        InvalidateRect(parent_hwnd, null_mut(), 0);
                    }
                }
            }
        }
    }
}
