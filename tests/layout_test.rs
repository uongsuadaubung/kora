#![cfg(windows)]

use kora::ui_framework::layout::*;

#[test]
fn test_hstack_fixed_and_spacer() {
    let mut stack = HStack::at(0, 0, 300, 30);
    stack.add_fixed(50, None);
    stack.add_spacer();
    stack.add_fixed(80, None);
    let rects = stack.apply();
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0], Rect::new(0, 0, 50, 30));
    assert_eq!(rects[1], Rect::new(50, 0, 170, 30));
    assert_eq!(rects[2], Rect::new(220, 0, 80, 30));
}

#[test]
fn test_hstack_stretch_proportions() {
    let mut stack = HStack::at(10, 20, 100, 40);
    stack.add_stretch(1, None);
    stack.add_stretch(3, None);
    let rects = stack.apply();
    assert_eq!(rects[0], Rect::new(10, 20, 25, 40));
    assert_eq!(rects[1], Rect::new(35, 20, 75, 40));
}

#[test]
fn test_hstack_spacing_and_padding() {
    let mut stack = HStack::at(0, 0, 200, 50);
    stack.spacing(10).padding(Padding::all(5));
    stack.add_fixed(40, None);
    stack.add_fixed(40, None);
    let rects = stack.apply();
    assert_eq!(rects[0], Rect::new(5, 5, 40, 40));
    assert_eq!(rects[1], Rect::new(55, 5, 40, 40));
}

#[test]
fn test_declarative_core_flex_row_and_col() {
    use kora::ui_framework::core::*;

    let tree: Element<()> = col(vec![
        row(vec![
            button("Left", Variant::Primary, ()).w(100),
            spacer(),
            button("Right", Variant::Secondary, ()).w(80),
        ])
        .items_center(),
        card(vec![
            text("Header").bold(),
            text("A long paragraph of text that wraps across multiple lines when constrained to a narrow width so we can verify automatic height calculation."),
        ]),
    ])
    .gap(3);

    let root = compute_layout(0, tree, 0, 0, 400);
    assert_eq!(root.rect.left, 0);
    assert_eq!(root.rect.right, 400);
    assert_eq!(root.children.len(), 2);

    // Row child: Left button at 0..100, Right button at 320..400
    let row_node = &root.children[0];
    assert_eq!(row_node.children.len(), 3);
    assert_eq!(row_node.children[0].rect.left, 0);
    assert_eq!(row_node.children[0].rect.right, 100);
    assert_eq!(row_node.children[2].rect.left, 320);
    assert_eq!(row_node.children[2].rect.right, 400);

    // Card child pushed down by row height + gap(3) (12px)
    let card_node = &root.children[1];
    assert_eq!(card_node.rect.top, row_node.rect.bottom + 12);
    assert!(card_node.rect.bottom - card_node.rect.top > 50);
}

#[test]
fn test_declarative_core_dynamic_alert_insertion_shifts_cards() {
    use kora::ui_framework::core::*;

    let without_alert: Element<()> = col(vec![card(vec![text("Group 1").bold()])]).gap(3);
    let layout_clean = compute_layout(0, without_alert, 0, 0, 500);
    let group1_top_clean = layout_clean.children[0].rect.top;
    assert_eq!(group1_top_clean, 0);

    let with_alert: Element<()> = col(vec![
        alert(
            Variant::Warning,
            "Conflict Detected",
            "Key F1 is assigned to both Disable and Macro.",
        ),
        card(vec![text("Group 1").bold()]),
    ])
    .gap(3);
    let layout_conflict = compute_layout(0, with_alert, 0, 0, 500);
    let alert_bottom = layout_conflict.children[0].rect.bottom;
    let group1_top_conflict = layout_conflict.children[1].rect.top;

    assert!(alert_bottom > 30);
    assert_eq!(group1_top_conflict, alert_bottom + 12);
}

