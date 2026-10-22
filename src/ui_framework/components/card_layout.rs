//! Progressive Vertical Card & Row Layout Builder for Control Panel Tabs
//!
//! Eliminates manual Y-coordinate math, hardcoded card bottom constants,
//! repetitive `parent`/`hfont` arguments, manual `hwnd()` visibility arrays,
//! and manual per-label `update_language` calls.

use crate::ui_framework::I18nSource;
use crate::ui_framework::components::action_table::ActionTable;
use crate::ui_framework::components::{
    ButtonVariant, CheckBox, ComboBox, ComponentHandle, Inheritable, Label, LabelTone, PillBadge,
    PushButton, Slider, SurfaceRole, TwoState,
};
use crate::ui_framework::theme::cached_font;
use crate::ui_framework::tokens::{layout, radii, spacing, typography};
use crate::ui_framework::win32::{
    FW_BOLD, InvalidateRect, RoundRect, SetWindowTextW, set_hwnds_visible, to_wide,
};
use std::marker::PhantomData;
use std::ptr::null;

/// Marker type indicating the layout cursor is outside any card container.
#[derive(Debug, Clone, Copy)]
pub struct OutsideCard;

/// Marker type indicating the layout cursor is currently inside an open card container.
#[derive(Debug, Clone, Copy)]
pub struct InCard;

/// Trait associating each layout typestate `S` with its phase-specific payload (`()` for `OutsideCard`, `card_top: i32` for `InCard`).
pub trait LayoutState {
    type Data: Copy + Clone + std::fmt::Debug;
}

impl LayoutState for OutsideCard {
    type Data = ();
}

impl LayoutState for InCard {
    type Data = i32;
}

/// Function pointer selecting a single localized static string from `L`.
pub type TextFn<L = ()> = fn(&L) -> &'static str;

/// Function pointer selecting a list of localized static strings (ComboBox items / ActionTable headers) from `L`.
pub type TextListFn<L = ()> = fn(&L) -> Vec<&'static str>;

/// Progressive layout cursor and component tracker for stacking Cards and rows inside a Tab view.
#[derive(Debug, Clone)]
#[must_use = "if inside a card (CardLayout<InCard>), call .end_card() or use .card(|c| ...)"]
pub struct CardLayout<S: LayoutState = OutsideCard, L: I18nSource = ()> {
    parent: isize,
    hfont_normal: isize,
    hfont_bold: isize,
    content_x: i32,
    content_w: i32,
    cur_y: i32,
    state: S::Data,
    cards: Vec<(i32, i32)>,
    hwnds: Vec<isize>,
    i18n: Vec<(isize, TextFn<L>)>,
    combo_i18n: Vec<(isize, TextListFn<L>)>,
    table_i18n: Vec<(isize, TextListFn<L>)>,
    _state: PhantomData<(S, L)>,
}

impl<L: I18nSource> Default for CardLayout<OutsideCard, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: LayoutState, L: I18nSource> CardLayout<S, L> {
    #[inline]
    fn into_state<Next: LayoutState>(self, state: Next::Data) -> CardLayout<Next, L> {
        CardLayout {
            parent: self.parent,
            hfont_normal: self.hfont_normal,
            hfont_bold: self.hfont_bold,
            content_x: self.content_x,
            content_w: self.content_w,
            cur_y: self.cur_y,
            state,
            cards: self.cards,
            hwnds: self.hwnds,
            i18n: self.i18n,
            combo_i18n: self.combo_i18n,
            table_i18n: self.table_i18n,
            _state: PhantomData,
        }
    }

    /// Current content X coordinate inside a card (defaults to `layout::CARD_CONTENT_X = 260`).
    #[inline]
    pub const fn x(&self) -> i32 {
        self.content_x
    }

    /// Current content width inside a card (defaults to `layout::CARD_CONTENT_WIDTH = 530`).
    #[inline]
    pub const fn width(&self) -> i32 {
        self.content_w
    }

    /// Right edge of the content area inside a card (`x + width`).
    #[inline]
    pub const fn right(&self) -> i32 {
        self.content_x + self.content_w
    }

