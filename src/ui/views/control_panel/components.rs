//! Kora Declarative Functional Components (`src/ui/views/control_panel/components.rs`)
//!
//! Pure React-style functional components (`fn(Props) -> Element<Msg>`) built on top of
//! the project-agnostic `crate::ui_framework::core` engine.

use crate::engine::profile::{KeyInjectionStrategy, TargetRule};
use crate::language::LanguageStrings;
use crate::ui_framework::core::*;

/// Renders a bold accent section header title inside a card.
pub fn section_header<M: Clone + 'static>(title: impl Into<String>) -> Element<M> {
    text(title).bold().accent()
}

/// Renders a top-of-page header block (Title + Subtitle) for global views (`System`, `About`).
pub fn page_header<M: Clone + 'static>(
    title: impl Into<String>,
    subtitle: impl Into<String>,
) -> Element<M> {
    col(vec![
        text(title).bold().size(Size::Lg),
        text(subtitle).muted(),
    ])
    .gap(1)
    .pb(2)
}

/// Renders a form field grouping a bold label with an arbitrary child control (`select`, `slider`, etc.).
/// Completely un-opinionated: does not enforce width or flex on either label or control.
pub fn form_group<M: Clone + 'static>(
    lbl: impl Into<String>,
    control: Element<M>,
) -> Element<M> {
    col(vec![
        label(lbl).bold(),
        control,
    ])
    .gap(1)
}

/// Renders a labeled vertical select field (`Label` + `Select` dropdown).
/// Natural content-sized by default: does not enforce `.w_full()` or `.flex_1()`.
pub fn form_select<M: Clone + 'static>(
    label: impl Into<String>,
    options: Vec<String>,
    selected: usize,
    on_change: impl Fn(usize) -> M + Send + Sync + 'static,
) -> Element<M> {
    form_group(label, select(options, selected, on_change))
}

/// Computes the next tri-state value when cycling an inheritable setting in a child profile,
/// or toggling a boolean setting in the root profile.
pub fn next_inheritable_state(
    is_root: bool,
    override_val: Option<bool>,
    global_val: bool,
) -> Option<bool> {
    if is_root {
        Some(!global_val)
    } else {
        match override_val {
            None => Some(!global_val),
            Some(v) if v != global_val => Some(global_val),
            Some(_) => None,
        }
    }
}

/// Renders a setting row with a modern toggle `switch`, optional subtitle, and automatic
/// inheritance badge (`Kế thừa: BẬT/TẮT` vs `Riêng ứng dụng`) when viewing a Child profile.
pub fn inheritable_switch_row<M: Clone + Send + Sync + 'static>(
    label: impl Into<String>,
    desc: Option<String>,
    is_root: bool,
    override_val: Option<bool>,
    global_val: bool,
    strings: &LanguageStrings,
    on_toggle: impl Fn(Option<bool>) -> M + Send + Sync + 'static,
) -> Element<M> {
    let effective = override_val.unwrap_or(global_val);
    let next_val = next_inheritable_state(is_root, override_val, global_val);
    let msg = on_toggle(next_val);

    let mut title_row = vec![text(label).bold()];
    if !is_root {
        let (badge_txt, badge_var) = match override_val {
            None => (
                if global_val {
                    strings.check_inherit_on
                } else {
                    strings.check_inherit_off
                },
                Variant::Secondary,
            ),
            Some(_) => (strings.origin_app_specific, Variant::Purple),
        };
        title_row.push(badge(badge_txt, badge_var));
    }

    let mut left_col = vec![row(title_row).items_center().gap(2)];
    if let Some(d) = desc
        && !d.is_empty()
    {
        left_col.push(text(d).muted().size(Size::Sm));
    }

    row(vec![
        col(left_col).gap(1).flex_1(),
        switch("", effective, move |_| msg.clone()),
    ])
    .items_center()
    .justify_between()
    .gap(3)
}

