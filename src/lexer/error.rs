use super::{Base, NumKind};

#[derive(Clone, PartialEq, Debug)]
pub struct LexerError {
    /// The index of character to start marking from
    pub col: usize,
    /// The length for marking (in characters)
    pub len: usize,
    /// The kind of error
    pub kind: LexerErrorKind,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexerErrorKind {
    UnknownToken,
    UnclosedStr,
    UnclosedChar,
    UnclosedDelimitedComment,
    ZeroChars,
    ManyChars,
    KufrOrInvalidChar,
    UnicodeCodePointHexDigitOnly,
    InvalidUnicodeCodePoint,
    UnknownEscapeSequence,
    MissingDigitsAfterBasePrefix,
    MissingDigitsAfterExponent,
    InvalidIntBasePrefix,
    InvalidNumSuffix,
    InvalidFloatSuffix,
    InvalidIntSuffix,
    InvalidDigitForBase(Base),
    NumIsOutOfRange(NumKind),
}