    /// Returns the current Y cursor position without advancing it.
    #[inline]
    pub const fn y(&self) -> i32 {
        self.cur_y
    }

    /// Advances the Y cursor by `dy` pixels.
    #[inline]
    pub fn skip(&mut self, dy: i32) {
        self.cur_y += dy;
    }

    /// Allocates a row of `height` pixels and advances the cursor by `height + gap_after`.
    /// Returns the top `y` coordinate for the row.
    #[inline]
    pub fn row(&mut self, height: i32, gap_after: i32) -> i32 {
        let y = self.cur_y;
        self.cur_y += height + gap_after;
        y
    }

    // -----------------------------------------------------------------------
    // Automatic HWND Visibility & Localization Tracking
    // -----------------------------------------------------------------------

    /// Registers a component's `HWND` so [`CardLayout::set_visible`] automatically shows/hides it.
    #[inline]
    pub fn track<C: ComponentHandle>(&mut self, comp: C) -> C {
        let h = comp.hwnd();
        if h != 0 && !self.hwnds.contains(&h) {
            self.hwnds.push(h);
        }
        comp
    }

    /// Registers a component's `HWND` for visibility AND binds an `L` selector
    /// so [`CardLayout::update_language`] automatically updates its text on language switch.
    #[inline]
    pub fn track_i18n<C: ComponentHandle>(&mut self, comp: C, text_fn: TextFn<L>) -> C {
        let h = comp.hwnd();
        if h != 0 {
            if !self.hwnds.contains(&h) {
                self.hwnds.push(h);
            }
            self.i18n.push((h, text_fn));
        }
        comp
    }

    /// Registers a `ComboBox` for visibility AND binds an item-list selector
    /// so [`CardLayout::update_language`] automatically updates its items while preserving selection.
    #[inline]
    pub fn track_combo_i18n(&mut self, combo: ComboBox, items_fn: TextListFn<L>) -> ComboBox {
        let h = combo.hwnd();
        if h != 0 {
            if !self.hwnds.contains(&h) {
                self.hwnds.push(h);
            }
            self.combo_i18n.push((h, items_fn));
        }
        combo
    }

    /// Registers an `ActionTable` for visibility AND binds a column-headers selector
    /// so [`CardLayout::update_language`] automatically updates all column titles.
    #[inline]
    pub fn track_table_i18n(&mut self, table: ActionTable, cols_fn: TextListFn<L>) -> ActionTable {
        let h = table.hwnd();
        if h != 0 {
            if !self.hwnds.contains(&h) {
                self.hwnds.push(h);
            }
            self.table_i18n.push((h, cols_fn));
        }
        table
    }

    #[inline]
    fn font_for(&self, bold: bool) -> isize {
        if bold {
            self.hfont_bold
        } else {
            self.hfont_normal
        }
    }

    /// Creates a localized `Label` for auto-layout engines (without manual x, y, w, h)
    /// and registers it via [`Self::track_i18n`].
    pub fn label(&mut self, text_fn: TextFn<L>, bold: bool) -> Option<Label> {
        let text = text_fn(L::current());
        let lbl = Label::new(self.parent, text, self.font_for(bold))?;
        Some(self.track_i18n(lbl, text_fn))
    }

    /// Creates a localized `PushButton` for auto-layout engines (without manual x, y, w, h)
    /// and registers it via [`Self::track_i18n`].
    pub fn button(
        &mut self,
        text_fn: TextFn<L>,
        bold: bool,
        variant: ButtonVariant,
    ) -> Option<PushButton> {
        let text = text_fn(L::current());
        let btn = PushButton::new(self.parent, text, self.font_for(bold), variant)?;
        Some(self.track_i18n(btn, text_fn))
    }

    /// Creates a localized 2-state `CheckBox<TwoState>` for auto-layout engines (without manual x, y, w, h)
    /// and registers it via [`Self::track_i18n`].
    pub fn checkbox(&mut self, text_fn: TextFn<L>, bold: bool) -> Option<CheckBox<TwoState>> {
        let text = text_fn(L::current());
        let chk = CheckBox::new(self.parent, text, self.font_for(bold))?;
        Some(self.track_i18n(chk, text_fn))
    }

