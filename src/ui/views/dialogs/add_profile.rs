//! Modal Dialog for Adding a New Application Profile
//! Allows user to pick a running app or specify an executable name.
//! Layout follows mockup_profiles.html: custom header, hint box, icon + name,
//! .exe field with file browser, searchable running-app list and footer bar.

use super::{attach_state, close_modal, detach_state, open_modal, reflect_dialog_messages, state, ModalSpec};
use crate::engine::profile::AppProfile;
use crate::platform::win32::app_detect::get_process_name_for_pid;
use crate::platform::win32::types::GetWindowThreadProcessId;
use crate::ui::colors::ThemePalette;
use crate::ui::components::*;
use crate::ui::theme::{get_app_font_bold, get_app_font_normal, is_current_dark};
use crate::ui::win32::*;
use std::sync::{Arc, Mutex, OnceLock};

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

#[derive(Clone)]
struct AppWindowEntry {
    title: String,
    exe_name: String,
}

struct AddProfileState {
    parent: isize,
    selected_icon: usize,
    running_apps: Arc<Mutex<Vec<AppWindowEntry>>>,
    visible_apps: Vec<AppWindowEntry>,
    btn_icon: Option<PushButton>,
    txt_name: Option<TextBox>,
    txt_exe: Option<TextBox>,
    lnk_rescan: Option<Label<Link>>,
    txt_search: Option<TextBox>,
    table: Option<ActionTable>,
}

static REGISTERED: std::sync::Once = std::sync::Once::new();
const CLASS_NAME: &str = "KoraAddProfileDialogClass";
const WM_APP_ENUM_DONE: u32 = 0x8000 + 104;

pub fn show_add_profile_dialog(parent: isize) {
    register_class_once(
        &REGISTERED,
        CLASS_NAME,
        add_profile_dialog_wnd_proc,
        0x0002_0000,
    );

    let strings = crate::language::current();
    let st = Box::new(AddProfileState {
        parent,
        selected_icon: DEFAULT_ICON,
        running_apps: Arc::new(Mutex::new(Vec::new())),
        visible_apps: Vec::new(),
        btn_icon: None,
        txt_name: None,
        txt_exe: None,
        lnk_rescan: None,
        txt_search: None,
        table: None,
    });

    // Borderless popup: header/close button are drawn by the dialog itself
    let hwnd = open_modal(
        parent,
        &ModalSpec {
            class_name: CLASS_NAME,
            title: strings.dlg_add_prof_title,
            width: W,
            height: H,
            ex_style: WS_EX_CONTROLPARENT,
            style: 0x8000_0000 /* WS_POPUP */ | WS_CLIPCHILDREN,
        },
        st,
    );

    if hwnd != 0 {
        unsafe {
            let disable: i32 = 1; // DWMWA_TRANSITIONS_FORCEDISABLED: no fade on show/hide
            DwmSetWindowAttribute(hwnd, 3, &disable as *const i32 as *const _, 4);
            let name_hwnd = state::<AddProfileState>(hwnd)
                .and_then(|s| s.txt_name.as_ref().map(|t| t.hwnd()))
                .unwrap_or(0);
            if name_hwnd != 0 {
                SetFocus(name_hwnd);
            }
        }
    }
}

