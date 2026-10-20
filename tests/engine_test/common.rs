use kora::{EngineAction, VietnameseEngine};

/// Helper to simulate typing a string and collecting the rendered output on screen
pub fn simulate_typing(engine: &mut VietnameseEngine, text: &str) -> String {
    let mut screen = String::new();
    for ch in text.chars() {
        let is_upper = ch.is_uppercase();
        match engine.on_key(ch, is_upper, false) {
            EngineAction::Passthrough => {
                screen.push(ch);
            }
            EngineAction::Replace { backspaces, output } => {
                for _ in 0..backspaces {
                    screen.pop();
                }
                screen.push_str(&output);
            }
            EngineAction::Consume => {}
        }
    }
    screen
}

/// Helper to simulate sending a single keystroke to engine and updating screen
pub fn type_key(eng: &mut VietnameseEngine, ch: char, scr: &mut String) {
    match eng.on_key(ch, ch.is_uppercase(), false) {
        EngineAction::Passthrough => scr.push(ch),
        EngineAction::Replace { backspaces, output } => {
            for _ in 0..backspaces {
                scr.pop();
            }
            scr.push_str(&output);
        }
        EngineAction::Consume => {}
    }
}

/// Helper to simulate sending a backspace event to engine and updating screen
pub fn backspace(eng: &mut VietnameseEngine, scr: &mut String) {
    scr.pop();
    eng.on_backspace();
}
