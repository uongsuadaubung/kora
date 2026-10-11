//! Kora Pure Win32 UI System
//! Fully componentized, lightweight, zero-dependency native Windows UI.

pub mod colors;
pub mod components;
pub mod theme;
pub mod tokens;
pub mod tray;
pub mod views;
pub mod win32;
pub mod wnd_proc;

pub use win32::*;

pub use theme::{apply_ui_theme, is_current_dark, resolve_is_dark};
pub use tokens::*;

use crate::engine::profile::AppProfile;
use crate::ui::theme::recreate_theme_brushes;
use crate::ui::tray::{get_small_icon_size, load_icon_from_memory, TrayHandler};
use crate::ui::views::ControlPanelControls;
use crate::ui::wnd_proc::{control_panel_wnd_proc, tray_helper_wnd_proc};
use std::ptr::null_mut;
use std::sync::Mutex;

static APP_ICO_BYTES: &[u8] = include_bytes!("../../assets/icon.ico");

pub static UI_MANAGER: Mutex<Option<UiState>> = Mutex::new(None);


pub struct UiState {
    pub h_helper: isize,
    pub h_panel: isize,
    pub tray: TrayHandler,
    pub controls: ControlPanelControls,
    pub hfont_normal: isize,
    pub hfont_bold: isize,
}

impl UiState {
    pub fn on_command(&mut self, hwnd: isize, control_id: u32, notify_code: u32) -> bool {
        let handled = self.controls.on_command(hwnd, control_id, notify_code);
        if control_id == crate::ui::tokens::id::IDC_COMBO_TYPING_MODE {
            if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                && let Some(ref engine) = *guard
            {
                self.tray.set_vietnamese_mode(engine.effective_config().enabled);
            }
        } else if control_id == crate::ui::tokens::id::IDC_COMBO_THEME
            || control_id == crate::ui::tokens::id::IDC_COMBO_LANG
        {
            self.tray.refresh_icon();
        }
        handled
    }
}

pub fn show_control_panel() {
    let (cfg, selected_profile, is_active, all_profiles, active_id, remaps, disabled, macros) = {
        if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock() {
            if let Some(ref engine) = *guard {
                let sel_id = engine.profile_manager.selected_id.clone();
                let act_id = engine.profile_manager.active_id.clone();
                let is_act = sel_id == act_id;
                let prof = engine
                    .profile_manager
                    .get_profile(&sel_id)
                    .cloned()
                    .unwrap_or_else(|| engine.profile_manager.global_profile.clone());
                let all: Vec<AppProfile> = engine
                    .profile_manager
                    .all_profiles()
                    .into_iter()
                    .cloned()
                    .collect();
                let r = engine.profile_manager.resolve_remap_items_for_ui(&sel_id);
                let d = engine.profile_manager.resolve_disable_items_for_ui(&sel_id);
                let m = engine
                    .profile_manager
                    .resolve_macro_items_for_ui(&sel_id, &engine.macro_table.get_sorted_entries());
                (
                    Some(engine.config().clone()),
                    Some(prof),
                    is_act,
                    all,
                    act_id,
                    r,
                    d,
                    m,
                )
            } else {
                (
                    None,
                    None,
                    false,
                    Vec::new(),
                    String::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
        } else {
            (
                None,
                None,
                false,
                Vec::new(),
                String::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        }
    };

    if let Ok(ui_guard) = UI_MANAGER.try_lock()
        && let Some(ref ui) = *ui_guard
    {
        unsafe {
            if let Some(ref c) = cfg {
                ui.controls.load_config(c);
                crate::language::set_current_language(c.language);
                ui.controls.update_language(c.language);
                let strings = crate::language::get_strings(c.language);
                SetWindowTextW(ui.h_panel, to_wide(strings.window_title).as_ptr());
                let is_dark = resolve_is_dark(c.theme);
                apply_ui_theme(ui.h_panel, &ui.controls, is_dark);

                if let Some(ref prof) = selected_profile {
                    let all_refs: Vec<&AppProfile> = all_profiles.iter().collect();
                    ui.controls
                        .load_profile_data(prof, is_active, c, &all_refs, &active_id);
                }
            }
            ui.controls.tab_remap.populate_remaps(&remaps);
            ui.controls.tab_remap.populate_disabled(&disabled);
            ui.controls.populate_macro_items(&macros);
            ui.controls
                .update_view_mode(crate::ui::views::control_panel::ViewMode::Profile(
                    ui.controls.tab_bar.get_cur_sel(),
                ));
            let h_panel = ui.h_panel;
            InvalidateRect(h_panel, null_mut(), 0);
            drop(ui_guard);
            ShowWindow(h_panel, SW_SHOW);
            SetForegroundWindow(h_panel);
        }
    }
}

pub fn update_tray_icon(is_vietnamese: bool) {
    if let Ok(mut ui_guard) = UI_MANAGER.lock()
        && let Some(ref mut ui) = *ui_guard
    {
        ui.tray.set_vietnamese_mode(is_vietnamese);
        ui.controls
            .tab_typing
            .combo_mode
            .set_selected(if is_vietnamese { 0 } else { 1 });
    }
}

/// Initializes the complete Win32 UI system (Tray + Control Panel)
pub fn init_ui() -> bool {
    init_common_controls();

    let class_helper_name = to_wide("KoraHelperClass");
    let class_panel_name = to_wide("KoraControlPanelClass");

    let (cfg_theme, cfg_lang, init_config) = {
        let guard = crate::platform::win32::ENGINE_INSTANCE.lock().unwrap();
        if let Some(ref e) = *guard {
            (
                e.config().theme,
                e.config().language,
                Some(e.config().clone()),
            )
        } else {
            (
                crate::engine::config::UiTheme::Auto,
                crate::language::Language::Vietnamese,
                None,
            )
        }
    };
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

        // 3. Create Hidden Helper Window
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

        if h_helper == 0 {
            eprintln!("[UI Error] Không thể tạo cửa sổ Helper!");
            return false;
        }

        // 4. Create Tray Handler
        let tray = TrayHandler::new(h_helper);

        // 5. Create Fonts (Segoe UI 9pt normal, 9pt semibold)
        let hfont_normal = create_app_font(crate::ui::tokens::typography::FONT_SIZE_BODY, FW_NORMAL);
        let hfont_bold = create_app_font(crate::ui::tokens::typography::FONT_SIZE_BODY, FW_SEMIBOLD);
        crate::ui::theme::set_app_fonts(hfont_normal, hfont_bold);

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

        // Create all components inside Control Panel
        let controls = match ControlPanelControls::create(h_panel, hfont_normal, hfont_bold) {
            Some(c) => c,
            None => {
                eprintln!("[UI Error] Không thể khởi tạo các controls của Bảng điều khiển!");
                return false;
            }
        };

        // Load initial config and synchronize language
        if let Some(ref c) = init_config {
            controls.load_config(c);
            controls.update_language(cfg_lang);
        }

        // Apply theme styling across all controls
        apply_ui_theme(h_panel, &controls, is_dark);

        // Initial view mode: Profile Tab 0 (Typing)
        controls.update_view_mode(crate::ui::views::control_panel::ViewMode::Profile(0));

        let mut ui_guard = UI_MANAGER.lock().unwrap();
        *ui_guard = Some(UiState {
            h_helper,
            h_panel,
            tray,
            controls,
            hfont_normal,
            hfont_bold,
        });

        if let Some(ref ui) = *ui_guard {
            SetWindowLongPtrW(
                h_panel,
                GWLP_USERDATA,
                &ui.controls as *const _ as isize,
            );
        }

        true
    }
}