#[test]
fn test_declarative_core_grid_equal_columns() {
    use kora::ui_framework::core::*;

    let tree: Element<()> = grid(
        2,
        vec![
            card(vec![text("Col 1")]),
            card(vec![text("Col 2")]),
        ],
    );
    let root = compute_layout(0, tree, 0, 0, 512);
    assert_eq!(root.children.len(), 2);
    // 512 width with gap(3)=12 -> each column is (512 - 12) / 2 = 250px
    assert_eq!(root.children[0].rect.left, 0);
    assert_eq!(root.children[0].rect.right, 250);
    assert_eq!(root.children[1].rect.left, 262);
    assert_eq!(root.children[1].rect.right, 512);
}

#[test]
fn test_declarative_core_bootstrap_col_proportions() {
    use kora::ui_framework::core::*;

    // row width = 600px, gap(0) = 0px
    // col_8 (8/12 = 400px) + col_4 (4/12 = 200px)
    let tree: Element<()> = row(vec![
        col(vec![text("Main Content")]).col_8(),
        col(vec![text("Sidebar")]).col_4(),
    ])
    .gap(0);

    let root = compute_layout(0, tree, 0, 0, 600);
    assert_eq!(root.children.len(), 2);
    assert_eq!(root.children[0].rect.left, 0);
    assert_eq!(root.children[0].rect.right, 400);
    assert_eq!(root.children[1].rect.left, 400);
    assert_eq!(root.children[1].rect.right, 600);
}

#[test]
fn test_declarative_core_select_full_column_width() {
    use kora::ui_framework::core::*;

    // 3 equal columns in a 480px row with gap(0) -> each col is exactly 160px
    let tree: Element<()> = row(vec![
        col(vec![
            text("Col 1").bold(),
            select(vec!["Option A", "Option B"], 0, |_| ()).w_full(),
        ]).flex_1(),
        col(vec![
            text("Col 2").bold(),
            select(vec!["Opt 1", "Opt 2"], 0, |_| ()).w_full(),
        ]).flex_1(),
        col(vec![
            text("Col 3").bold(),
            select(vec!["Unicode", "TCVN3"], 0, |_| ()).w_full(),
        ]).flex_1(),
    ])
    .gap(0);

    let root = compute_layout(0, tree, 0, 0, 480);
    assert_eq!(root.children.len(), 3);

    // Each column must be 160px wide
    for (i, col_node) in root.children.iter().enumerate() {
        assert_eq!(col_node.rect.left, (i as i32) * 160);
        assert_eq!(col_node.rect.right, ((i as i32) + 1) * 160);

        // And the select widget inside each column must stretch to the full 160px width
        let select_node = &col_node.children[1];
        assert_eq!(select_node.rect.left, (i as i32) * 160);
        assert_eq!(select_node.rect.right, ((i as i32) + 1) * 160);
        assert_eq!(select_node.rect.right - select_node.rect.left, 160);
    }
}

#[test]
fn test_switch_row_button_alignment() {
    use kora::ui_framework::core::*;

    let tree: Element<()> = card(vec![
        row(vec![
            col(vec![text("Phím chuyển đổi E ⇄ V").bold()]).flex_1(),
            switch("", true, |_| ()),
        ])
        .items_center()
        .justify_between(),
        row(vec![
            text("Phím tắt hiện tại:").muted().flex_1(),
            row(vec![
                badge("Ctrl + Shift", Variant::Primary),
                button("Đổi phím", Variant::Primary, ()).size(Size::Sm),
            ])
            .items_center()
            .gap(2),
        ])
        .items_center()
        .justify_between()
        .p(2)
        .bg_window()
        .border()
        .rounded(6),
    ]);

    let root = compute_layout(0, tree, 0, 0, 560);
    // Card child 1 is the inner gray box
    let gray_box = &root.children[1];
    let inner_row = &gray_box.children[1];
    let change_btn = &inner_row.children[1];

    // The inner gray box's right boundary is 548 (card width 560 minus 12px card padding)
    assert_eq!(gray_box.rect.right, 548);
    // The "Đổi phím" button must be flush with the inner right boundary of the gray box (548 - 8px p(2) = 540px)
    assert_eq!(change_btn.rect.right, 540);
}

