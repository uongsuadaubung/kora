//! Modal Dialog for Adding a New Application Profile
//! Allows user to pick a running app or specify an executable name.
//! Layout follows mockup_profiles.html: custom header, hint box, icon + name,
//! .exe field with file browser, searchable running-app list and footer bar.

use crate::engine::profile::AppProfile;
use crate::platform::win32::app_detect::get_process_name_for_pid;
use crate::platform::win32::types::GetWindowThreadProcessId;
use crate::ui::colors::ThemePalette;
use crate::ui::components::*;
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, get_panel_bg_brush, is_current_dark};
use crate::ui::tokens::id::*;
use crate::ui::win32::*;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

static DIALOG_PARENT_HWND: AtomicIsize = AtomicIsize::new(0);
static SELECTED_ICON: AtomicUsize = AtomicUsize::new(DEFAULT_ICON);

/// Emoji per entry of `Strings::profile_icons()` (same order)
const ICONS: [&str; 6] = ["🎮", "💻", "📝", "🎨", "🌐", "⚙️"];
const DEFAULT_ICON: usize = 5;

// Layout (client coordinates)
const W: i32 = 520;
const PAD: i32 = 16;
const HEADER_H: i32 = 52;
const HINT_Y: i32 = 68;
const HINT_H: i32 = 58;
const NAME_LBL_Y: i32 = 140;
const NAME_Y: i32 = 162;
const EXE_LBL_Y: i32 = 212;
const EXE_Y: i32 = 240;
const APPS_LBL_Y: i32 = 290;
const SEARCH_Y: i32 = 314;
const FIELD_H: i32 = 34;
const LIST_Y: i32 = SEARCH_Y + FIELD_H - 1;
const LIST_H: i32 = 5 * 32;
const FOOTER_Y: i32 = LIST_Y + LIST_H + PAD;
const FOOTER_H: i32 = 60;
const H: i32 = FOOTER_Y + FOOTER_H;

const DT_RIGHT: u32 = 0x0002;
const DT_CALCRECT: u32 = 0x0400;
const DT_END_ELLIPSIS: u32 = 0x4000;
const EN_CHANGE: usize = 0x0300;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumWindows(
        lp_enum_func: Option<unsafe extern "system" fn(hwnd: isize, lparam: isize) -> i32>,
        lparam: isize,
    ) -> i32;
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
struct OPENFILENAMEW {
    l_struct_size: u32,
    hwnd_owner: isize,
    h_instance: isize,
    lpstr_filter: *const u16,
    lpstr_custom_filter: *mut u16,
    n_max_cust_filter: u32,
    n_filter_index: u32,
    lpstr_file: *mut u16,
    n_max_file: u32,
    lpstr_file_title: *mut u16,
    n_max_file_title: u32,
    lpstr_initial_dir: *const u16,
    lpstr_title: *const u16,
    flags: u32,
    n_file_offset: u16,
    n_file_extension: u16,
    lpstr_def_ext: *const u16,
    l_cust_data: isize,
    lpfn_hook: *const std::ffi::c_void,
    lp_template_name: *const u16,
    pv_reserved: *mut std::ffi::c_void,
    dw_reserved: u32,
    flags_ex: u32,
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    fn GetOpenFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
}

pub fn show_add_profile_dialog(parent: isize) {
    let class_name = to_wide("KoraAddProfileDialogClass");

    unsafe {
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.style = 0x0002_0000; // CS_DROPSHADOW
        wc.lpfn_wnd_proc = Some(add_profile_dialog_wnd_proc);
        wc.h_instance = 0;
        wc.h_cursor = LoadCursorW(0, 32512);
        wc.lpsz_class_name = class_name.as_ptr();
        RegisterClassExW(&wc);

        DIALOG_PARENT_HWND.store(parent, Ordering::SeqCst);
        SELECTED_ICON.store(DEFAULT_ICON, Ordering::SeqCst);

        let mut rc: RECT = std::mem::zeroed();
        GetWindowRect(parent, &mut rc);
        let x = rc.left + ((rc.right - rc.left) - W) / 2;
        let y = rc.top + ((rc.bottom - rc.top) - H) / 2;

        EnableWindow(parent, 0);

        let strings = crate::language::current();
        // Borderless popup: header/close button are drawn by the dialog itself
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            class_name.as_ptr(),
            to_wide(strings.dlg_add_prof_title).as_ptr(),
            0x8000_0000 /* WS_POPUP */ | WS_CLIPCHILDREN,
            x,
            y,
            W,
            H,
            parent,
            0,
            0,
            std::ptr::null_mut(),
        );

        if hwnd != 0 {
            apply_modern_window_styling(hwnd, is_current_dark());
            let disable: i32 = 1; // DWMWA_TRANSITIONS_FORCEDISABLED: no fade on show/hide
            DwmSetWindowAttribute(hwnd, 3, &disable as *const i32 as *const _, 4);
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
            SetFocus(GetDlgItem(hwnd, IDC_EDIT_PROFILE_NAME as i32));
        } else {
            EnableWindow(parent, 1);
        }
    }
}