    /// Creates a localized 3-state `CheckBox<Inheritable<L>>` for auto-layout engines (without manual x, y, w, h)
    /// and registers it via [`Self::track_i18n`].
    pub fn inheritable_checkbox(
        &mut self,
        text_fn: TextFn<L>,
        bold: bool,
    ) -> Option<CheckBox<Inheritable<L>>> {
        let text = text_fn(L::current());
        let chk = CheckBox::new_inheritable(self.parent, text, self.font_for(bold))?;
        Some(self.track_i18n(chk, text_fn))
    }

    /// Creates a ComboBox with localized items for auto-layout engines (without manual x, y, w, h)
    /// and registers it via [`Self::track_combo_i18n`].
    pub fn combo(&mut self, items_fn: TextListFn<L>) -> Option<ComboBox> {
        let strings = L::current();
        let items = items_fn(strings);
        let cmb = ComboBox::new_with_items(self.parent, self.hfont_normal, &items, 0)?;
        Some(self.track_combo_i18n(cmb, items_fn))
    }

    /// Creates a Slider for auto-layout engines (without manual x, y, w, h)
    /// and registers it via [`Self::track`].
    pub fn slider(&mut self, min: u32, max: u32, initial_value: u32) -> Option<Slider> {
        let sl = Slider::new_with_value(self.parent, 0, min, max, initial_value)?;
        Some(self.track(sl))
    }
}

impl<L: I18nSource> CardLayout<OutsideCard, L> {
    /// Starts a new card layout at the standard sub-tab card top (`layout::CARD_START_Y = 112`).
    pub fn new() -> Self {
        Self::at_y(layout::CARD_START_Y)
    }