#[test]
fn test_footer_surface_layout() {
    use kora::ui_framework::core::*;

    #[derive(Clone, Debug)]
    enum FooterMsg {
        Exit,
        Close,
    }

    let tree: Element<FooterMsg> = row(vec![
        button("Thoát Kora", Variant::Danger, FooterMsg::Exit).w(114),
        spacer(),
        button("Đóng", Variant::Primary, FooterMsg::Close).w(112),
    ])
    .items_center()
    .w_full();

    let root = compute_layout(0, tree, 0, 0, 562);
    assert_eq!(root.children.len(), 3);
    // Exit button
    assert_eq!(root.children[0].rect.left, 0);
    assert_eq!(root.children[0].rect.right, 114);
    // Close button (flush with right edge: 562 - 112 = 450)
    assert_eq!(root.children[2].rect.left, 450);
    assert_eq!(root.children[2].rect.right, 562);
}

#[test]
fn test_sidebar_column_flex_layout() {
    use kora::ui_framework::core::*;

    let tree: Element<()> = col(vec![
        row(vec![text("Header")]).h(28),
        col(vec![text("Item 1").h(48), text("Item 2").h(48)]),
        spacer().flex_1(),
        row(vec![text("Footer")]).h(32),
    ])
    .h(500);

    let root = compute_layout(0, tree, 0, 0, 200);
    assert_eq!(root.children.len(), 4);
    // Header
    assert_eq!(root.children[0].rect.top, 0);
    assert_eq!(root.children[0].rect.bottom, 28);
    // Profile items
    assert_eq!(root.children[1].rect.top, 28);
    assert_eq!(root.children[1].rect.bottom, 28 + 96);
    // Spacer takes 500 - 28 - 96 - 32 = 344
    assert_eq!(root.children[2].rect.top, 124);
    assert_eq!(root.children[2].rect.bottom, 468);
    // Footer button pinned at bottom
    assert_eq!(root.children[3].rect.top, 468);
    assert_eq!(root.children[3].rect.bottom, 500);
}

#[test]
fn test_declarative_label_layout_and_modifiers() {
    use kora::ui_framework::core::*;

    let tree: Element<i32> = row(vec![
        label("Bảng mã").bold(),
        label("Tùy chỉnh nâng cao").accent().on_click(42),
    ])
    .gap(12);

    let root = compute_layout(0, tree, 0, 0, 400);
    assert_eq!(root.children.len(), 2);
    assert!(root.children[0].rect.right > root.children[0].rect.left);
    assert_eq!(root.children[0].rect.bottom - root.children[0].rect.top, 16);
    assert!(root.children[1].rect.right > root.children[1].rect.left);
    assert_eq!(root.children[1].rect.left, root.children[0].rect.right + 12);
    assert_eq!(root.children[1].element.on_click, Some(42));
}

#[test]
fn test_declarative_tab_group_layout_and_item_hit_targets() {
    use kora::ui_framework::core::*;

    let tree: Element<usize> = tab_group(
        vec!["Phím gõ", "Phím bấm", "Âm thanh", "Gõ tắt"],
        1,
        |idx| idx,
    );

    let root = compute_layout(0, tree, 0, 0, 562);
    assert_eq!(root.rect.left, 0);
    assert_eq!(root.rect.right, 562);
    assert_eq!(root.rect.bottom - root.rect.top, 32);

    // 4 tab items laid out as children
    assert_eq!(root.children.len(), 4);
    for (i, child) in root.children.iter().enumerate() {
        assert_eq!(child.element.on_click, Some(i));
        assert!(child.rect.left >= 3);
        assert!(child.rect.right <= 562);
        assert!(child.rect.right > child.rect.left);
    }
}