/// Properties for a reusable shortcut / key-rule row inside `OverviewPanel` or `TabTyping`.
pub struct ShortcutRowProps<M> {
    pub title: String,
    pub desc: String,
    pub checked: Option<(bool, M)>,
    pub status_badge: Option<(String, Variant)>,
    pub origin_badge: Option<(String, Variant)>,
    pub key_pill: String,
    pub key_variant: Variant,
    pub is_conflict: bool,
    pub action_btn: Option<(String, Variant, M)>,
}

/// Renders a structured key rule / shortcut row with left toggle/title/description and right key pill + action button.
pub fn shortcut_row<M: Clone + Send + Sync + 'static>(props: ShortcutRowProps<M>) -> Element<M> {
    let mut header_items: Vec<Element<M>> = Vec::new();

    if let Some((is_on, toggle_msg)) = props.checked {
        header_items.push(checkbox(props.title, is_on, move |_| toggle_msg.clone()));
    } else {
        header_items.push(text(props.title).bold());
    }

    if let Some((b_txt, b_var)) = props.origin_badge {
        header_items.push(badge(b_txt, b_var));
    }
    if let Some((s_txt, s_var)) = props.status_badge {
        header_items.push(badge(s_txt, s_var));
    }

    let desc_el = if props.is_conflict {
        text(props.desc).warning().size(Size::Sm)
    } else {
        text(props.desc).muted().size(Size::Sm)
    };

    let mut right_items: Vec<Element<M>> = vec![badge(props.key_pill, props.key_variant)];
    if let Some((btn_label, btn_var, btn_msg)) = props.action_btn {
        right_items.push(button(btn_label, btn_var, btn_msg).size(Size::Sm));
    }

    let r = row(vec![
        col(vec![
            row(header_items).items_center().gap(2),
            desc_el,
        ])
        .gap(1)
        .flex_1(),
        row(right_items).items_center().gap(2),
    ])
    .items_center()
    .justify_between()
    .gap(3)
    .px(3)
    .py(2);

    if props.is_conflict {
        r.bg_subtle(Variant::Warning)
    } else {
        r
    }
}

/// Renders a single Target Rule card inside `TabTyping` Card 4 (Child profile).
pub fn target_rule_card<M: Clone + Send + Sync + 'static>(
    index: usize,
    rule: &TargetRule,
    strings: &LanguageStrings,
    on_strategy: impl Fn(usize) -> M + Send + Sync + 'static,
    on_delete: M,
) -> Element<M> {
    let strategy_idx = match rule.strategy {
        KeyInjectionStrategy::AutocompleteGuard => 0,
        KeyInjectionStrategy::ClipboardPaste => 1,
    };
    let desc_text = match rule.strategy {
        KeyInjectionStrategy::AutocompleteGuard => strings.strategy_desc_autocomplete_guard,
        KeyInjectionStrategy::ClipboardPaste => strings.strategy_desc_clipboard_paste,
    };
    let strategy_options = vec![
        strings.strategy_autocomplete_guard.to_string(),
        strings.strategy_clipboard_paste.to_string(),
    ];

    col(vec![
        row(vec![
            row(vec![
                text(format!("#{}", index + 1)).bold().accent(),
                badge(rule.target_id.clone(), Variant::Primary),
            ])
            .items_center()
            .gap(2)
            .flex_1(),
            button("✕", Variant::Danger, on_delete).size(Size::Sm).w(30),
        ])
        .items_center()
        .justify_between(),
        col(vec![
            row(vec![
                text(strings.typing_target_method_label).muted().w(115),
                select(strategy_options, strategy_idx, on_strategy).flex_1(),
            ])
            .items_center()
            .gap(2),
            text(desc_text).muted().size(Size::Sm),
        ])
        .p(2)
        .gap(2)
        .bg_card()
        .border()
        .rounded(6),
    ])
    .p(3)
    .gap(2)
    .bg_window()
    .border()
    .rounded_card()
}

