use super::error::LexerError;
use documented::{DocumentedFields, DocumentedVariants};
use nazmc_diagnostics::span::*;
use strum::EnumIter;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Token<'a> {
    pub val: &'a str,
    pub span: Span,
    pub kind: TokenKind,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum TokenKind {
    #[default]
    EOF,
    EOL,
    Space,
    LineComment,
    DelimitedComment,
    Literal(LiteralKind),
    Id,
    Symbol(SymbolKind),
    Keyword(KeywordKind),
    Bad(Vec<LexerError>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum LiteralKind {
    Str(String),
    Char(char),
    Bool(bool),
    Num(NumKind),
}

#[derive(Clone, PartialEq, Debug)]
pub enum NumKind {
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
pub enum SymbolKind {
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
    /// #
    Hash,
}

#[derive(DocumentedVariants, Debug, Clone, PartialEq, EnumIter)]
pub enum KeywordKind {
    /// دالة
    Fn,
    /// احجز
    Let,
    /// متغير
    Mut,
    /// ثابت
    Const,
    /// على
    On,
}