    /// Starts a card layout at a custom top `y` coordinate (e.g. `86` for views without a sub-tab bar).
    pub fn at_y(start_y: i32) -> Self {
        Self {
            parent: 0,
            hfont_normal: 0,
            hfont_bold: 0,
            content_x: layout::CARD_CONTENT_X,
            content_w: layout::CARD_CONTENT_WIDTH,
            cur_y: start_y,
            state: (),
            cards: Vec::with_capacity(4),
            hwnds: Vec::with_capacity(16),
            i18n: Vec::with_capacity(12),
            combo_i18n: Vec::new(),
            table_i18n: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Starts a context-aware card layout bound to `parent` and standard fonts.
    /// Enables 1-line row builders (`section_title`, `checkbox_row`, `label_row`, `page_header`)
    /// and automatic HWND visibility/localization tracking.
    pub fn with_parent(parent: isize, hfont_normal: isize, hfont_bold: isize) -> Self {
        Self {
            parent,
            hfont_normal,
            hfont_bold,
            ..Self::new()
        }
    }

    /// Overrides the starting `Y` coordinate (builder style).
    #[inline]
    pub fn at(mut self, start_y: i32) -> Self {
        self.cur_y = start_y;
        self
    }

    /// Overrides the horizontal content bounds `(x, width)` inside cards (builder style).
    #[inline]
    pub fn with_bounds(mut self, content_x: i32, content_w: i32) -> Self {
        self.content_x = content_x;
        self.content_w = content_w;
        self
    }

    /// Creates the standard two-line page header (`HEADER_X`, title + secondary subtitle on `SurfaceRole::Window`),
    /// tracking both labels for visibility and language switching. Only valid outside of a card.
    pub fn page_header(
        &mut self,
        title_fn: TextFn<L>,
        sub_fn: TextFn<L>,
    ) -> Option<(Label, Label)> {
        let strings = L::current();
        let hfont_title = cached_font(typography::FONT_SIZE_TITLE, FW_BOLD);
        let lbl_title = Label::create(
            self.parent,
            title_fn(strings),
            layout::HEADER_X,
            layout::HEADER_TITLE_Y,
            layout::HEADER_TITLE_WIDTH,
            layout::HEADER_TITLE_HEIGHT,
            hfont_title,
        )?
        .on_surface(SurfaceRole::Window);

        let lbl_sub = Label::create(
            self.parent,
            sub_fn(strings),
            layout::HEADER_X,
            layout::HEADER_SUB_Y,
            layout::HEADER_SUB_WIDTH,
            layout::HEADER_SUB_HEIGHT,
            self.hfont_normal,
        )?
        .with_tone(LabelTone::Secondary)
        .on_surface(SurfaceRole::Window);

        Some((
            self.track_i18n(lbl_title, title_fn),
            self.track_i18n(lbl_sub, sub_fn),
        ))
    }

    /// Begins a new card with custom top padding and transitions to [`InCard`].
    #[inline]
    fn begin_card_with_pad(mut self, pad_top: i32) -> CardLayout<InCard, L> {
        let card_top = self.cur_y;
        self.cur_y += pad_top;
        self.into_state(card_top)
    }

    /// Closure-scoped card builder with default top/bottom padding (`14px`).
    /// Guarantees at compile time that the card is closed when the closure exits.
    #[inline]
    pub fn card<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut CardLayout<InCard, L>) -> R,
    {
        self.card_with_pad(spacing::SPACE_14, spacing::SPACE_14, f)
    }

    /// Closure-scoped card builder with custom `pad_top` and `pad_bottom`.
    /// Guarantees at compile time that the card is closed when the closure exits.
    pub fn card_with_pad<R, F>(&mut self, pad_top: i32, pad_bottom: i32, f: F) -> R
    where
        F: FnOnce(&mut CardLayout<InCard, L>) -> R,
    {
        let taken = std::mem::replace(self, Self::at_y(0));
        let mut in_card = taken.begin_card_with_pad(pad_top);
        let ret = f(&mut in_card);
        *self = in_card.end_card_with_pad(pad_bottom);
        ret
    }

    /// Closure-scoped card builder for a card stretching to a fixed `bottom_y`.
    /// Guarantees at compile time that the card is closed when the closure exits.
    pub fn card_until<R, F>(&mut self, pad_top: i32, bottom_y: i32, f: F) -> R
    where
        F: FnOnce(&mut CardLayout<InCard, L>) -> R,
    {
        let taken = std::mem::replace(self, Self::at_y(0));
        let mut in_card = taken.begin_card_with_pad(pad_top);
        let ret = f(&mut in_card);
        *self = in_card.end_card_at(bottom_y);
        ret
    }

    /// Returns the recorded `(top, bottom)` card bounds slice for passing directly to `draw_cards`.
    #[inline]
    pub fn cards(&self) -> &[(i32, i32)] {
        &self.cards
    }

    /// Draws rounded card containers `(top, bottom)` spanning `[left..right]` with `radii::RADIUS_CARD`.
    pub fn draw_cards(hdc: isize, left: i32, right: i32, y_ranges: &[(i32, i32)]) {
        unsafe {
            for &(top, bottom) in y_ranges {
                RoundRect(
                    hdc,
                    left,
                    top,
                    right,
                    bottom,
                    radii::RADIUS_CARD,
                    radii::RADIUS_CARD,
                );
            }
        }
    }

    /// Paints all cards recorded in this layout spanning `[left..right]`.
    #[inline]
    pub fn paint_cards(&self, hdc: isize, left: i32, right: i32) {
        Self::draw_cards(hdc, left, right, &self.cards);
    }

    /// Shows or hides all components tracked by this layout in one call.
    #[inline]
    pub fn set_visible(&self, visible: bool) {
        set_hwnds_visible(&self.hwnds, visible);
    }

    /// Returns all tracked `HWND` handles.
    #[inline]
    pub fn hwnds(&self) -> &[isize] {
        &self.hwnds
    }

    /// Automatically updates the window text, ComboBox items, and ActionTable headers of all
    /// components registered via [`Self::track_i18n`], [`Self::track_combo_i18n`],
    /// [`Self::track_table_i18n`], or the 1-line row builders.
    pub fn update_language(&self, strings: &L) {
        for &(hwnd, text_fn) in &self.i18n {
            let wide = to_wide(text_fn(strings));
            unsafe {
                SetWindowTextW(hwnd, wide.as_ptr());
                InvalidateRect(hwnd, null(), 1);
            }
        }
        for &(hwnd, items_fn) in &self.combo_i18n {
            let items = items_fn(strings);
            ComboBox::set_items_by_hwnd(hwnd, &items);
        }
        for &(hwnd, cols_fn) in &self.table_i18n {
            let cols = cols_fn(strings);
            ActionTable::set_column_titles_by_hwnd(hwnd, &cols);
        }
    }
}

impl<L: I18nSource> CardLayout<InCard, L> {
    /// Allocates a `20px` row and creates a bold section title `Label` spanning the card width.
    /// Automatically tracked for visibility and `update_language`.
    pub fn section_title(&mut self, text_fn: TextFn<L>, gap_after: i32) -> Option<Label> {
        self.label_row(text_fn, 20, gap_after, true)
    }

