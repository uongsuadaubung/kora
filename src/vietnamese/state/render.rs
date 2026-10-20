use super::types::SyllableState;
use std::fmt;

impl fmt::Display for SyllableState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render_compact())
    }
}

impl SyllableState {
    /// Renders current state directly to a pre-allocated buffer with zero formatting overhead
    #[inline]
    pub fn render_to(&self, out: &mut String) {
        out.push_str(&self.render_compact());
    }

    /// Renders current state directly to an inline char list with zero heap allocation
    #[inline]
    pub fn render_chars_to(&self, out: &mut crate::vietnamese::InlineList<char, 16>) {
        match self {
            SyllableState::Empty => {}
            SyllableState::Onset(onset) => onset.render_chars_to(out),
            SyllableState::Nucleus(nucleus) => nucleus.render_chars_to(out),
            SyllableState::Coda(coda) => coda.render_chars_to(out),
            SyllableState::Passthrough(raw) => {
                for &ch in raw.iter() {
                    out.push(ch);
                }
            }
        }
    }

    /// Renders current state to a CompactString on the stack (zero heap allocation)
    #[inline]
    pub fn render_compact(&self) -> crate::engine::action::CompactString {
        match self {
            SyllableState::Empty => crate::engine::action::CompactString::new(),
            SyllableState::Onset(onset) => onset.render_compact(),
            SyllableState::Nucleus(nucleus) => nucleus.render_compact(),
            SyllableState::Coda(coda) => coda.render_compact(),
            SyllableState::Passthrough(raw) => raw.iter().copied().collect(),
        }
    }

    /// Renders current state to a displayed string with pre-allocated capacity
    #[inline]
    pub fn render(&self) -> String {
        self.render_compact().to_string()
    }
}
