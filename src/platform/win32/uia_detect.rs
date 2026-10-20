//! Windows UI Automation (UIA) COM Wrapper
//! Zero external dependencies — pure Win32 C-ABI COM vtables.
//!
//! Queries the currently focused UI element's `AutomationId` or `ClassName`
//! to enable surgical, field-level key injection customization.

use std::ptr::null_mut;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

// CLSID_CUIAutomation: {ff48dba4-60ef-4201-aa87-54103eef594e}
pub const CLSID_CUI_AUTOMATION: GUID = GUID {
    data1: 0xFF48DBA4,
    data2: 0x60EF,
    data3: 0x4201,
    data4: [0xAA, 0x87, 0x54, 0x10, 0x3E, 0xEF, 0x59, 0x4E],
};

// IID_IUIAutomation: {30cbe57d-d9d0-452a-ab13-7ac5ac4825ee}
pub const IID_IUI_AUTOMATION: GUID = GUID {
    data1: 0x30CBE57D,
    data2: 0xD9D0,
    data3: 0x452A,
    data4: [0xAB, 0x13, 0x7A, 0xC5, 0xAC, 0x48, 0x25, 0xEE],
};

pub const CLSCTX_ALL: u32 = 23;
pub const COINIT_MULTITHREADED: u32 = 0x0;

