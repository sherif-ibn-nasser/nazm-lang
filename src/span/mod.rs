#[derive(Clone, Copy, PartialEq)]
pub struct Span{
    pub line: usize,
    pub start: usize,
    pub end: usize,
}