fn scan_running_apps_async(hwnd: isize, running_apps: Arc<Mutex<Vec<AppWindowEntry>>>) {
    std::thread::spawn(move || {
        let mut list: Vec<AppWindowEntry> = Vec::new();
        unsafe extern "system" fn worker_enum_proc(wnd: isize, lparam: isize) -> i32 {
            unsafe {
                if IsWindowVisible(wnd) == 0 {
                    return 1;
                }
                let title = window_text(wnd);
                if title.is_empty() || title.starts_with("Kora") {
                    return 1;
                }
                let mut pid = 0u32;
                GetWindowThreadProcessId(wnd, &mut pid);
                let exe = get_process_name_for_pid(pid);
                if !exe.is_empty() {
                    let acc = &mut *(lparam as *mut Vec<AppWindowEntry>);
                    if !acc.iter().any(|a| a.exe_name == exe) {
                        acc.push(AppWindowEntry {
                            title,
                            exe_name: exe,
                        });
                    }
                }
                1
            }
        }

        unsafe {
            EnumWindows(
                Some(worker_enum_proc),
                &mut list as *mut Vec<AppWindowEntry> as isize,
            );
        }
        if let Ok(mut g) = running_apps.lock() {
            *g = list;
        }
        unsafe {
            PostMessageW(hwnd, WM_APP_ENUM_DONE, 0, 0);
        }
    });
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
unsafe fn refresh_running_apps(hwnd: isize, rescan: bool) {
    let Some(st) = (unsafe { state::<AddProfileState>(hwnd) }) else {
        return;
    };
    if rescan {
        scan_running_apps_async(hwnd, st.running_apps.clone());
        return;
    }
    let query = st
        .txt_search
        .as_ref()
        .map(|t| t.get_text().trim().to_lowercase())
        .unwrap_or_default();
    let visible: Vec<AppWindowEntry> = st
        .running_apps
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

    if let Some(ref tbl) = st.table {
        tbl.clear();
        tbl.set_rows(visible.iter().map(app_row));
    }
    st.visible_apps = visible;
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
        draw_text(hdc, strings.dlg_add_prof_title,
            &RECT { left: PAD + 34, top: 0, right: W - 60, bottom: HEADER_H },
            crate::ui::theme::cached_font(11, FW_BOLD), palette.text_primary,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE);

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
        if let Some(btn) = PushButton::create_with_variant(
            hwnd, 0, "✕", W - PAD - 32, 10, 32, 32,
            font_normal, ButtonVariant::Ghost, Some(palette.bg_sidebar),
        ) {
            btn.on_click(move || {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            });
        }

        // Icon + display name
        Label::create(hwnd, strings.dlg_add_prof_icon, PAD, NAME_LBL_Y, 56, 18, font_label)
            .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
        Label::create(hwnd, strings.dlg_add_prof_name, PAD + 68, NAME_LBL_Y, 300, 18, font_label)
            .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
        let btn_icon = PushButton::create_with_variant(
            hwnd, 0, ICONS[DEFAULT_ICON], PAD, NAME_Y, 56, FIELD_H,
            font_normal, ButtonVariant::Secondary, Some(palette.bg_window),
        );
        if let Some(ref btn) = btn_icon {
            btn.set_on_click(move || pick_icon(hwnd));
        }
        let txt_name = TextBox::create_with_placeholder(
            hwnd, 0, strings.dlg_add_prof_name_placeholder,
            PAD + 68, NAME_Y, cw - 68, FIELD_H, font_normal,
        );

        // Executable + file browser
        Label::create(hwnd, strings.dlg_add_prof_exe, PAD, EXE_LBL_Y, 300, 18, font_label)
            .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
        if let Some(btn) = PushButton::create_with_variant(
            hwnd, 0, strings.dlg_add_prof_browse, W - PAD - 150, EXE_LBL_Y - 6, 150, 28,
            font_normal, ButtonVariant::Secondary, Some(palette.bg_window),
        ) {
            btn.on_click(move || {
                if let Some(exe) = browse_exe(hwnd)
                    && let Some(st) = state::<AddProfileState>(hwnd)
                {
                    if let Some(ref txt_exe) = st.txt_exe {
                        txt_exe.set_text(&exe);
                    }
                    if let Some(ref txt_name) = st.txt_name
                        && txt_name.get_text().is_empty()
                    {
                        let stem = exe
                            .strip_suffix(".exe")
                            .or_else(|| exe.strip_suffix(".EXE"))
                            .unwrap_or(&exe);
                        txt_name.set_text(stem);
                    }
                }
            });
        }
        let txt_exe = TextBox::create_with_placeholder(
            hwnd, 0, strings.dlg_add_prof_exe_placeholder,
            PAD, EXE_Y, cw, FIELD_H, font_normal,
        );

        // Running apps: label, rescan link, search box, list
        Label::create(hwnd, strings.dlg_add_prof_linked_app, PAD, APPS_LBL_Y, 340, 18, font_label)
            .map(|l| l.with_tone(LabelTone::Secondary).on_surface(SurfaceRole::Window));
        let lnk_rescan = Label::create_right(
            hwnd, strings.dlg_add_prof_rescan, W - PAD - 120, APPS_LBL_Y, 120, 18,
            font_normal,
        )
        .map(|l| l.as_link().on_surface(SurfaceRole::Window));
        let txt_search = TextBox::create_with_placeholder(
            hwnd, 0, strings.dlg_add_prof_search_placeholder,
            PAD, SEARCH_Y, cw, FIELD_H, font_normal,
        );

        let table = ActionTable::create_with_defs(
            hwnd, 0, PAD, LIST_Y, cw, LIST_H, font_normal,
            vec![ColumnDef::flex("", ColumnStyle::Default)],
        );
        if let Some(ref tbl) = table {
            tbl.hide_header();
            tbl.set_on_select(move |sel| {
                let Some(st) = state::<AddProfileState>(hwnd) else {
                    return;
                };
                if let Some(app) = st.visible_apps.get(sel as usize).cloned() {
                    if let Some(ref txt_name) = st.txt_name
                        && txt_name.get_text().is_empty()
                    {
                        let clean_title = app
                            .title
                            .split(['-', '|', '—'])
                            .next()
                            .unwrap_or(&app.title)
                            .trim();
                        txt_name.set_text(clean_title);
                    }
                    if let Some(ref txt_exe) = st.txt_exe {
                        txt_exe.set_text(&app.exe_name);
                    }
                }
            });
        }

        // Footer buttons
        let btn_y = FOOTER_Y + (FOOTER_H - 32) / 2;
        super::create_dialog_footer_buttons(
            hwnd,
            (strings.dlg_btn_cancel, W - PAD - 110 - 8 - 90, 90),
            (
                strings.dlg_add_prof_btn_create,
                W - PAD - 110,
                110,
                ButtonVariant::Primary,
            ),
            btn_y,
            32,
            Some(palette.bg_sidebar),
            move || create_profile(hwnd),
        );

        if let Some(st) = state::<AddProfileState>(hwnd) {
            st.btn_icon = btn_icon;
            st.txt_name = txt_name;
            st.txt_exe = txt_exe;
            st.lnk_rescan = lnk_rescan;
            st.txt_search = txt_search;
            st.table = table;
        }
        refresh_running_apps(hwnd, true);
    }
}

