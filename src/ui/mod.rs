//! Kora Pure Win32 UI System
//! Fully componentized, lightweight, zero-dependency native Windows UI.

pub mod controller;
pub mod theme;
pub mod tray;
pub mod views;
pub mod wnd_proc;

pub use crate::ui_framework::{colors, components, dialog, tokens, win32};
pub use win32::*;

pub use theme::{apply_ui_theme, is_current_dark, resolve_is_dark};
pub use tokens::*;

impl crate::ui_framework::I18nSource for crate::language::LanguageStrings {
    fn current() -> &'static Self {
        crate::language::current()
    }

    fn inherit_note(&self, inherited_on: bool) -> &'static str {
        if inherited_on {
            self.check_inherit_on
        } else {
            self.check_inherit_off
        }
    }
}

/// Application-level UI prelude specializing `CardLayout`, `Inheritable`, `TextFn`, and `TextListFn` to [`crate::language::LanguageStrings`].
pub mod prelude {
    pub use crate::ui_framework::dialog::*;
    pub use crate::ui_framework::prelude::{
        borders, cached_font, get_app_font_bold, get_app_font_normal, is_current_dark, layout,
        next_control_id, radii, resolve_id, set_hwnds_visible, spacing, to_wide, typography,
        ActionHover, ActionTable, ActionVariant, BadgeConfig, BadgeVariant, ButtonVariant,
        CellAlign, CheckBox, ColumnDef, ColumnStyle, ColumnWidth, ComboBox, ComponentHandle,
        InCard, I18nSource, Label, LabelKind, LabelTone, LayoutState, Link, ListBox, OutsideCard,
        PillBadge, PopupMenu, PushButton, ScrollView, Slider, StaticText, SurfaceRole, TabBar, TableAction,
        TableCell, TableRow, TextBox, ThemePalette, TwoState, RECT,
    };

    pub type CardLayout<S = OutsideCard> =
        crate::ui_framework::components::CardLayout<S, crate::language::LanguageStrings>;
    pub type Inheritable =
        crate::ui_framework::components::Inheritable<crate::language::LanguageStrings>;
    pub type TextFn = crate::ui_framework::components::TextFn<crate::language::LanguageStrings>;
    pub type TextListFn =
        crate::ui_framework::components::TextListFn<crate::language::LanguageStrings>;
}

use crate::ui::theme::recreate_theme_brushes;
use crate::ui::tray::{get_small_icon_size, load_icon_from_memory, TrayHandler};
use crate::ui::views::ControlPanelControls;
use crate::ui::wnd_proc::{control_panel_wnd_proc, tray_helper_wnd_proc};
use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicIsize, Ordering};

static APP_ICO_BYTES: &[u8] = include_bytes!("../../assets/icon.ico");

thread_local! {
    /// UI state, created once by `init_ui` and kept for the process lifetime.
    /// Thread-local: the whole UI (and the keyboard hook) runs on one thread, so no
    /// lock is needed and other threads simply get `None`.
    static UI: Cell<Option<&'static UiState>> = const { Cell::new(None) };
}

/// The UI state; `None` before `init_ui` finished or when called off the UI thread.
pub fn ui() -> Option<&'static UiState> {
    UI.with(Cell::get)
}

/// Shortcut for the control panel controls (see [`ui`]).
pub fn controls() -> Option<&'static ControlPanelControls> {
    ui().map(|u| &u.controls)
}

/// Hidden helper window (tray owner). Used as the target for messages posted from the hook.
static HELPER_HWND: AtomicIsize = AtomicIsize::new(0);

/// Posted to the helper window when the typing mode toggles (wparam = 1 for Vietnamese).
pub const WM_APP_MODE_CHANGED: u32 = 0x8000 + 120;

/// Posted to the control panel by the update-check worker; lparam = `Box<UpdateStatus>` raw ptr.
pub const WM_APP_UPDATE_CHECKED: u32 = 0x8000 + 121;

/// Posted to the control panel by the soundpack installer worker; lparam = `Box<SoundpackInstallStatus>` raw ptr.
pub const WM_APP_SOUNDPACK_INSTALLED: u32 = 0x8000 + 122;

/// Id of the root ("Mặc định") profile in `ProfileManager`.
pub const GLOBAL_PROFILE_ID: &str = "global";

pub struct UiState {
    pub h_panel: isize,
    pub tray: RefCell<TrayHandler>,
    pub controls: ControlPanelControls,
}

/// Mode-change callback for the keyboard hook. It runs inside the hook while ENGINE_INSTANCE
/// is locked, so it must not touch UI state synchronously: just post to the UI message queue.
pub fn post_mode_change(is_vietnamese: bool) {
    let h = HELPER_HWND.load(Ordering::Relaxed);
    if h != 0 {
        unsafe {
            PostMessageW(h, WM_APP_MODE_CHANGED, is_vietnamese as usize, 0);
        }
    }
}

