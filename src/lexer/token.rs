use crate::span::Span;

#[derive(Clone, PartialEq)]
pub struct Token<'a>{
    pub val: &'a str,
    pub span: Span,
    pub typ: TokenType,
}

#[derive(Clone, PartialEq)]
pub enum TokenType {
    Bad,
    EOL,
    EOF,
    Space,
    Comment,
    Literal(LiteralTokenType),
    Id,
    Symbol,
    Keyword,
}

#[derive(Clone, PartialEq)]
pub enum LiteralTokenType{
    String(String),
    Char(char),
    Bool(bool),
    Num(NumTokenType),
}

#[derive(Clone, Copy, PartialEq)]
pub enum NumTokenType {
    I(isize),
    I1(i8),
    I2(i16),
    I4(i32),
    I8(i64),
    U(usize),
    U1(u8),
    U2(u16),
    U4(u32),
    U8(u64),
    F4(f32),
    F8(f64),
}