    /// Allocates a row of `height` pixels and creates a full-width `Label` (bold or normal).
    /// Automatically tracked for visibility and `update_language`.
    pub fn label_row(
        &mut self,
        text_fn: TextFn<L>,
        height: i32,
        gap_after: i32,
        bold: bool,
    ) -> Option<Label> {
        let strings = L::current();
        let y = self.row(height, gap_after);
        let font = if bold {
            self.hfont_bold
        } else {
            self.hfont_normal
        };
        let lbl = Label::create(
            self.parent,
            text_fn(strings),
            self.content_x,
            y,
            self.content_w,
            height,
            font,
        )?;
        Some(self.track_i18n(lbl, text_fn))
    }

    /// Allocates a row of `height` pixels and creates a full-width 2-state `CheckBox<TwoState>` with initial `checked` state.
    /// Automatically tracked for visibility and `update_language`.
    pub fn checkbox_row(
        &mut self,
        text_fn: TextFn<L>,
        height: i32,
        gap_after: i32,
        bold: bool,
        checked: bool,
    ) -> Option<CheckBox<TwoState>> {
        let strings = L::current();
        let y = self.row(height, gap_after);
        let font = if bold {
            self.hfont_bold
        } else {
            self.hfont_normal
        };
        let chk = CheckBox::create_with_state(
            self.parent,
            0,
            text_fn(strings),
            self.content_x,
            y,
            self.content_w,
            height,
            font,
            checked,
        )?;
        Some(self.track_i18n(chk, text_fn))
    }

    /// Allocates a row of `height` pixels and creates a full-width 3-state `CheckBox<Inheritable<L>>`.
    /// Automatically tracked for visibility and `update_language`.
    pub fn inheritable_checkbox_row(
        &mut self,
        text_fn: TextFn<L>,
        height: i32,
        gap_after: i32,
        bold: bool,
    ) -> Option<CheckBox<Inheritable<L>>> {
        let strings = L::current();
        let y = self.row(height, gap_after);
        let font = if bold {
            self.hfont_bold
        } else {
            self.hfont_normal
        };
        let chk = CheckBox::create_inheritable(
            self.parent,
            0,
            text_fn(strings),
            self.content_x,
            y,
            self.content_w,
            height,
            font,
        )?;
        Some(self.track_i18n(chk, text_fn))
    }

