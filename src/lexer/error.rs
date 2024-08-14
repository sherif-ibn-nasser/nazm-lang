use super::{Base, NumType};

#[derive(Clone, PartialEq, Debug)]
pub struct LexerError {
    /// The index of character to start marking from
    pub col: usize,
    /// The length for marking (in characters)
    pub len: usize,
    /// The type of error
    pub typ: LexerErrorType,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexerErrorType {
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
    InvalidDigitForBase(Base),
    InvalidIntBasePrefix,
    InvalidNumSuffix,
    InvalidFloatSuffix,
    InvalidIntSuffixForBase(Base),
    NumIsOutOfRange(NumType),
}