
#[derive(Clone, PartialEq, Debug)]
pub struct LexerError {
    /// The start index to mark from
    pub col: usize,
    /// The length for marking (in bytes)
    pub len: usize,
    /// The type of error
    pub typ: LexerErrorType,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexerErrorType {
    UnclosedString,
    UnclosedChar,
    OneCharOnly,
    KufrOrInvalidChar,
    UnicodeCodePointHexDigitOnly,
    InvalidUnicodeCodePoint,
    UnknownEscapeSequence,
}