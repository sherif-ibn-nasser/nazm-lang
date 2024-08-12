
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
    UnclosedStr,
    UnclosedChar,
    UnclosedDelimitedComment,
    ZeroChars,
    ManyChars,
    KufrOrInvalidChar,
    UnicodeCodePointHexDigitOnly,
    InvalidUnicodeCodePoint,
    UnknownEscapeSequence,
}