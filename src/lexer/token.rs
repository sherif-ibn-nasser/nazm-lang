use documented::{DocumentedFields, DocumentedVariants};
use strum::EnumIter;

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
    Keyword(KeywordType),
}

#[derive(Clone, PartialEq, Debug)]
pub enum LiteralTokenType{
    Str(String),
    Char(char),
    Bool(bool),
    Num(NumType),
}

#[derive(Clone, PartialEq, Debug)]
pub enum NumType {
    F4(f32),
    F8(f64),
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
    UnspecifiedInt(u64),
    UnspecifiedFloat(f64),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Base {
    Bin = 2,
    Oct = 8,
    Dec = 10,
    Hex = 16,
}

#[derive(Clone, PartialEq, Debug, DocumentedVariants, DocumentedFields, EnumIter)]
pub enum SymbolType {

    /// <..<
    LessDotDotLess,
    /// <..
    LessDotDot,
    /// ..<
    DotDotLess,
    /// <<=
    ShrEqual,
    /// >>=
    ShlEqual,
    /// **=
    PowerEqual,

    /// <=
    LessEqual,
    /// <<
    Shr,
    /// >=
    GreaterEqual,
    /// >>
    Shl,
    /// *=
    StarEqual,
    /// **
    Power,
    /// /=
    SlashEqual,
    /// +=
    PLusEqual,
    /// ++
    PlusPlus,
    /// -=
    MinusEqual,
    /// --
    MinusMinus,
    /// |=
    BitOrEqual,
    /// ||
    LogicalOr,
    /// &=
    BitAndEqual,
    /// &&
    LogicalAnd,
    /// %=
    ModuloEqual,
    /// ~=
    BitNotEqual,
    /// ^=
    XorEqual,
    /// ==
    EqualEqual,
    /// !=
    NotEqual,
    /// ::
    DoubleColons,
    /// ..
    DotDot,

    /// ،
    Comma,
    /// ؛
    Semicolon,
    /// ؟
    QuestionMark,
    /// (
    OpenParenthesis,
    /// )
    CloseParenthesis,
    /// {
    OpenCurlyBraces,
    /// }
    CloseCurlyBraces,
    /// [
    OpenSquareBracket,
    /// ]
    CloseSquareBracket,
    /// .
    Dot,
    /// <
    OpenAngleBracketOrLess,
    /// >
    CloseAngleBracketOrGreater,
    /// *
    Star,
    /// /
    Slash,
    /// +
    Plus,
    /// -
    Minus,
    /// |
    BitOr,
    /// &
    BitAnd,
    /// %
    Modulo,
    /// ~
    BitNot,
    /// ^
    Xor,
    /// !
    ExclamationMark,
    /// :
    Colon,
    /// =
    Equal,
    
}

#[derive(DocumentedVariants, Debug, Clone, PartialEq, EnumIter)]
pub enum KeywordType {
    /// دالة
    Fn,
    /// احجز
    Let,
    /// متغير
    Mut,
    /// ثابت
    Const,
}