#[link(name = "ole32")]
unsafe extern "system" {
    pub fn CoInitializeEx(pv_reserved: *mut std::ffi::c_void, dw_co_init: u32) -> i32;
    pub fn CoUninitialize();
    pub fn CoCreateInstance(
        rclsid: *const GUID,
        p_unk_outer: *mut std::ffi::c_void,
        dw_cls_context: u32,
        riid: *const GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> i32;
}

#[link(name = "oleaut32")]
unsafe extern "system" {
    pub fn SysStringLen(bstr: *const u16) -> u32;
    pub fn SysFreeString(bstr: *mut u16);
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IUIAutomationVtbl {
    pub query_interface: usize,
    pub add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub compare_elements: usize,
    pub compare_runtime_ids: usize,
    pub get_root_element: usize,
    pub element_from_handle: usize,
    pub element_from_point: usize,
    pub get_focused_element: unsafe extern "system" fn(
        this: *mut std::ffi::c_void,
        element: *mut *mut std::ffi::c_void,
    ) -> i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IUIAutomationElementVtbl {
    pub query_interface: usize,
    pub add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
    pub set_focus: usize,
    pub get_runtime_id: usize,
    pub find_first: usize,
    pub find_all: usize,
    pub find_first_build_cache: usize,
    pub find_all_build_cache: usize,
    pub build_updated_cache: usize,
    pub get_current_property_value: usize,
    pub get_current_property_value_ex: usize,
    pub get_cached_property_value: usize,
    pub get_cached_property_value_ex: usize,
    pub get_current_pattern_as: usize,
    pub get_cached_pattern_as: usize,
    pub get_current_pattern: usize,
    pub get_cached_pattern: usize,
    pub get_cached_parent: usize,
    pub get_cached_children: usize,
    pub get_current_process_id:
        unsafe extern "system" fn(this: *mut std::ffi::c_void, ret_val: *mut i32) -> i32,
    pub get_current_control_type: usize,
    pub get_current_localized_control_type: usize,
    pub get_current_name:
        unsafe extern "system" fn(this: *mut std::ffi::c_void, ret_val: *mut *mut u16) -> i32,
    pub get_current_accelerator_key: usize,
    pub get_current_access_key: usize,
    pub get_current_has_keyboard_focus: usize,
    pub get_current_is_keyboard_focusable: usize,
    pub get_current_is_enabled: usize,
    pub get_current_automation_id:
        unsafe extern "system" fn(this: *mut std::ffi::c_void, ret_val: *mut *mut u16) -> i32,
    pub get_current_class_name:
        unsafe extern "system" fn(this: *mut std::ffi::c_void, ret_val: *mut *mut u16) -> i32,
}

pub const CLSID_CUI_AUTOMATION8: GUID = GUID {
    data1: 0xE22AD333,
    data2: 0xB25F,
    data3: 0x460C,
    data4: [0x83, 0xD0, 0x05, 0x81, 0x10, 0x73, 0x95, 0xC9],
};

pub const CLSCTX_INPROC_SERVER: u32 = 1;

/// Information about the currently focused UI element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedElementInfo {
    pub process_name: String,
    pub target_id: String,
}

/// Retrieves the UI Automation identifier and associated process name of the currently focused control.
/// Resolution priority for identifier:
/// 1. `AutomationId` (e.g., "urlbar-input", "search-box")
/// 2. `ClassName` (e.g., "MozillaCompositorWindowClass", "Edit")
/// 3. `Name` (e.g., "Search or enter address")
pub fn get_focused_element_info() -> Option<FocusedElementInfo> {
    unsafe {
        let _ = CoInitializeEx(null_mut(), COINIT_MULTITHREADED);

        let mut uia_ptr: *mut std::ffi::c_void = null_mut();
        // Try modern CUIAutomation8 first, then fallback to CUIAutomation
        let mut hr = CoCreateInstance(
            &CLSID_CUI_AUTOMATION8,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IUI_AUTOMATION,
            &mut uia_ptr,
        );

        if hr != 0 || uia_ptr.is_null() {
            hr = CoCreateInstance(
                &CLSID_CUI_AUTOMATION,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IUI_AUTOMATION,
                &mut uia_ptr,
            );
        }

        if hr != 0 || uia_ptr.is_null() {
            CoUninitialize();
            return None;
        }

        let uia_vtbl = **(uia_ptr as *mut *mut IUIAutomationVtbl);

        let mut elem_ptr: *mut std::ffi::c_void = null_mut();
        let hr_elem = (uia_vtbl.get_focused_element)(uia_ptr, &mut elem_ptr);

        let mut result = None;

        if hr_elem == 0 && !elem_ptr.is_null() {
            let elem_vtbl = **(elem_ptr as *mut *mut IUIAutomationElementVtbl);

            // Retrieve process ID of the focused element
            let mut pid = 0i32;
            let hr_pid = (elem_vtbl.get_current_process_id)(elem_ptr, &mut pid);
            let mut proc_name = if hr_pid == 0 && pid > 0 {
                crate::platform::win32::app_detect::get_process_name_for_pid(pid as u32)
            } else {
                String::new()
            };
            if proc_name.is_empty() {
                proc_name = crate::platform::win32::app_detect::get_foreground_process_name();
            }

            let mut resolved_id = None;

            // 1. Try AutomationId first
            let mut bstr: *mut u16 = null_mut();
            let hr_id = (elem_vtbl.get_current_automation_id)(elem_ptr, &mut bstr);
            if hr_id == 0 && !bstr.is_null() {
                let len = SysStringLen(bstr) as usize;
                if len > 0 {
                    let slice = std::slice::from_raw_parts(bstr, len);
                    let id_str = String::from_utf16_lossy(slice).trim().to_string();
                    if !id_str.is_empty() {
                        resolved_id = Some(id_str);
                    }
                }
                SysFreeString(bstr);
            }

            // 2. Fallback to ClassName if AutomationId is empty
            if resolved_id.is_none() {
                let mut bstr_class: *mut u16 = null_mut();
                let hr_class = (elem_vtbl.get_current_class_name)(elem_ptr, &mut bstr_class);
                if hr_class == 0 && !bstr_class.is_null() {
                    let len = SysStringLen(bstr_class) as usize;
                    if len > 0 {
                        let slice = std::slice::from_raw_parts(bstr_class, len);
                        let class_str = String::from_utf16_lossy(slice).trim().to_string();
                        if !class_str.is_empty() {
                            resolved_id = Some(class_str);
                        }
                    }
                    SysFreeString(bstr_class);
                }
            }

            // 3. Fallback to Name if both AutomationId and ClassName are empty
            if resolved_id.is_none() {
                let mut bstr_name: *mut u16 = null_mut();
                let hr_name = (elem_vtbl.get_current_name)(elem_ptr, &mut bstr_name);
                if hr_name == 0 && !bstr_name.is_null() {
                    let len = SysStringLen(bstr_name) as usize;
                    if len > 0 {
                        let slice = std::slice::from_raw_parts(bstr_name, len);
                        let name_str = String::from_utf16_lossy(slice).trim().to_string();
                        if !name_str.is_empty() {
                            resolved_id = Some(name_str);
                        }
                    }
                    SysFreeString(bstr_name);
                }
            }

            if let Some(target_id) = resolved_id {
                result = Some(FocusedElementInfo {
                    process_name: proc_name,
                    target_id,
                });
            }

            (elem_vtbl.release)(elem_ptr);
        }

        (uia_vtbl.release)(uia_ptr);
        CoUninitialize();

        result
    }
}

/// Retrieves the UI Automation identifier of the currently focused control.
pub fn get_focused_element_id() -> Option<String> {
    get_focused_element_info().map(|info| info.target_id)
}
