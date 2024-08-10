use crate::span::Span;

use super::error::LexerError;

#[derive(Clone, PartialEq, Debug)]
pub struct Token<'a>{
    pub val: &'a str,
    pub span: Span,
    pub typ: TokenType,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    Bad(Vec<LexerError>),
    EOL,
    EOF,
    Space,
    Comment,
    Literal(LiteralTokenType),
    Id,
    Symbol(SymbolType),
    Keyword,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LiteralTokenType{
    String(String),
    Char(char),
    Bool(bool),
    F4(f32),
    F8(f64),
    I(isize, Radix),
    I1(i8, Radix),
    I2(i16, Radix),
    I4(i32, Radix),
    I8(i64, Radix),
    U(usize, Radix),
    U1(u8, Radix),
    U2(u16, Radix),
    U4(u32, Radix),
    U8(u64, Radix),
    UnspecifiedInt(u64, Radix),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Radix {
    Bin = 2,
    Oct = 8,
    Dec = 10,
    Hex = 16,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SymbolType {
    /// `<<=`
    ShrEqual,
    /// `>>=`
    ShlEqual,
    /// `**=`
    PowerEqual,
    /// `<<`
    SHR,
    /// `>>`
    SHL,
    /// `**`
    Power,
    /// `++`
    PlusPlus,
    /// `--`
    MinusMinus,
    /// `>=`
    GreaterEqual,
    /// `<=`
    LessEqual,
    /// `==`
    EqualEqual,
    /// `!=`
    NotEqual,
    /// `&&`
    LogicalAnd,
    /// `||`
    LogicalOr,
    /// `+=`
    PLusEqual,
    /// `-=`
    MinusEqual,
    /// `*=`
    StarEqual,
    /// `/=`
    SlashEqual,
    /// `%=`
    ModuloEqual,
    /// `~=`
    BitNotEqual,
    /// `&=`
    BitAndEqual,
    /// `^=`
    XorEqual,
    /// `|=`
    BitOrEqual,
    /// `::`
    DoubleColons,
    /// `،`
    Comma,
    /// `؛`
    Semicolon,
    /// `؟`
    QuestionMark,
    /// `<`
    OpenAngleBracket,
    /// `>`
    CloseAngleBracket,
    /// `(`
    OpenParenthesis,
    /// `)`
    CloseParenthesis,
    /// `{`
    OpenCurlyBraces,
    /// `}`
    CloseCurlyBraces,
    /// `[`
    OpenSquareBracket,
    /// `]`
    CloseSquareBracket,
    /// `:`
    COLON,
    /// `!`
    ExclamationMark,
    /// `~`
    BitNot,
    /// `&`
    AMPERSAND,
    /// `^`
    XOR,
    /// `|`
    BAR,
    /// `.`
    DOT,
    /// `\"`
    DoubleQuote,
    /// `\'`
    SingleQuote,
    /// `\\`
    BackSlash,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `=`
    Equal,
    /// `%`
    Modulo,
}