unsafe fn pick_icon(hwnd: isize) {
    unsafe {
        let (current, btn_hwnd) = state::<AddProfileState>(hwnd)
            .map(|s| (s.selected_icon, s.btn_icon.as_ref().map_or(0, |b| b.hwnd())))
            .unwrap_or((DEFAULT_ICON, 0));
        if btn_hwnd == 0 {
            return;
        }
        let Some(mut menu) = PopupMenu::new() else { return };
        for (i, label) in crate::language::current().profile_icons().iter().enumerate() {
            menu.add_checked_item(i as u32 + 1, label, i == current);
        }
        let mut rc = RECT::default();
        GetWindowRect(btn_hwnd, &mut rc);
        let cmd = menu.track(hwnd, rc.left, rc.bottom + 2) as usize;
        if (1..=ICONS.len()).contains(&cmd)
            && let Some(st) = state::<AddProfileState>(hwnd)
        {
            st.selected_icon = cmd - 1;
            if let Some(ref btn) = st.btn_icon {
                btn.set_text(ICONS[cmd - 1]);
            }
        }
    }
}

unsafe fn create_profile(hwnd: isize) {
    unsafe {
        let Some(st) = state::<AddProfileState>(hwnd) else {
            return;
        };
        let name = st
            .txt_name
            .as_ref()
            .map(|t| t.get_text().trim().to_string())
            .unwrap_or_default();
        let exe = st
            .txt_exe
            .as_ref()
            .map(|t| t.get_text().trim().to_lowercase())
            .unwrap_or_default();
        let selected_icon = st.selected_icon;
        let parent = st.parent;
        let name_hwnd = st.txt_name.as_ref().map_or(0, |t| t.hwnd());

        if !name.is_empty() {
            let icon = ICONS[selected_icon.min(ICONS.len() - 1)];

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
            EnableWindow(parent, 1);
            ShowWindow(hwnd, SW_HIDE);
            if parent != 0
                && let Some(c) = crate::ui::controls().or_else(|| {
                    let ptr = GetWindowLongPtrW(parent, GWLP_USERDATA) as *const crate::ui::views::ControlPanelControls;
                    if !ptr.is_null() {
                        Some(&*ptr)
                    } else {
                        None
                    }
                })
            {
                crate::ui::views::control_panel::SidebarControls::switch_to_profile(c, parent, &id);
            }
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        } else if name_hwnd != 0 {
            SetFocus(name_hwnd);
        }
    }
}

/// # Safety
///
/// Win32 window procedure callback called by Windows OS message dispatch.
pub unsafe extern "system" fn add_profile_dialog_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    const HTCLIENT: isize = 1;
    const HTCAPTION: isize = 2;
    const WM_NCDESTROY: u32 = 0x0082;

    unsafe {
        if let Some(res) = reflect_dialog_messages(hwnd, msg, wparam, lparam) {
            return res;
        }
        match msg {
            WM_CREATE => {
                attach_state::<AddProfileState>(hwnd, lparam);
                create_controls(hwnd);
                0
            }
            WM_APP_ENUM_DONE => {
                refresh_running_apps(hwnd, false);
                0
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let vk = wparam as u32;
                if vk == 0x1B {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                    return 0;
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
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
            WM_COMMAND => {
                let ctl_id = (wparam & 0xFFFF) as u32;
                let notify_code = ((wparam >> 16) & 0xFFFF) as u32;
                let Some(st) = state::<AddProfileState>(hwnd) else {
                    return 0;
                };

                if st
                    .txt_search
                    .as_ref()
                    .is_some_and(|t| t.is_changed(ctl_id, notify_code))
                {
                    refresh_running_apps(hwnd, false);
                } else if st
                    .lnk_rescan
                    .as_ref()
                    .is_some_and(|l| l.is_clicked(ctl_id, notify_code))
                {
                    refresh_running_apps(hwnd, true);
                } else if ctl_id == IDCANCEL {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                } else if ctl_id == IDOK {
                    create_profile(hwnd);
                }
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint(hwnd);
                0
            }
            WM_CLOSE => {
                let parent = state::<AddProfileState>(hwnd).map_or(0, |s| s.parent);
                close_modal(hwnd, parent);
                0
            }
            WM_NCDESTROY => {
                if let Some(st) = detach_state::<AddProfileState>(hwnd) {
                    EnableWindow(st.parent, 1);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
