use super::types::SyllableState;
use std::fmt;

impl fmt::Display for SyllableState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::with_capacity(16);
        self.render_to(&mut s);
        f.write_str(&s)
    }
}

impl SyllableState {
    /// Renders current state directly to a pre-allocated buffer with zero formatting overhead
    #[inline]
    pub fn render_to(&self, out: &mut String) {
        match self {
            SyllableState::Empty => {}
            SyllableState::Onset(onset) => onset.render_to(out),
            SyllableState::Nucleus(nucleus) => nucleus.render_to(out),
            SyllableState::Coda(coda) => coda.render_to(out),
            SyllableState::Passthrough(raw) => {
                for &ch in raw.iter() {
                    out.push(ch);
                }
            }
        }
    }

    /// Renders current state to a displayed string with pre-allocated capacity
    #[inline]
    pub fn render(&self) -> String {
        let mut s = String::with_capacity(16);
        self.render_to(&mut s);
        s
    }
}