    /// Creates a 2-column pair of `(Label, ComboBox)` side by side using `GridLayout`, automatically tracking
    /// both labels (`track_i18n`) and both comboboxes (`track_combo_i18n`).
    pub fn two_col_combos(
        &mut self,
        col_gap: i32,
        gap_after: i32,
        label_bold: bool,
        left: (TextFn<L>, TextListFn<L>),
        right: (TextFn<L>, TextListFn<L>),
    ) -> Option<((Label, ComboBox), (Label, ComboBox))> {
        let strings = L::current();
        let y_lbl = self.row(18, 4);
        let lbl_font = if label_bold {
            self.hfont_bold
        } else {
            self.hfont_normal
        };
        let mut lbl1 = Label::new(self.parent, (left.0)(strings), lbl_font)?;
        let mut lbl2 = Label::new(self.parent, (right.0)(strings), lbl_font)?;
        if !label_bold {
            lbl1 = lbl1.with_tone(LabelTone::Secondary);
            lbl2 = lbl2.with_tone(LabelTone::Secondary);
        }
        let lbl1 = self.track_i18n(lbl1, left.0);
        let lbl2 = self.track_i18n(lbl2, right.0);

        let y_cmb = self.row(28, gap_after);
        let items1 = (left.1)(strings);
        let items2 = (right.1)(strings);
        let cmb1 = ComboBox::new_with_items(self.parent, self.hfont_normal, &items1, 0)?;
        let cmb2 = ComboBox::new_with_items(self.parent, self.hfont_normal, &items2, 0)?;
        let cmb1 = self.track_combo_i18n(cmb1, left.1);
        let cmb2 = self.track_combo_i18n(cmb2, right.1);

        // 2x2 Declarative GridLayout
        let total_h = (y_cmb + 28) - y_lbl;
        let mut grid = crate::ui_framework::layout::GridLayout::at(
            self.content_x,
            y_lbl,
            self.content_w,
            total_h,
            2,
            2,
        );
        grid.gap(col_gap, 4);
        grid.cell(0, 0, Some(lbl1.hwnd()));
        grid.cell(0, 1, Some(lbl2.hwnd()));
        grid.cell(1, 0, Some(cmb1.hwnd()));
        grid.cell(1, 1, Some(cmb2.hwnd()));
        grid.apply();

        Some(((lbl1, cmb1), (lbl2, cmb2)))
    }

    /// Creates a 1-column `(Label, ComboBox)` taking up the first half of a 2-column grid using `GridLayout`,
    /// automatically tracking the label (`track_i18n`) and combobox (`track_combo_i18n`).
    pub fn single_col_combo(
        &mut self,
        gap_after: i32,
        label_bold: bool,
        item: (TextFn<L>, TextListFn<L>),
    ) -> Option<(Label, ComboBox)> {
        let strings = L::current();
        let y_lbl = self.row(18, 4);
        let lbl_font = if label_bold {
            self.hfont_bold
        } else {
            self.hfont_normal
        };
        let mut lbl = Label::new(self.parent, (item.0)(strings), lbl_font)?;
        if !label_bold {
            lbl = lbl.with_tone(LabelTone::Secondary);
        }
        let lbl = self.track_i18n(lbl, item.0);

        let y_cmb = self.row(28, gap_after);
        let items = (item.1)(strings);
        let cmb = ComboBox::new_with_items(self.parent, self.hfont_normal, &items, 0)?;
        let cmb = self.track_combo_i18n(cmb, item.1);

        // Place in column 0 of a 2-column grid
        let total_h = (y_cmb + 28) - y_lbl;
        let mut grid = crate::ui_framework::layout::GridLayout::at(
            self.content_x,
            y_lbl,
            self.content_w,
            total_h,
            2,
            2,
        );
        grid.gap(20, 4);
        grid.cell(0, 0, Some(lbl.hwnd()));
        grid.cell(1, 0, Some(cmb.hwnd()));
        grid.apply();

        Some((lbl, cmb))
    }

    /// Runs a horizontal layout inside this card row with auto-positioning.
    /// Allocates `height` pixels on the Y axis, then passes an `HStack` (HorizontalLayout) builder to `f`.
    pub fn horizontal<R, F>(&mut self, height: i32, gap_after: i32, f: F) -> R
    where
        F: FnOnce(&mut crate::ui_framework::layout::HStack) -> R,
    {
        let y = self.row(height, gap_after);
        let mut hstack = crate::ui_framework::layout::HStack::at(self.content_x, y, self.content_w, height);
        let ret = f(&mut hstack);
        hstack.apply();
        ret
    }

    /// Runs a vertical layout inside this card row with auto-positioning.
    /// Allocates `height` pixels on the Y axis, then passes a `VStack` (VerticalLayout) builder to `f`.
    pub fn vertical<R, F>(&mut self, height: i32, gap_after: i32, f: F) -> R
    where
        F: FnOnce(&mut crate::ui_framework::layout::VStack) -> R,
    {
        let y = self.row(height, gap_after);
        let mut vstack = crate::ui_framework::layout::VStack::at(self.content_x, y, self.content_w, height);
        let ret = f(&mut vstack);
        vstack.apply();
        ret
    }

