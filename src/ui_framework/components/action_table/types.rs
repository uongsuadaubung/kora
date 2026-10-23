//! ActionTable Models & Data Types

use crate::ui_framework::colors::ThemePalette;
use crate::ui_framework::win32::RECT;
use std::sync::Arc;

pub const HEADER_HEIGHT: i32 = 30;
pub const ROW_HEIGHT: i32 = 32;
pub const IDC_HAND_ID: usize = 32649;
pub const IDC_ARROW_ID: usize = 32512;



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeVariant {
    KeyPill,
    Danger,
    Secondary,
    Info,
    Warning,
    Success,
    Purple,
}

#[derive(Clone, Debug)]
pub struct BadgeConfig {
    pub text: String,
    pub variant: BadgeVariant,
    pub suffix: Option<String>,
}

impl BadgeConfig {
    pub fn key(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::KeyPill,
            suffix: None,
        }
    }

    pub fn key_with_arrow(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::KeyPill,
            suffix: Some("➜".to_string()),
        }
    }

    pub fn danger_key(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Danger,
            suffix: None,
        }
    }

    pub fn secondary(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Secondary,
            suffix: None,
        }
    }

    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Info,
            suffix: None,
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Warning,
            suffix: None,
        }
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Success,
            suffix: None,
        }
    }

    pub fn purple(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: BadgeVariant::Purple,
            suffix: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionVariant {
    Primary,
    Danger,
    Secondary,
}

#[derive(Clone)]
pub struct TableAction {
    pub label: String,
    pub variant: ActionVariant,
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl TableAction {
    pub fn new(label: impl Into<String>, variant: ActionVariant) -> Self {
        Self {
            label: label.into(),
            variant,
            on_click: None,
        }
    }

    pub fn with_callback<F: Fn() + Send + Sync + 'static>(mut self, cb: F) -> Self {
        self.on_click = Some(Arc::new(cb));
        self
    }

    pub fn edit<F: Fn() + Send + Sync + 'static>(label: impl Into<String>, cb: F) -> Self {
        Self::new(label, ActionVariant::Primary).with_callback(cb)
    }

    pub fn delete<F: Fn() + Send + Sync + 'static>(label: impl Into<String>, cb: F) -> Self {
        Self::new(label, ActionVariant::Danger).with_callback(cb)
    }

    pub fn secondary<F: Fn() + Send + Sync + 'static>(label: impl Into<String>, cb: F) -> Self {
        Self::new(label, ActionVariant::Secondary).with_callback(cb)
    }
}

impl std::fmt::Debug for TableAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TableAction(label={:?}, variant={:?}, has_cb={})",
            self.label,
            self.variant,
            self.on_click.is_some()
        )
    }
}

pub type CustomCellRenderer = Arc<dyn Fn(isize, &RECT, &ThemePalette) + Send + Sync>;

#[derive(Clone)]
pub enum TableCell {
    Text(String),
    SecondaryText(String),
    Badge(BadgeConfig),
    Actions(Vec<TableAction>),
    Custom(CustomCellRenderer),
}

impl std::fmt::Debug for TableCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "Text({:?})", s),
            Self::SecondaryText(s) => write!(f, "SecondaryText({:?})", s),
            Self::Badge(b) => write!(f, "Badge({:?})", b),
            Self::Actions(acts) => write!(f, "Actions({:?})", acts),
            Self::Custom(_) => write!(f, "Custom(<closure>)"),
        }
    }
}

#[derive(Clone, Default)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

impl TableRow {
    pub fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub fn cell(mut self, cell: TableCell) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::Text(text.into()));
        self
    }

    pub fn secondary(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::SecondaryText(text.into()));
        self
    }

    pub fn key(mut self, text: impl Into<String>) -> Self {
        self.cells.push(TableCell::Badge(BadgeConfig::key(text)));
        self
    }

    pub fn key_with_arrow(mut self, text: impl Into<String>) -> Self {
        self.cells
            .push(TableCell::Badge(BadgeConfig::key_with_arrow(text)));
        self
    }

    pub fn danger_key(mut self, text: impl Into<String>) -> Self {
        self.cells
            .push(TableCell::Badge(BadgeConfig::danger_key(text)));
        self
    }

    pub fn badge(mut self, badge: BadgeConfig) -> Self {
        self.cells.push(TableCell::Badge(badge));
        self
    }

    pub fn actions(mut self, actions: Vec<TableAction>) -> Self {
        self.cells.push(TableCell::Actions(actions));
        self
    }
}

impl From<TableRow> for Vec<TableCell> {
    fn from(r: TableRow) -> Self {
        r.cells
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnWidth {
    Fixed(i32),
    Flex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CellAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColumnStyle {
    #[default]
    Default,
    KeyBadge,
    KeyBadgeWithArrow,
    DangerKeyBadge,
    CenterSecondary,
    OriginBadge,
    Action,
}

#[derive(Clone, Debug)]
pub struct ColumnDef {
    pub title: String,
    pub width: ColumnWidth,
    pub align: CellAlign,
    pub style: ColumnStyle,
}

impl ColumnDef {
    pub fn fixed(title: impl Into<String>, width: i32, style: ColumnStyle) -> Self {
        Self {
            title: title.into(),
            width: ColumnWidth::Fixed(width),
            align: match style {
                ColumnStyle::CenterSecondary | ColumnStyle::Action => CellAlign::Center,
                _ => CellAlign::Left,
            },
            style,
        }
    }

    pub fn flex(title: impl Into<String>, style: ColumnStyle) -> Self {
        Self {
            title: title.into(),
            width: ColumnWidth::Flex,
            align: match style {
                ColumnStyle::CenterSecondary | ColumnStyle::Action => CellAlign::Center,
                _ => CellAlign::Left,
            },
            style,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionHover {
    None,
    Action {
        row: usize,
        col: usize,
        action_idx: usize,
    },
}

pub type ActionCallback = Arc<dyn Fn(i32) + Send + Sync>;

pub struct ActionTableState {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<TableCell>>,
    pub font: isize,
    pub font_bold: isize,
    pub hover_row: Option<usize>,
    pub hover_action: ActionHover,
    pub selected_row: Option<usize>,
    pub scroll_y: i32,
    pub on_select: Option<ActionCallback>,
    pub is_dragging_scroll: bool,
    pub scroll_drag_start_y: i32,
    pub scroll_drag_start_val: i32,
    pub is_hover_scrollbar: bool,
    pub header_h: i32,
}