#[test]
fn test_modal_confirm_declarative_layout() {
    use kora::ui::views::dialogs::confirm::{render_confirm_view, ConfirmDialogOptions};
    use kora::ui_framework::core::*;

    let options = ConfirmDialogOptions {
        title: "Xác nhận xóa".to_string(),
        message: "Bạn có chắc chắn muốn bỏ chặn phím Left Windows?".to_string(),
        yes_text: "Đồng ý".to_string(),
        no_text: "Không".to_string(),
        is_danger: true,
    };

    let view = render_confirm_view(&options, 125);

    let root = compute_layout(0, view, 0, 0, 390);
    assert_eq!(root.rect.left, 0);
    assert_eq!(root.rect.right, 390);
    assert_eq!(root.rect.bottom - root.rect.top, 125);

    // 2 sections: body and footer
    assert_eq!(root.children.len(), 2);
    let body = &root.children[0];
    let footer_section = &root.children[1];
    assert_eq!(footer_section.rect.bottom, root.rect.bottom);

    // In body: message container
    assert_eq!(body.children.len(), 1);

    // In footer section: row containing spacer, button_no, button_yes
    assert_eq!(footer_section.children.len(), 1);
    let footer_row = &footer_section.children[0];
    assert_eq!(footer_row.children.len(), 3);
    if let WidgetKind::Button { on_click, .. } = &footer_row.children[1].element.kind {
        assert_eq!(*on_click, Some(false));
    } else {
        panic!("Expected button for no_text");
    }
    if let WidgetKind::Button { on_click, .. } = &footer_row.children[2].element.kind {
        assert_eq!(*on_click, Some(true));
    } else {
        panic!("Expected button for yes_text");
    }
}

#[test]
fn test_modal_dialog_3_sections_optional() {
    use kora::ui_framework::core::*;

    // 1. All 3 sections: header, body, footer
    let full_dialog: Element<()> = modal_dialog()
        .header(text("Tiêu đề").bold())
        .body(text("Nội dung"))
        .footer(modal_footer([button("Đóng", Variant::Secondary, ())]))
        .build();
    let root3 = compute_layout(0, full_dialog, 0, 0, 400);
    assert_eq!(root3.children.len(), 3);

    // 2. Only body: no header, no footer
    let body_only: Element<()> = modal_dialog()
        .body(text("Chỉ có nội dung"))
        .build();
    let root1 = compute_layout(0, body_only, 0, 0, 400);
    assert_eq!(root1.children.len(), 1);

    // 3. Header + body: no footer
    let header_body: Element<()> = modal_dialog()
        .header(text("Tiêu đề"))
        .body(text("Nội dung"))
        .build();
    let root2 = compute_layout(0, header_body, 0, 0, 400);
    assert_eq!(root2.children.len(), 2);
}

#[test]
fn test_modal_disable_key_declarative_layout() {
    use kora::ui::views::dialogs::base::{dialog_callout, key_capture_box};
    use kora::ui_framework::core::*;

    #[derive(Clone, PartialEq, Debug)]
    enum TestMsg {
        Toggle,
        Preset(u32),
        Cancel,
        Submit,
    }

    let view: Element<TestMsg> = modal_dialog()
        .header(dialog_callout("Chọn phím bạn muốn vô hiệu hóa"))
        .body(
            col([
                col([
                    key_capture_box("Caps Lock", false, false, TestMsg::Toggle),
                ])
                .gap(1),
                row([
                    button("🚫 Win", Variant::Secondary, TestMsg::Preset(0x5B)),
                    button("🚫 Caps", Variant::Secondary, TestMsg::Preset(0x14)),
                ])
                .gap(2),
            ])
            .gap(3),
        )
        .footer(
            row([
                text("Áp dụng cho: 🌐 Mặc định").secondary().small(),
                spacer(),
                button("Hủy", Variant::Secondary, TestMsg::Cancel),
                button("Khóa phím", Variant::Danger, TestMsg::Submit).disabled(false),
            ])
            .gap(2)
            .items_center(),
        )
        .build()
        .h(340);

    let root = compute_layout(0, view, 0, 0, 480);
    assert_eq!(root.rect.left, 0);
    assert_eq!(root.rect.right, 480);
    assert_eq!(root.rect.bottom - root.rect.top, 340);
    assert_eq!(root.children.len(), 3); // Header, Body, Footer

    // Section 0: Callout box
    assert!(root.children[0].rect.right <= 480);

    // Section 1: Body with key capture box
    let body = &root.children[1];
    let body_col = &body.children[0];
    let key_box_col = &body_col.children[0];
    let key_box = &key_box_col.children[0];
    assert_eq!(key_box.element.on_click, Some(TestMsg::Toggle));
    assert_eq!(key_box.rect.bottom - key_box.rect.top, 46);

    // Section 2: Footer PINNED directly to bottom border
    let footer_section = &root.children[2];
    assert_eq!(footer_section.rect.bottom, root.rect.bottom);

    let footer_row = &footer_section.children[0];
    let cancel_btn = &footer_row.children[2];
    let apply_btn = &footer_row.children[3];
    assert_eq!(apply_btn.rect.left - cancel_btn.rect.right, 8); // 8px margin between buttons

    if let WidgetKind::Button { on_click, disabled, .. } = &apply_btn.element.kind {
        assert_eq!(*on_click, Some(TestMsg::Submit));
        assert_eq!(*disabled, false);
    } else {
        panic!("Expected submit button in footer");
    }
}

