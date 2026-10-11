//! WM_COMMAND dispatcher for Kora Profile-First Workspace Hub
//! Delegates command notifications directly to component owners via ControlPanelControls.


use crate::ui::win32::*;

pub fn handle_command(
    hwnd: isize,
    wparam: usize,
    lparam: isize,
    controls: Option<&crate::ui::views::ControlPanelControls>,
) -> isize {
    let control_id = (wparam & 0xFFFF) as u32;
    let notify_code = ((wparam >> 16) & 0xFFFF) as u32;
    let target_hwnd = lparam;

    // Message Reflection: Forward OCM_COMMAND directly to child control
    if target_hwnd != 0 {
        let res = unsafe {
            SendMessageW(
                target_hwnd,
                OCM_COMMAND,
                wparam,
                lparam,
            )
        };
        if res != 0 {
            return 0;
        }
    }

    if let Some(c) = controls {
        c.on_command(hwnd, control_id, notify_code);
    }

    if control_id == crate::ui::tokens::id::IDC_COMBO_TYPING_MODE {
        if let Ok(guard) = crate::platform::win32::ENGINE_INSTANCE.lock()
            && let Some(ref engine) = *guard
        {
            crate::ui::update_tray_icon(engine.effective_config().enabled);
        }
    } else if control_id == crate::ui::tokens::id::IDC_COMBO_THEME
        || control_id == crate::ui::tokens::id::IDC_COMBO_LANG
    {
        if let Ok(mut ui_guard) = crate::ui::UI_MANAGER.lock()
            && let Some(ref mut ui) = *ui_guard
        {
            ui.tray.refresh_icon();
        }
    }

    0
}
