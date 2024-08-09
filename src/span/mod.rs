#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Span{
    pub start: SpanCursor,
    pub end: SpanCursor,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SpanCursor{
    /// The line index
    pub line: usize,
    /// The column index
    pub col: usize,
}