/// Typing-mode combo index for the profile currently selected in the panel.
/// Root layout: [Vietnamese, English]; child layout: [Inherit, Vietnamese, English].
fn selected_mode_combo_index() -> Option<usize> {
    use crate::engine::profile::LanguageMode;
    let guard = crate::platform::win32::ENGINE_INSTANCE.lock().ok()?;
    let engine = guard.as_ref()?;
    let pm = &engine.profile_manager;
    Some(match pm.get_profile(&pm.selected_id) {
        Some(p) => match p.mode {
            LanguageMode::Inherit => 0,
            LanguageMode::Vietnamese => 1,
            LanguageMode::English => 2,
        },
        None => usize::from(!engine.config().enabled),
    })
}

pub fn show_control_panel() {
    let Some(ui) = ui() else { return };

    // Align thread DPI awareness context with the Control Panel's native context
    // (DPI_AWARENESS_CONTEXT_UNAWARE_GDISCALED) to prevent physical/logical coordinate
    // divergence when invoked from a Per-Monitor V2 window or context (e.g. tray menu).
    let panel_dpi_ctx = unsafe { GetWindowDpiAwarenessContext(ui.h_panel) };
    if panel_dpi_ctx != 0 {
        unsafe { SetThreadDpiAwarenessContext(panel_dpi_ctx); }
    }

    let c = &ui.controls;
    c.update_view_mode(crate::ui::views::control_panel::ViewMode::Profile(c.tab_bar.get_cur_sel()));
    let Some(cfg) = c.refresh_from_engine() else { return };


    // Language / theme are kept in sync by their own handlers (system tab, defaults,
    // WM_SETTINGCHANGE); only re-apply here if they drifted while hidden.
    if crate::language::current_language() != cfg.language {
        c.update_language(cfg.language);
        let strings = crate::language::get_strings(cfg.language);
        unsafe { SetWindowTextW(ui.h_panel, to_wide(strings.window_title).as_ptr()) };
    }
    let is_dark = resolve_is_dark(cfg.theme);
    if is_dark != is_current_dark() {
        apply_ui_theme(ui.h_panel, c, is_dark);
    }

    // Reconcile tray icon with effective typing mode in case it drifted while hidden
    if let Some(is_viet) = crate::ui::controller::with_engine(|e| e.effective_config().enabled) {
        ui.tray.borrow_mut().set_vietnamese_mode(is_viet);
    }

    unsafe {
        InvalidateRect(ui.h_panel, null_mut(), 0);
        ShowWindow(ui.h_panel, SW_SHOW);
        SetForegroundWindow(ui.h_panel);
    }
}

/// Hides the control panel and trims the process working set back to ~1-2 MB.
pub fn hide_control_panel() {
    if let Some(ui) = ui() {
        unsafe {
            ShowWindow(ui.h_panel, SW_HIDE);
        }
    }
    crate::platform::win32::trim_working_set();
}

/// Syncs the tray icon and the typing-mode combo after a mode change.
/// Must be called on the UI thread without holding ENGINE_INSTANCE.
pub fn update_tray_icon(is_vietnamese: bool) {
    let combo_idx = selected_mode_combo_index();
    if let Some(ui) = ui() {
        ui.tray.borrow_mut().set_vietnamese_mode(is_vietnamese);
        if let Some(idx) = combo_idx {
            ui.controls.tab_typing.combo_mode.set_selected(idx);
        }
    }
}