#[derive(Clone)]
struct AppWindowEntry {
    title: String,
    exe_name: String,
}

static RUNNING_APPS: Mutex<Vec<AppWindowEntry>> = Mutex::new(Vec::new());
/// Entries currently shown in the list (after search filter), indexed by row
static VISIBLE_APPS: Mutex<Vec<AppWindowEntry>> = Mutex::new(Vec::new());
static APP_TABLE: Mutex<Option<ActionTable>> = Mutex::new(None);

unsafe extern "system" fn enum_windows_proc(hwnd: isize, _lparam: isize) -> i32 {
    unsafe {
        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }
        let title = window_text(hwnd);
        if title.is_empty() || title.starts_with("Kora") {
            return 1;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let exe = get_process_name_for_pid(pid);
        if !exe.is_empty()
            && let Ok(mut list) = RUNNING_APPS.lock()
            && !list.iter().any(|a| a.exe_name == exe)
        {
            list.push(AppWindowEntry {
                title,
                exe_name: exe,
            });
        }
        1
    }
}

fn window_text(hwnd: isize) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1);
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}

fn mono_font() -> isize {
    static FONT: OnceLock<isize> = OnceLock::new();
    *FONT.get_or_init(|| unsafe {
        CreateFontW(
            -12, 0, 0, 0, FW_NORMAL, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
            to_wide("Consolas").as_ptr(),
        )
    })
}

/// One list row: window title on the left, monospace exe name right-aligned
fn app_row(app: &AppWindowEntry) -> TableRow {
    let title = to_wide(&app.title);
    let exe = to_wide(&app.exe_name);
    TableRow::new().cell(TableCell::Custom(Arc::new(move |hdc: isize, rc: &RECT, palette: &ThemePalette| unsafe {
        SetBkMode(hdc, TRANSPARENT);
        let old = SelectObject(hdc, mono_font());
        let mut calc = *rc;
        DrawTextW(hdc, exe.as_ptr(), -1, &mut calc, DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX);
        let exe_w = (calc.right - calc.left).min((rc.right - rc.left) / 2);
        let mut exe_rc = RECT { left: rc.right - 14 - exe_w, right: rc.right - 14, ..*rc };
        SetTextColor(hdc, palette.text_secondary);
        DrawTextW(hdc, exe.as_ptr(), -1, &mut exe_rc,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS);

        SelectObject(hdc, get_app_font_normal());
        let mut title_rc = RECT { left: rc.left + 14, right: exe_rc.left - 12, ..*rc };
        SetTextColor(hdc, palette.text_primary);
        DrawTextW(hdc, title.as_ptr(), -1, &mut title_rc,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS);
        SelectObject(hdc, old);
    })))
}

