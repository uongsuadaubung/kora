//! Keystroke Injection & Native SendInput Synthesizer

use super::app_detect::{AutocompleteFixType, detect_autocomplete_context};
use super::types::{
    INPUT, INPUT_KEYBOARD, INPUT_UNION, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    MAGIC_EXTRA_INFO, SendInput, VK_BACK,
};

#[inline]
pub fn make_backspace_down() -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                w_vk: VK_BACK as u16,
                w_scan: 0x0E,
                dw_flags: 0,
                time: 0,
                dw_extra_info: MAGIC_EXTRA_INFO,
            },
        },
    }
}

#[inline]
pub fn make_backspace_up() -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                w_vk: VK_BACK as u16,
                w_scan: 0x0E,
                dw_flags: KEYEVENTF_KEYUP,
                time: 0,
                dw_extra_info: MAGIC_EXTRA_INFO,
            },
        },
    }
}

#[inline]
pub fn make_unicode_down(code_unit: u16) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                w_vk: 0,
                w_scan: code_unit,
                dw_flags: KEYEVENTF_UNICODE,
                time: 0,
                dw_extra_info: MAGIC_EXTRA_INFO,
            },
        },
    }
}

#[inline]
pub fn make_unicode_up(code_unit: u16) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                w_vk: 0,
                w_scan: code_unit,
                dw_flags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                time: 0,
                dw_extra_info: MAGIC_EXTRA_INFO,
            },
        },
    }
}

#[inline]
pub fn push_backspace(inputs: &mut Vec<INPUT>) {
    inputs.push(make_backspace_down());
    inputs.push(make_backspace_up());
}

#[inline]
pub fn push_unicode_char(inputs: &mut Vec<INPUT>, code_unit: u16) {
    inputs.push(make_unicode_down(code_unit));
    inputs.push(make_unicode_up(code_unit));
}

const STACK_INPUT_LIMIT: usize = 64;

/// Sends backspaces and replacement string in a SINGLE atomic SendInput batch.
/// - In normal apps & web pages: sends pure standard Backspaces and replacement characters (no invisible chars).
/// - In browser Omnibox / Excel: collapses inline autocomplete selection cleanly without polluting document text.
pub fn send_replace(backspaces: usize, text: &str) {
    let utf16_count = text.encode_utf16().count();
    if backspaces == 0 && utf16_count == 0 {
        return;
    }

    let fix_type = detect_autocomplete_context();
    let is_autocomplete = matches!(
        fix_type,
        AutocompleteFixType::ChromiumOmnibox | AutocompleteFixType::GenericAutocomplete
    );

    let extra = if is_autocomplete && backspaces > 0 && utf16_count > 0 {
        4
    } else {
        0
    };

    let total_needed = extra + backspaces * 2 + utf16_count * 2;

    if total_needed <= STACK_INPUT_LIMIT {
        // Fast path: Zero heap allocations on the Windows low-level hook thread.
        // Use MaybeUninit to completely eliminate zeroing 2.5 KB on the stack every keystroke.
        let mut inputs: [std::mem::MaybeUninit<INPUT>; STACK_INPUT_LIMIT] =
            [std::mem::MaybeUninit::uninit(); STACK_INPUT_LIMIT];
        let mut count = 0;

        if is_autocomplete && backspaces > 0 && utf16_count > 0 {
            inputs[count].write(make_unicode_down(0x202F));
            count += 1;
            inputs[count].write(make_unicode_up(0x202F));
            count += 1;
            inputs[count].write(make_backspace_down());
            count += 1;
            inputs[count].write(make_backspace_up());
            count += 1;
        }

        for _ in 0..backspaces {
            inputs[count].write(make_backspace_down());
            count += 1;
            inputs[count].write(make_backspace_up());
            count += 1;
        }

        for ch in text.encode_utf16() {
            inputs[count].write(make_unicode_down(ch));
            count += 1;
            inputs[count].write(make_unicode_up(ch));
            count += 1;
        }

        unsafe {
            SendInput(
                count as u32,
                inputs.as_ptr() as *const INPUT,
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    } else {
        // Fallback for massive macro expansion strings (>64 inputs)
        let mut inputs = Vec::with_capacity(total_needed);

        if is_autocomplete && backspaces > 0 && utf16_count > 0 {
            push_unicode_char(&mut inputs, 0x202F);
            push_backspace(&mut inputs);
        }
        for _ in 0..backspaces {
            push_backspace(&mut inputs);
        }
        for ch in text.encode_utf16() {
            push_unicode_char(&mut inputs, ch);
        }

        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }
}

/// Sends simulated Backspace keystrokes to active window
pub fn send_backspaces(count: usize) {
    send_replace(count, "");
}

/// Sends simulated UTF-16 Unicode characters to active window
pub fn send_unicode_string(text: &str) {
    send_replace(0, text);
}

/// Sends a simulated hardware virtual-key event (Key Down or Key Up) with MAGIC_EXTRA_INFO
pub fn send_vk_event(vk: u32, is_up: bool) {
    let mut flags = 0;
    if is_up {
        flags |= KEYEVENTF_KEYUP;
    }
    let is_extended = matches!(
        vk,
        0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x26 | 0x27 | 0x28 | 0x2D | 0x2E | 0x5B | 0x5C | 0x5D | 0x90 | 0xAD | 0xAE | 0xAF | 0xB0 | 0xB1 | 0xB2 | 0xB3
    );
    if is_extended {
        flags |= super::types::KEYEVENTF_EXTENDEDKEY;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        u: INPUT_UNION {
            ki: KEYBDINPUT {
                w_vk: vk as u16,
                w_scan: 0,
                dw_flags: flags,
                time: 0,
                dw_extra_info: MAGIC_EXTRA_INFO,
            },
        },
    };
    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