/// Initializes the complete Win32 UI system (Tray + Control Panel)
pub fn init_ui() -> bool {
    init_common_controls();

    let class_helper_name = to_wide("KoraHelperClass");
    let class_panel_name = to_wide("KoraControlPanelClass");

    let (cfg_theme, cfg_lang, cfg_enabled) = crate::platform::win32::ENGINE_INSTANCE
        .lock()
        .ok()
        .and_then(|g| {
            g.as_ref().map(|e| {
                (
                    e.config().theme,
                    e.config().language,
                    e.effective_config().enabled,
                )
            })
        })
        .unwrap_or((
            crate::engine::config::UiTheme::Auto,
            crate::language::Language::Vietnamese,
            true,
        ));
    crate::language::set_current_language(cfg_lang);
    let is_dark = resolve_is_dark(cfg_theme);
    set_preferred_app_mode(is_dark);

    unsafe {
        recreate_theme_brushes(is_dark);

        // 1. Register Helper Window Class
        let mut wc_helper: WNDCLASSEXW = std::mem::zeroed();
        wc_helper.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc_helper.lpfn_wnd_proc = Some(tray_helper_wnd_proc);
        wc_helper.lpsz_class_name = class_helper_name.as_ptr();
        wc_helper.h_cursor = LoadCursorW(0, IDC_ARROW);
        RegisterClassExW(&wc_helper);

        // 2. Load Application Icons for Taskbar and Title Bar (DPI-aware)
        let (sm_w, sm_h) = get_small_icon_size();
        let hicon_big = load_icon_from_memory(APP_ICO_BYTES, 32, 32).unwrap_or(0);
        let hicon_sm = load_icon_from_memory(APP_ICO_BYTES, sm_w, sm_h).unwrap_or(0);

        // Register Control Panel Window Class
        let mut wc_panel: WNDCLASSEXW = std::mem::zeroed();
        wc_panel.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc_panel.lpfn_wnd_proc = Some(control_panel_wnd_proc);
        wc_panel.lpsz_class_name = class_panel_name.as_ptr();
        wc_panel.h_br_background = 0; // Handled dynamically by WM_ERASEBKGND with PANEL_BG_BRUSH
        wc_panel.h_icon = hicon_big;
        wc_panel.h_icon_sm = hicon_sm;
        wc_panel.h_cursor = LoadCursorW(0, IDC_ARROW);
        RegisterClassExW(&wc_panel);

        // 3. Create Hidden Helper Window under Per-Monitor V2 DPI awareness so tray menus
        // position and scale accurately on mixed-DPI multi-monitor setups regardless of
        // which monitor the GDI-scaled Control Panel resides on.
        let prev_dpi_ctx = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let h_helper = CreateWindowExW(
            0,
            class_helper_name.as_ptr(),
            to_wide(crate::language::current().helper_window_title).as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            null_mut(),
        );
        SetThreadDpiAwarenessContext(prev_dpi_ctx);

        if h_helper == 0 {
            eprintln!("[UI Error] Không thể tạo cửa sổ Helper!");
            return false;
        }

        // Register helper window for Windows session change notifications (lock, unlock, logon)
        WTSRegisterSessionNotification(h_helper, NOTIFY_FOR_THIS_SESSION);

        // 4. Create Tray Handler
        let tray = TrayHandler::new(h_helper, cfg_enabled);

        // 5. Fonts (Segoe UI 9pt normal, 9pt semibold) from the shared font cache
        let hfont_normal = crate::ui::theme::get_app_font_normal();
        let hfont_bold = crate::ui::theme::get_app_font_bold();

        // 6. Create Control Panel Window with WS_EX_APPWINDOW to display on Windows Taskbar
        // Ensure client area is exactly 820x590 for 2-column multi-profile layout
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: crate::ui::tokens::layout::WINDOW_CLIENT_WIDTH,
            bottom: crate::ui::tokens::layout::WINDOW_CLIENT_HEIGHT,
        };
        AdjustWindowRect(
            &mut rc,
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            0,
        );
        let panel_width = rc.right - rc.left;
        let panel_height = rc.bottom - rc.top;
        let h_panel = CreateWindowExW(
            WS_EX_APPWINDOW | WS_EX_CONTROLPARENT,
            class_panel_name.as_ptr(),
            to_wide(crate::language::current().window_title).as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN,
            0,
            0,
            panel_width,
            panel_height,
            0,
            0,
            0,
            null_mut(),
        );

        if h_panel == 0 {
            eprintln!("[UI Error] Không thể tạo cửa sổ Control Panel!");
            return false;
        }

        // Set official App Icon on the Window & Windows Taskbar button
        if hicon_big != 0 {
            SendMessageW(h_panel, WM_SETICON, ICON_BIG, hicon_big);
        }
        if hicon_sm != 0 {
            SendMessageW(h_panel, WM_SETICON, ICON_SMALL, hicon_sm);
        }

        // Center and guarantee exact client area (820x590) regardless of OS non-client frame metrics
        let mut client_rc = RECT::default();
        GetClientRect(h_panel, &mut client_rc);
        let actual_w = client_rc.right - client_rc.left;
        let actual_h = client_rc.bottom - client_rc.top;
        let target_w = if actual_w < crate::ui::tokens::layout::WINDOW_CLIENT_WIDTH {
            panel_width + (crate::ui::tokens::layout::WINDOW_CLIENT_WIDTH - actual_w)
        } else {
            panel_width
        };
        let target_h = if actual_h < crate::ui::tokens::layout::WINDOW_CLIENT_HEIGHT {
            panel_height + (crate::ui::tokens::layout::WINDOW_CLIENT_HEIGHT - actual_h)
        } else {
            panel_height
        };
        center_window(h_panel, target_w, target_h);

        // Create all components inside Control Panel (texts use the current language;
        // data is loaded by `show_control_panel` when the panel is first shown)
        let controls = match ControlPanelControls::create(h_panel, hfont_normal, hfont_bold) {
            Some(c) => c,
            None => {
                eprintln!("[UI Error] Không thể khởi tạo các controls của Bảng điều khiển!");
                return false;
            }
        };

        // Apply theme styling across all controls
        apply_ui_theme(h_panel, &controls, is_dark);

        // Initial view mode: Profile Tab 0 (Typing)
        controls.update_view_mode(crate::ui::views::control_panel::ViewMode::Profile(0));

        // Lives for the rest of the process; leaked once so handlers get a plain `&'static`.
        let state: &'static UiState = Box::leak(Box::new(UiState {
            h_panel,
            tray: RefCell::new(tray),
            controls,
        }));
        SetWindowLongPtrW(
            h_panel,
            GWLP_USERDATA,
            &state.controls as *const _ as isize,
        );
        UI.with(|cell| cell.set(Some(state)));
        HELPER_HWND.store(h_helper, Ordering::Relaxed);
        crate::platform::win32::capture::set_target_capture_notify_hwnd(h_panel);

        true
    }
}