/// Re-enumerates windows (if `rescan`) and refills the list using the search box filter
fn refresh_running_apps(hwnd: isize, rescan: bool) {
    if rescan {
        if let Ok(mut list) = RUNNING_APPS.lock() {
            list.clear();
        }
        unsafe {
            EnumWindows(Some(enum_windows_proc), 0);
        }
    }
    let query = window_text(unsafe { GetDlgItem(hwnd, IDC_EDIT_SEARCH_APPS as i32) })
        .trim()
        .to_lowercase();
    let visible: Vec<AppWindowEntry> = RUNNING_APPS
        .lock()
        .map(|list| {
            list.iter()
                .filter(|a| {
                    a.title.to_lowercase().contains(&query)
                        || a.exe_name.to_lowercase().contains(&query)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    if let Ok(guard) = APP_TABLE.lock()
        && let Some(ref tbl) = *guard
    {
        tbl.clear();
        tbl.set_rows(visible.iter().map(app_row));
    }
    if let Ok(mut v) = VISIBLE_APPS.lock() {
        *v = visible;
    }
}

/// Opens the native file picker and returns the chosen file name (e.g. "app.exe")
fn browse_exe(owner: isize) -> Option<String> {
    let mut file = [0u16; 260];
    let filter: Vec<u16> = "Application (*.exe)\0*.exe\0\0".encode_utf16().collect();
    unsafe {
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.l_struct_size = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwnd_owner = owner;
        ofn.lpstr_filter = filter.as_ptr();
        ofn.lpstr_file = file.as_mut_ptr();
        ofn.n_max_file = file.len() as u32;
        ofn.flags = 0x1000 /* OFN_FILEMUSTEXIST */ | 0x800 /* OFN_PATHMUSTEXIST */ | 0x8 /* OFN_NOCHANGEDIR */;
        if GetOpenFileNameW(&mut ofn) == 0 {
            return None;
        }
    }
    let len = file.iter().position(|&c| c == 0).unwrap_or(0);
    let path = String::from_utf16_lossy(&file[..len]);
    std::path::Path::new(&path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
}

unsafe fn fill(hdc: isize, rc: &RECT, color: u32) {
    unsafe {
        let br = CreateSolidBrush(color);
        FillRect(hdc, rc, br);
        DeleteObject(br);
    }
}

unsafe fn round_box(hdc: isize, rc: &RECT, bg: u32, border: u32, radius: i32) {
    unsafe {
        let br = CreateSolidBrush(bg);
        let pen = CreatePen(PS_SOLID, 1, border);
        let ob = SelectObject(hdc, br);
        let op = SelectObject(hdc, pen);
        RoundRect(hdc, rc.left, rc.top, rc.right, rc.bottom, radius, radius);
        SelectObject(hdc, ob);
        SelectObject(hdc, op);
        DeleteObject(br);
        DeleteObject(pen);
    }
}

unsafe fn draw_text(hdc: isize, text: &str, rc: &RECT, font: isize, color: u32, flags: u32) {
    unsafe {
        let old = SelectObject(hdc, font);
        SetTextColor(hdc, color);
        let mut r = *rc;
        DrawTextW(hdc, to_wide(text).as_ptr(), -1, &mut r, flags | DT_NOPREFIX);
        SelectObject(hdc, old);
    }
}

unsafe fn paint(hwnd: isize) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        if hdc == 0 {
            return;
        }
        let palette = ThemePalette::get(is_current_dark());
        let strings = crate::language::current();
        let mut rc = RECT::default();
        GetClientRect(hwnd, &mut rc);
        SetBkMode(hdc, TRANSPARENT);
        fill(hdc, &rc, palette.bg_window);

        // Header: accent "+" badge, title, bottom divider
        fill(hdc, &RECT { left: 0, top: 0, right: W, bottom: HEADER_H }, palette.bg_sidebar);
        fill(hdc, &RECT { left: 0, top: HEADER_H - 1, right: W, bottom: HEADER_H }, palette.border);
        let badge = RECT { left: PAD, top: 15, right: PAD + 22, bottom: 37 };
        round_box(hdc, &badge, palette.accent, palette.accent, 6);
        draw_text(hdc, "+", &badge, get_app_font_bold(), palette.text_white,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        let title_font = create_app_font(11, FW_BOLD);
        draw_text(hdc, strings.dlg_add_prof_title,
            &RECT { left: PAD + 34, top: 0, right: W - 60, bottom: HEADER_H },
            title_font, palette.text_primary, DT_LEFT | DT_VCENTER | DT_SINGLELINE);
        DeleteObject(title_font);

        // Hint box (text vertically centered)
        let hint = RECT { left: PAD, top: HINT_Y, right: W - PAD, bottom: HINT_Y + HINT_H };
        round_box(hdc, &hint, palette.bg_card_header, palette.border, 8);
        let mut inner = RECT { left: hint.left + 12, top: hint.top + 6, right: hint.right - 12, bottom: hint.bottom - 6 };
        let hint_w = to_wide(strings.dlg_add_prof_hint);
        let old = SelectObject(hdc, get_app_font_normal());
        let mut calc = inner;
        DrawTextW(hdc, hint_w.as_ptr(), -1, &mut calc, DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX);
        inner.top += ((inner.bottom - inner.top) - (calc.bottom - calc.top)).max(0) / 2;
        SetTextColor(hdc, palette.text_secondary);
        DrawTextW(hdc, hint_w.as_ptr(), -1, &mut inner, DT_WORDBREAK | DT_NOPREFIX);
        SelectObject(hdc, old);

        // Footer bar
        fill(hdc, &RECT { left: 0, top: FOOTER_Y, right: W, bottom: H }, palette.bg_sidebar);
        fill(hdc, &RECT { left: 0, top: FOOTER_Y, right: W, bottom: FOOTER_Y + 1 }, palette.border);
        draw_text(hdc, strings.dlg_add_prof_footer,
            &RECT { left: PAD + 4, top: FOOTER_Y, right: W - PAD - 110 - 8 - 90 - 8, bottom: H },
            get_app_font_normal(), palette.text_secondary,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS);

        // Window outline
        let br = CreateSolidBrush(palette.border);
        FrameRect(hdc, &rc, br);
        DeleteObject(br);

        EndPaint(hwnd, &ps);
    }
}

unsafe fn create_controls(hwnd: isize) {
    unsafe {
        let palette = ThemePalette::get(is_current_dark());
        let strings = crate::language::current();
        let font_label = get_app_font_bold();
        let font_normal = get_app_font_normal();
        let cw = W - 2 * PAD;

        // Header close button
        PushButton::create_with_variant(
            hwnd, IDC_BTN_CLOSE_ADD_PROFILE, "✕", W - PAD - 32, 10, 32, 32,
            font_normal, ButtonVariant::Ghost, Some(palette.bg_sidebar),
        );

        // Icon + display name
        Label::create(hwnd, strings.dlg_add_prof_icon, PAD, NAME_LBL_Y, 56, 18, font_label);
        Label::create(hwnd, strings.dlg_add_prof_name, PAD + 68, NAME_LBL_Y, 300, 18, font_label);
        PushButton::create_with_variant(
            hwnd, IDC_BTN_PROFILE_ICON, ICONS[DEFAULT_ICON], PAD, NAME_Y, 56, FIELD_H,
            font_normal, ButtonVariant::Secondary, Some(palette.bg_window),
        );
        TextBox::create_with_placeholder(
            hwnd, IDC_EDIT_PROFILE_NAME, strings.dlg_add_prof_name_placeholder,
            PAD + 68, NAME_Y, cw - 68, FIELD_H, font_normal,
        );

        // Executable + file browser
        Label::create(hwnd, strings.dlg_add_prof_exe, PAD, EXE_LBL_Y, 300, 18, font_label);
        PushButton::create_with_variant(
            hwnd, IDC_BTN_BROWSE_EXE, strings.dlg_add_prof_browse, W - PAD - 150, EXE_LBL_Y - 6, 150, 28,
            font_normal, ButtonVariant::Secondary, Some(palette.bg_window),
        );
        TextBox::create_with_placeholder(
            hwnd, IDC_EDIT_PROFILE_EXE, strings.dlg_add_prof_exe_placeholder,
            PAD, EXE_Y, cw, FIELD_H, font_normal,
        );

        // Running apps: label, rescan link, search box, list
        Label::create(hwnd, strings.dlg_add_prof_linked_app, PAD, APPS_LBL_Y, 340, 18, font_label);
        Label::create_right_with_id(
            hwnd, IDC_LINK_RESCAN_APPS, strings.dlg_add_prof_rescan, W - PAD - 120, APPS_LBL_Y, 120, 18,
            font_normal,
        );
        TextBox::create_with_placeholder(
            hwnd, IDC_EDIT_SEARCH_APPS, strings.dlg_add_prof_search_placeholder,
            PAD, SEARCH_Y, cw, FIELD_H, font_normal,
        );

        let table = ActionTable::create_with_defs(
            hwnd, IDC_LIST_RUNNING_APPS, PAD, LIST_Y, cw, LIST_H, font_normal,
            vec![ColumnDef::flex("", ColumnStyle::Default)],
        );
        if let Some(ref tbl) = table {
            tbl.hide_header();
            tbl.set_on_select(move |sel| {
                let app = VISIBLE_APPS.lock().ok().and_then(|v| v.get(sel as usize).cloned());
                if let Some(app) = app {
                    let h_name = GetDlgItem(hwnd, IDC_EDIT_PROFILE_NAME as i32);
                    let h_exe = GetDlgItem(hwnd, IDC_EDIT_PROFILE_EXE as i32);
                    if GetWindowTextLengthW(h_name) == 0 {
                        let clean_title = app.title.split(['-', '|', '—']).next().unwrap_or(&app.title).trim();
                        SetWindowTextW(h_name, to_wide(clean_title).as_ptr());
                    }
                    SetWindowTextW(h_exe, to_wide(&app.exe_name).as_ptr());
                }
            });
        }
        if let Ok(mut guard) = APP_TABLE.lock() {
            *guard = table;
        }
        refresh_running_apps(hwnd, true);

        // Footer buttons
        let btn_y = FOOTER_Y + (FOOTER_H - 32) / 2;
        PushButton::create_with_variant(
            hwnd, IDC_BTN_CANCEL_PROFILE, strings.dlg_btn_cancel, W - PAD - 110 - 8 - 90, btn_y, 90, 32,
            font_normal, ButtonVariant::Secondary, Some(palette.bg_sidebar),
        );
        PushButton::create_with_variant(
            hwnd, IDC_BTN_CREATE_PROFILE, strings.dlg_add_prof_btn_create, W - PAD - 110, btn_y, 110, 32,
            font_label, ButtonVariant::Primary, Some(palette.bg_sidebar),
        );
    }
}

unsafe fn pick_icon(hwnd: isize) {
    unsafe {
        let Some(mut menu) = PopupMenu::new() else { return };
        let current = SELECTED_ICON.load(Ordering::SeqCst);
        for (i, label) in crate::language::current().profile_icons().iter().enumerate() {
            menu.add_checked_item(i as u32 + 1, label, i == current);
        }
        let btn = GetDlgItem(hwnd, IDC_BTN_PROFILE_ICON as i32);
        let mut rc = RECT::default();
        GetWindowRect(btn, &mut rc);
        let cmd = menu.track(hwnd, rc.left, rc.bottom + 2) as usize;
        if (1..=ICONS.len()).contains(&cmd) {
            SELECTED_ICON.store(cmd - 1, Ordering::SeqCst);
            SetWindowTextW(btn, to_wide(ICONS[cmd - 1]).as_ptr());
            InvalidateRect(btn, std::ptr::null(), 0);
        }
    }
}

unsafe fn create_profile(hwnd: isize) {
    unsafe {
        let name = window_text(GetDlgItem(hwnd, IDC_EDIT_PROFILE_NAME as i32)).trim().to_string();
        let exe = window_text(GetDlgItem(hwnd, IDC_EDIT_PROFILE_EXE as i32)).trim().to_lowercase();

        if !name.is_empty() {
            let icon = ICONS[SELECTED_ICON.load(Ordering::SeqCst).min(ICONS.len() - 1)];

            let id = exe
                .trim_end_matches(".exe")
                .replace([' ', '-', '.'], "_")
                .to_lowercase();
            let id = if id.is_empty() {
                name.to_lowercase().replace(' ', "_")
            } else {
                id
            };

            let profile = AppProfile::new_child(id.clone(), name, exe, icon);

            if let Ok(mut guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
                && let Some(ref mut engine) = *guard
            {
                engine.profile_manager.add_profile(profile);
                engine.profile_manager.selected_id = id.clone();
                crate::engine::config_store::save_config_all_debounced(
                    engine.config(),
                    &engine.macro_table,
                    &engine.profile_manager,
                );
            }

            // Hide first so the user sees the result immediately, then refresh the panel
            // (engine lock above is already released; switch_to_profile re-locks it).
            // Owner must be enabled before hiding, otherwise Windows activates another app.
            let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
            EnableWindow(parent, 1);
            ShowWindow(hwnd, SW_HIDE);
            let controls = GetWindowLongPtrW(parent, GWLP_USERDATA)
                as *const crate::ui::views::ControlPanelControls;
            if parent != 0 && !controls.is_null() {
                crate::ui::views::control_panel::SidebarControls::switch_to_profile(&*controls, parent, &id);
            }
        }

        PostMessageW(hwnd, WM_CLOSE, 0, 0);
    }
}

const WM_CLOSE: u32 = 0x0010;

pub unsafe extern "system" fn add_profile_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_COMMAND: u32 = 0x0111;
    const WM_ERASEBKGND: u32 = 0x0014;
    const WM_DRAWITEM: u32 = 0x002B;
    const WM_SETCURSOR: u32 = 0x0020;
    const WM_NCHITTEST: u32 = 0x0084;
    const WM_CTLCOLORSTATIC: u32 = 0x0138;
    const HTCLIENT: isize = 1;
    const HTCAPTION: isize = 2;

    unsafe {
        match msg {
            WM_CREATE => {
                create_controls(hwnd);
                0
            }
            WM_NCHITTEST => {
                // Header strip drags the borderless window
                let mut pt = POINT {
                    x: (lparam & 0xFFFF) as i16 as i32,
                    y: ((lparam >> 16) & 0xFFFF) as i16 as i32,
                };
                ScreenToClient(hwnd, &mut pt);
                if pt.y < HEADER_H { HTCAPTION } else { HTCLIENT }
            }
            WM_SETCURSOR => {
                if wparam as isize == GetDlgItem(hwnd, IDC_LINK_RESCAN_APPS as i32) {
                    SetCursor(LoadCursorW(0, IDC_HAND));
                    return 1;
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_CTLCOLORSTATIC => {
                let palette = ThemePalette::get(is_current_dark());
                let hdc = wparam as isize;
                let color = if GetDlgCtrlID(lparam) == IDC_LINK_RESCAN_APPS as i32 {
                    palette.text_link
                } else {
                    palette.text_secondary
                };
                SetTextColor(hdc, color);
                SetBkColor(hdc, palette.bg_window);
                get_panel_bg_brush()
            }
            WM_COMMAND => {
                // TextBox forwards EN_* with its container hwnd in lparam
                if wparam >> 16 == EN_CHANGE && lparam == GetDlgItem(hwnd, IDC_EDIT_SEARCH_APPS as i32) {
                    refresh_running_apps(hwnd, false);
                    return 0;
                }
                match (wparam & 0xFFFF) as u32 {
                    IDC_BTN_CANCEL_PROFILE | IDC_BTN_CLOSE_ADD_PROFILE => {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    }
                    IDC_BTN_PROFILE_ICON => pick_icon(hwnd),
                    IDC_LINK_RESCAN_APPS => refresh_running_apps(hwnd, true),
                    IDC_BTN_BROWSE_EXE => {
                        if let Some(exe) = browse_exe(hwnd) {
                            SetWindowTextW(GetDlgItem(hwnd, IDC_EDIT_PROFILE_EXE as i32), to_wide(&exe).as_ptr());
                            let h_name = GetDlgItem(hwnd, IDC_EDIT_PROFILE_NAME as i32);
                            if GetWindowTextLengthW(h_name) == 0 {
                                let stem = exe.strip_suffix(".exe").or_else(|| exe.strip_suffix(".EXE")).unwrap_or(&exe);
                                SetWindowTextW(h_name, to_wide(stem).as_ptr());
                            }
                        }
                    }
                    IDC_BTN_CREATE_PROFILE => create_profile(hwnd),
                    _ => {}
                }
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint(hwnd);
                0
            }
            WM_DRAWITEM => {
                let dis = &*(lparam as *const DRAWITEMSTRUCT);
                if dis.hwnd_item != 0 {
                    let res = SendMessageW(dis.hwnd_item, OCM_DRAWITEM, wparam, lparam);
                    if res != 0 {
                        return res;
                    }
                }
                0
            }
            WM_CLOSE => {
                let parent = DIALOG_PARENT_HWND.load(Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                    SetForegroundWindow(parent);
                }
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                if let Ok(mut guard) = APP_TABLE.lock() {
                    *guard = None;
                }
                let parent = DIALOG_PARENT_HWND.swap(0, Ordering::SeqCst);
                if parent != 0 {
                    EnableWindow(parent, 1);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
