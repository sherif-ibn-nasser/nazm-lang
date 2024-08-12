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
    LineComment,
    DelimitedComment,
    Literal(LiteralTokenType),
    Id,
    Symbol(SymbolType),
    Keyword,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LiteralTokenType{
    Str(String),
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
    /// `،`
    Comma,
    /// `؛`
    Semicolon,
    /// `؟`
    QuestionMark,
    /// `.`
    DOT,
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

    /// `<`
    OpenAngleBracketOrLess,
    /// `<=`
    LessEqual,
    /// `<<`
    Shr,
    /// `<<=`
    ShrEqual,

    /// `>`
    CloseAngleBracketOrGreater,
    /// `>=`
    GreaterEqual,
    /// `>>`
    Shl,
    /// `>>=`
    ShlEqual,

    /// `*`
    Star,
    /// `*=`
    StarEqual,
    /// `**`
    Power,
    /// `**=`
    PowerEqual,

    /// `/`
    Slash,
    /// `/=`
    SlashEqual,

    /// `+`
    Plus,
    /// `+=`
    PLusEqual,
    /// `++`
    PlusPlus,

    /// `-`
    Minus,
    /// `-=`
    MinusEqual,
    /// `--`
    MinusMinus,

    /// `|`
    BitOr,
    /// `|=`
    BitOrEqual,
    /// `||`
    LogicalOr,

    /// `&`
    BitAnd,
    /// `&=`
    BitAndEqual,
    /// `&&`
    LogicalAnd,

    /// `%`
    Modulo,
    /// `%=`
    ModuloEqual,

    /// `~`
    BitNot,
    /// `~=`
    BitNotEqual,

    /// `^`
    Xor,
    /// `^=`
    XorEqual,

    /// `=`
    Equal,
    /// `==`
    EqualEqual,

    /// `!`
    ExclamationMark,
    /// `!=`
    NotEqual,

    /// `:`
    Colon,
    /// `::`
    DoubleColons,
    
}