#[test]
fn test_modal_remap_key_declarative_layout() {
    use kora::ui::views::dialogs::base::{dialog_callout, key_capture_box};
    use kora::ui_framework::core::*;

    #[derive(Clone, PartialEq, Debug)]
    enum TestMsg {
        CaptureFrom,
        CaptureTo,
        Cancel,
        Apply,
    }

    let view: Element<TestMsg> = modal_dialog()
        .header(dialog_callout("Chọn phím bạn muốn đổi"))
        .body(
            row([
                col([
                    key_capture_box("Caps Lock", false, false, TestMsg::CaptureFrom),
                ]).flex_1(),
                col([text("➜").accent().title_font()]).w(48).pad_xy(2, 0),
                col([
                    key_capture_box("Chưa gán", false, true, TestMsg::CaptureTo),
                ]).flex_1(),
            ]).items_center(),
        )
        .footer(
            row([
                text("Áp dụng cho: 🌐 Mặc định").secondary().small(),
                spacer(),
                button("Hủy", Variant::Secondary, TestMsg::Cancel),
                button("Đổi phím", Variant::Primary, TestMsg::Apply).disabled(true),
            ])
            .gap(2)
            .items_center(),
        )
        .build()
        .h(300);

    let root = compute_layout(0, view, 0, 0, 500);
    assert_eq!(root.rect.left, 0);
    assert_eq!(root.rect.right, 500);
    assert_eq!(root.rect.bottom - root.rect.top, 300);
    assert_eq!(root.children.len(), 3); // Header, Body, Footer

    // Section 0: Header callout
    let header = &root.children[0];
    assert!(header.rect.bottom > 0);

    // Section 1: Body
    let body = &root.children[1];
    let remap_row = &body.children[0];
    assert_eq!(remap_row.children.len(), 3);

    // From and To boxes should share flex proportions equally
    let from_box_col = &remap_row.children[0];
    let to_box_col = &remap_row.children[2];
    let from_w = from_box_col.rect.right - from_box_col.rect.left;
    let to_w = to_box_col.rect.right - to_box_col.rect.left;
    assert_eq!(from_w, to_w);
    assert!(from_w > 180);

    // Text inside key capture box must span full width for perfect DT_CENTER centering
    let key_box = &from_box_col.children[0];
    let text_node = &key_box.children[0];
    let key_box_w = key_box.rect.right - key_box.rect.left;
    let text_w = text_node.rect.right - text_node.rect.left;
    assert_eq!(text_w, key_box_w);

    // Section 2: Footer pinned to bottom
    let footer_section = &root.children[2];
    assert_eq!(footer_section.rect.bottom, root.rect.bottom);
    let footer_row = &footer_section.children[0];

    // Buttons in footer must have 8px margin (gap(2)) between each other
    let cancel_btn = &footer_row.children[2];
    let apply_btn = &footer_row.children[3];
    assert_eq!(apply_btn.rect.left - cancel_btn.rect.right, 8);

    if let WidgetKind::Button { disabled, .. } = &apply_btn.element.kind {
        assert_eq!(*disabled, true);
    } else {
        panic!("Expected disabled apply button in footer");
    }
}
