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
fn test_vstack_fixed_and_spacer() {
    let mut stack = VStack::at(0, 0, 100, 300);
    stack.add_fixed(50, None);
    stack.add_spacer();
    stack.add_fixed(80, None);
    let rects = stack.apply();
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0], Rect::new(0, 0, 100, 50));
    assert_eq!(rects[1], Rect::new(0, 50, 100, 170));
    assert_eq!(rects[2], Rect::new(0, 220, 100, 80));
}

#[test]
fn test_grid_layout_2x2() {
    let mut grid = GridLayout::at(0, 0, 200, 100, 2, 2);
    grid.gap(10, 10);
    grid.cell(0, 0, None);
    grid.cell(0, 1, None);
    grid.cell(1, 0, None);
    grid.cell(1, 1, None);
    let rects = grid.apply();
    assert_eq!(rects.len(), 4);
    assert_eq!(rects[0], Rect::new(0, 0, 95, 45));
    assert_eq!(rects[1], Rect::new(105, 0, 95, 45));
    assert_eq!(rects[2], Rect::new(0, 55, 95, 45));
    assert_eq!(rects[3], Rect::new(105, 55, 95, 45));
}

#[test]
fn test_grid_layout_spans() {
    let mut grid = GridLayout::at(0, 0, 300, 200, 3, 2);
    grid.cell_span(0, 0, 2, 1, None); // Span 2 cols in row 0
    grid.cell(0, 2, None);             // 1 col in row 0
    let rects = grid.apply();
    assert_eq!(rects[0], Rect::new(0, 0, 200, 100));
    assert_eq!(rects[1], Rect::new(200, 0, 100, 100));
}

#[test]
fn test_form_layout_single_row() {
    let mut form = FormLayout::at(0, 0, 400, 100);
    form.label_width(100).col_gap(10);
    form.row_with_value(None, None, None, 40);
    let rects = form.compute();
    assert_eq!(rects.len(), 1);
    let (lbl, ctl, val) = rects[0];
    assert_eq!(lbl, Rect::new(0, 4, 100, 24));
    // remaining = 400 - 100 - 10 - 40 - 10 = 240
    assert_eq!(ctl, Rect::new(110, 0, 240, 28));
    assert_eq!(val, Some(Rect::new(360, 4, 40, 24)));
}

#[test]
fn test_split_fixed() {
    let split = SplitLayout::at(0, 0, 500, 300, SplitDirection::Horizontal, SplitRule::FixedFirst(200));
    let (r1, r2) = split.compute();
    assert_eq!(r1, Rect::new(0, 0, 200, 300));
    assert_eq!(r2, Rect::new(200, 0, 300, 300));
}

#[test]
fn test_split_ratio() {
    let split = SplitLayout::at(0, 0, 500, 300, SplitDirection::Horizontal, SplitRule::Ratio(0.4));
    let (r1, r2) = split.compute();
    assert_eq!(r1, Rect::new(0, 0, 200, 300));
    assert_eq!(r2, Rect::new(200, 0, 300, 300));
}

#[test]
fn test_split_vertical_with_gap() {
    let mut split = SplitLayout::at(0, 0, 200, 410, SplitDirection::Vertical, SplitRule::FixedFirst(100));
    split.gap(10);
    let (r1, r2) = split.compute();
    assert_eq!(r1, Rect::new(0, 0, 200, 100));
    assert_eq!(r2, Rect::new(0, 110, 200, 300));
}

