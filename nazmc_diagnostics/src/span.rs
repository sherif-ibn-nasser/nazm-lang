use std::usize;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Span {
    pub start: SpanCursor,
    pub end: SpanCursor,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SpanCursor {
    /// The line index
    pub line: usize,
    /// The column index
    pub col: usize,
}

impl Span {
    #[inline]
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        Self {
            start: SpanCursor {
                line: start.0,
                col: start.1,
            },
            end: SpanCursor {
                line: end.0,
                col: end.1,
            },
        }
    }

    #[inline]
    pub fn merged_with(&self, with: &Span) -> Self {
        Self {
            start: self.start,
            end: with.end,
        }
    }

    /// Returns a zero-column span located after given the span
    #[inline]
    pub fn after(given: &Span) -> Self {
        Self {
            start: given.end,
            end: given.end,
        }
    }
}
