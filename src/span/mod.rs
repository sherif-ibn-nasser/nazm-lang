#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Span{
    pub line: usize,
    pub start: usize,
    pub end: usize,
}