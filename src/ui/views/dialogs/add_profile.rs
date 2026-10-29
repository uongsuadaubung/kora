//! Declarative Modal Dialog for Adding a New Application Profile
//!
//! Uses [`crate::ui_framework::core::modal::{open_modal_with_wndproc, modal_dialog, modal_footer}`]
//! and declarative widgets [`input`, `select`, `button`, `text`] to render a clean, modern
//! 3-section modal dialog without manual Win32 child HWND coordinates or GDI subclassing.

use std::sync::{Arc, Mutex};

use super::base::dialog_callout;
use crate::engine::profile::AppProfile;
use crate::platform::win32::app_detect::get_process_name_for_pid;
use crate::platform::win32::types::GetWindowThreadProcessId;
use crate::ui_framework::core::*;
use crate::ui_framework::win32::*;

/// Emoji per entry of `Strings::profile_icons()` (same order)
const ICONS: [&str; 6] = ["🎮", "💻", "📝", "🎨", "🌐", "⚙️"];
const DEFAULT_ICON: usize = 5;

const WM_APP_ENUM_DONE: u32 = 0x8000 + 104;

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

#[derive(Clone, Debug)]
pub struct AppWindowEntry {
    pub title: String,
    pub exe_name: String,
}

#[derive(Clone, Debug)]
pub struct AddProfileState {
    pub selected_icon: usize,
    pub name: String,
    pub exe: String,
    pub search_query: String,
    pub running_apps: Arc<Mutex<Vec<AppWindowEntry>>>,
    pub visible_apps: Vec<AppWindowEntry>,
    pub is_scanning: bool,
    pub scroll_offset: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AddProfileMsg {
    IconChanged(usize),
    NameChanged(String),
    ExeChanged(String),
    BrowseExe,
    SearchChanged(String),
    Scroll(i32),
    Rescan,
    AppsScanned,
    SelectApp(String, String),
    Submit,
    Cancel,
}

fn update_visible_apps(st: &mut AddProfileState) {
    let q = st.search_query.trim().to_lowercase();
    if let Ok(apps) = st.running_apps.lock() {
        st.visible_apps = if q.is_empty() {
            apps.clone()
        } else {
            apps.iter()
                .filter(|a| {
                    a.title.to_lowercase().contains(&q)
                        || a.exe_name.to_lowercase().contains(&q)
                })
                .cloned()
                .collect()
        };
    }
    st.scroll_offset = 0;
}

pub fn render_add_profile_view(state: &AddProfileState) -> Element<AddProfileMsg> {
    let strings = crate::language::current();
    let can_submit = !state.name.trim().is_empty();

    // Strings in profile_icons() already start with the emoji icon (e.g. "🎮 Trò chơi")
    let icon_options: Vec<String> = strings
        .profile_icons()
        .iter()
        .map(|&s| s.to_string())
        .collect();

    let total_apps = state.visible_apps.len();

    let mut app_tbl = table()
        .columns([
            col_def("Tiến trình").flex_1(),
            col_def("Tên file").auto().min_w(110).max_w(150),
        ])
        .hide_header(true)
        .scroll_offset(state.scroll_offset)
        .on_scroll(AddProfileMsg::Scroll)
        .bordered(true)
        .column_dividers(true)
        .empty_text(if state.is_scanning {
            "⏳ Đang quét các ứng dụng đang chạy..."
        } else {
            "Không tìm thấy ứng dụng phù hợp"
        });

    for app in &state.visible_apps {
        let is_selected = !state.exe.is_empty()
            && app.exe_name.eq_ignore_ascii_case(&state.exe);
        app_tbl = app_tbl.row(
            table_row([
                table_cell(
                    text(&app.title)
                        .small()
                        .tone(if is_selected {
                            TextTone::Accent
                        } else {
                            TextTone::Primary
                        })
                        .nowrap(),
                ),
                table_cell(
                    text(&app.exe_name)
                        .small()
                        .secondary()
                        .nowrap(),
                ),
            ])
            .selected(is_selected)
            .on_click(AddProfileMsg::SelectApp(
                app.title.clone(),
                app.exe_name.clone(),
            )),
        );
    }

    modal_dialog()
        .header(dialog_callout(strings.dlg_add_prof_hint))
        .body(
            col(elements![
                // Row 1: Icon & Name
                row(elements![
                    col(elements![
                        text(strings.dlg_add_prof_icon).bold().secondary().small(),
                        select(
                            icon_options,
                            state.selected_icon,
                            AddProfileMsg::IconChanged,
                        )
                        .key(201)
                        .h(32),
                    ])
                    .w(140)
                    .gap(1),
                    col(elements![
                        text(strings.dlg_add_prof_name).bold().secondary().small(),
                        input(
                            &state.name,
                            strings.dlg_add_prof_name_placeholder,
                            AddProfileMsg::NameChanged,
                        )
                        .key(202)
                        .auto_focus(true),
                    ])
                    .flex_1()
                    .gap(1),
                ])
                .gap(2),

                // Row 2: Exe & Browse button (dynamic intrinsic width prevents text clipping)
                col(elements![
                    text(strings.dlg_add_prof_exe).bold().secondary().small(),
                    row(elements![
                        input(
                            &state.exe,
                            strings.dlg_add_prof_exe_placeholder,
                            AddProfileMsg::ExeChanged,
                        )
                        .key(203)
                        .flex_1(),
                        button(
                            strings.dlg_add_prof_browse,
                            Variant::Secondary,
                            AddProfileMsg::BrowseExe,
                        ),
                    ])
                    .gap(2),
                ])
                .gap(1),

                // Row 3: Running apps header with search & rescan
                col(elements![
                    row(elements![
                        text(if total_apps > 0 {
                            format!("{} ({} tiến trình)", strings.dlg_add_prof_linked_app, total_apps)
                        } else {
                            strings.dlg_add_prof_linked_app.to_string()
                        })
                        .bold()
                        .secondary()
                        .small(),
                        spacer(),
                        if state.is_scanning {
                            text("⏳ Đang quét...").secondary().small().into_element()
                        } else {
                            button(
                                strings.dlg_add_prof_rescan,
                                Variant::Ghost,
                                AddProfileMsg::Rescan,
                            )
                            .sm()
                            .into_element()
                        },
                    ])
                    .items_center(),
                    input(
                        &state.search_query,
                        strings.dlg_add_prof_search_placeholder,
                        AddProfileMsg::SearchChanged,
                    )
                    .key(204),
                    app_tbl,
                ])
                .gap(1),
            ])
            .gap(3)
            .h_flex(1),
        )
        .footer(
            row(elements![
                text(strings.dlg_add_prof_footer).secondary().small().flex_1(),
                button(strings.dlg_btn_cancel, Variant::Secondary, AddProfileMsg::Cancel).w(84),
                button(
                    strings.dlg_add_prof_btn_create,
                    Variant::Primary,
                    AddProfileMsg::Submit,
                )
                .disabled(!can_submit)
                .w(110),
            ])
            .items_center()
            .gap(2),
        )
        .build()
}

/// Displays the declarative modal dialog for adding a new application profile.
pub fn show_add_profile_dialog(parent: isize) {
    let strings = crate::language::current();
    let title = strings.dlg_add_prof_title;

    let state = Arc::new(Mutex::new(AddProfileState {
        selected_icon: DEFAULT_ICON,
        name: String::new(),
        exe: String::new(),
        search_query: String::new(),
        running_apps: Arc::new(Mutex::new(Vec::new())),
        visible_apps: Vec::new(),
        is_scanning: true,
        scroll_offset: 0,
    }));

    let state_for_render = Arc::clone(&state);
    let state_for_msg = Arc::clone(&state);

    let initial_view = render_add_profile_view(&state_for_render.lock().unwrap());
    let config = ModalConfig::new(title, 520, 560);

    let handle = open_modal_with_wndproc(
        parent,
        config,
        initial_view,
        move |handle, msg| {
            let mut st = state_for_msg.lock().unwrap();
            match msg {
                AddProfileMsg::IconChanged(idx) => {
                    st.selected_icon = idx;
                    let view = render_add_profile_view(&st);
                    drop(st);
                    handle.render(view);
                }
                AddProfileMsg::NameChanged(name) => {
                    st.name = name;
                    let view = render_add_profile_view(&st);
                    drop(st);
                    handle.render(view);
                }
                AddProfileMsg::ExeChanged(exe) => {
                    st.exe = exe;
                    let view = render_add_profile_view(&st);
                    drop(st);
                    handle.render(view);
                }
                AddProfileMsg::BrowseExe => {
                    let hwnd = handle.hwnd;
                    drop(st);
                    if let Some(exe) = browse_exe(hwnd) {
                        let mut st = state_for_msg.lock().unwrap();
                        st.exe = exe.clone();
                        if st.name.trim().is_empty() {
                            let stem = exe
                                .strip_suffix(".exe")
                                .or_else(|| exe.strip_suffix(".EXE"))
                                .unwrap_or(&exe);
                            st.name = stem.to_string();
                        }
                        let view = render_add_profile_view(&st);
                        drop(st);
                        handle.render(view);
                    }
                }
                AddProfileMsg::SearchChanged(query) => {
                    st.search_query = query;
                    update_visible_apps(&mut st);
                    let view = render_add_profile_view(&st);
                    drop(st);
                    handle.render(view);
                }
                AddProfileMsg::Scroll(delta) => {
                    let page_size = 6;
                    let max_offset = st.visible_apps.len().saturating_sub(page_size);
                    if max_offset > 0 {
                        let new_offset = if delta < 0 {
                            st.scroll_offset.saturating_sub((-delta) as usize)
                        } else {
                            (st.scroll_offset + delta as usize).min(max_offset)
                        };
                        if new_offset != st.scroll_offset {
                            st.scroll_offset = new_offset;
                            let view = render_add_profile_view(&st);
                            drop(st);
                            handle.render(view);
                        }
                    }
                }
                AddProfileMsg::Rescan => {
                    st.is_scanning = true;
                    let running_apps = Arc::clone(&st.running_apps);
                    let hwnd = handle.hwnd;
                    let view = render_add_profile_view(&st);
                    drop(st);
                    handle.render(view);
                    scan_running_apps_async(hwnd, running_apps);
                }
                AddProfileMsg::AppsScanned => {
                    st.is_scanning = false;
                    update_visible_apps(&mut st);
                    let view = render_add_profile_view(&st);
                    drop(st);
                    handle.render(view);
                }
                AddProfileMsg::SelectApp(title, exe_name) => {
                    st.exe = exe_name;
                    if st.name.trim().is_empty() {
                        let clean_title = title
                            .split(['-', '|', '—'])
                            .next()
                            .unwrap_or(&title)
                            .trim();
                        st.name = clean_title.to_string();
                    }
                    let view = render_add_profile_view(&st);
                    drop(st);
                    handle.render(view);
                }
                AddProfileMsg::Submit => {
                    let name = st.name.trim().to_string();
                    let exe = st.exe.trim().to_lowercase();
                    let selected_icon = st.selected_icon;

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

                        if parent != 0
                            && let Some(c) = crate::ui::controls().or_else(|| {
                                let ptr = unsafe { GetWindowLongPtrW(parent, GWLP_USERDATA) }
                                    as *const crate::ui::views::ControlPanelControls;
                                if !ptr.is_null() {
                                    unsafe { Some(&*ptr) }
                                } else {
                                    None
                                }
                            })
                        {
                            crate::ui::views::control_panel::SidebarControls::switch_to_profile(
                                c, parent, &id,
                            );
                        }
                        drop(st);
                        handle.close();
                    }
                }
                AddProfileMsg::Cancel => {
                    drop(st);
                    handle.close();
                }
            }
        },
        move |handle, msg, wparam, _lparam| match msg {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if wparam == 0x0D
                /* VK_RETURN */
                {
                    handle.dispatch(AddProfileMsg::Submit);
                    Some(0)
                } else {
                    None
                }
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam >> 16) as i16) as i32;
                if delta > 0 {
                    handle.dispatch(AddProfileMsg::Scroll(-2));
                } else if delta < 0 {
                    handle.dispatch(AddProfileMsg::Scroll(2));
                }
                Some(0)
            }
            WM_APP_ENUM_DONE => {
                handle.dispatch(AddProfileMsg::AppsScanned);
                Some(0)
            }
            _ => None,
        },
    );

    if let Some(h) = handle {
        let running_apps = Arc::clone(&state.lock().unwrap().running_apps);
        scan_running_apps_async(h.hwnd, running_apps);
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
                    if !acc.iter().any(|a| a.exe_name.eq_ignore_ascii_case(&exe)) {
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