    /// Runs a 2D grid layout inside this card row with auto-positioning.
    /// Allocates `height` pixels on the Y axis, then passes a `GridLayout` builder to `f`.
    pub fn grid<R, F>(&mut self, height: i32, gap_after: i32, cols: usize, rows: usize, f: F) -> R
    where
        F: FnOnce(&mut crate::ui_framework::layout::GridLayout) -> R,
    {
        let y = self.row(height, gap_after);
        let mut grid = crate::ui_framework::layout::GridLayout::at(self.content_x, y, self.content_w, height, cols, rows);
        let ret = f(&mut grid);
        grid.apply();
        ret
    }

    /// Runs a form layout inside this card row with auto-positioning.
    /// Allocates `height` pixels on the Y axis, then passes a `FormLayout` builder to `f`.
    pub fn form<R, F>(&mut self, height: i32, gap_after: i32, f: F) -> R
    where
        F: FnOnce(&mut crate::ui_framework::layout::FormLayout) -> R,
    {
        let y = self.row(height, gap_after);
        let mut form = crate::ui_framework::layout::FormLayout::at(self.content_x, y, self.content_w, height);
        let ret = f(&mut form);
        form.apply();
        ret
    }

    /// Runs a two-pane split layout inside this card row with auto-positioning.
    /// Allocates `height` pixels on the Y axis, then passes a `SplitLayout` builder to `f`.
    pub fn split<R, F>(
        &mut self,
        height: i32,
        gap_after: i32,
        direction: crate::ui_framework::layout::SplitDirection,
        rule: crate::ui_framework::layout::SplitRule,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut crate::ui_framework::layout::SplitLayout) -> R,
    {
        let y = self.row(height, gap_after);
        let mut splitter = crate::ui_framework::layout::SplitLayout::at(
            self.content_x,
            y,
            self.content_w,
            height,
            direction,
            rule,
        );
        let ret = f(&mut splitter);
        splitter.apply();
        ret
    }

    /// Creates a standard hotkey row:
    /// - Left: Label with `lbl_text_fn`
    /// - Center: PillBadge showing `current_combo`
    /// - Right: PushButton with `btn_text_fn` (Primary variant)
    pub fn hotkey_row(
        &mut self,
        lbl_text_fn: TextFn<L>,
        current_combo: &str,
        btn_text_fn: TextFn<L>,
        gap_after: i32,
    ) -> Option<(Label, Label<PillBadge>, PushButton)> {
        let lbl = self.label(lbl_text_fn, true)?;
        let badge = self.track(
            Label::new_centered(self.parent, current_combo, self.hfont_bold)?
                .as_pill_badge(120, 300),
        );
        let btn = self.button(btn_text_fn, true, ButtonVariant::Primary)?;

        self.horizontal(28, gap_after, |row| {
            row.align(crate::ui_framework::layout::CrossAlign::Center);
            row.add_fixed(124, Some(lbl.hwnd()));
            row.add_fixed(10, None);
            row.add_fixed(120, Some(badge.hwnd()));
            row.add_spacer();
            row.add_fixed(86, Some(btn.hwnd()));
        });

        Some((lbl, badge, btn))
    }

    /// Closes the current card with custom bottom padding and transitions back to [`OutsideCard`].
    #[inline]
    fn end_card_with_pad(mut self, pad_bottom: i32) -> CardLayout<OutsideCard, L> {
        let bottom = self.cur_y + pad_bottom;
        self.cards.push((self.state, bottom));
        self.cur_y = bottom + layout::CARD_GAP;
        self.into_state(())
    }

    /// Closes a card that stretches down to a fixed `bottom_y` (e.g. `layout::CARD_BOTTOM_Y` for tables)
    /// and transitions back to [`OutsideCard`].
    #[inline]
    fn end_card_at(mut self, bottom_y: i32) -> CardLayout<OutsideCard, L> {
        self.cards.push((self.state, bottom_y));
        self.cur_y = bottom_y + layout::CARD_GAP;
        self.into_state(())
    }
}
