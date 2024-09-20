use std::rc::Rc;

use crate::*;

impl<'a> LexerIter<'a> {
    pub(crate) fn next_str_or_char_token(&mut self) -> TokenKind {
        let (start, quote) = self.cursor.stopped_at;

        let is_char = quote == '\'';

        let mut rust_str_lit = String::new();

        let mut errs = vec![];

        loop {
            match self.next_valid_nazm_rust_char_in_str(quote) {
                Err(mut err) => {
                    if err.kind == LexerErrorKind::UnclosedStr {
                        if is_char {
                            err.kind = LexerErrorKind::UnclosedChar;
                        }
                        self.errs.push(err);

                        return if is_char {
                            TokenKind::Literal(LiteralKind::Char('\0'))
                        } else {
                            TokenKind::Literal(LiteralKind::Str(Rc::new("".to_string())))
                        }; // Return unclosed delimiter errors early before validation of typed chars
                    }

                    errs.push(err);
                }
                Ok(option) => match option {
                    Some(ch) => rust_str_lit.push(ch),
                    None => {
                        self.next_cursor();
                        break;
                    } // The string is closed
                },
            }
        }

        if !is_char {
            self.errs.append(&mut errs);
            return TokenKind::Literal(LiteralKind::Str(Rc::new(rust_str_lit)));
        }

        let mut iter = rust_str_lit.chars();

        let ch = match iter.next() {
            None => {
                self.errs.push(LexerError {
                    token_idx: self.current_token_idx,
                    col: start.col,
                    len: 1,
                    kind: LexerErrorKind::ZeroChars,
                }); // Skip literal errors and show only this error
                return TokenKind::Literal(LiteralKind::Char('\0'));
            }
            Some(ch) => ch,
        };

        if iter.next().is_some() {
            self.errs.push(LexerError {
                token_idx: self.current_token_idx,
                col: start.col,
                len: 1,
                kind: LexerErrorKind::ManyChars,
            }); // Skip literal errors and show only this error
            return TokenKind::Literal(LiteralKind::Char('\0'));
        }

        self.errs.append(&mut errs); // Append literal errors
        TokenKind::Literal(LiteralKind::Char(ch))
    }

    fn next_valid_nazm_rust_char_in_str(
        &mut self,
        quote: char,
    ) -> Result<Option<char>, LexerError> {
        let ch = match self.next_cursor_non_eol() {
            Some((_, ch)) => {
                if ch == quote {
                    return Ok(None);
                }
                ch
            }
            None => return self.unclosed_delimiter_err(),
        };

        if ch != '\\' {
            return self.check_is_kufr_or_unsupported_char();
        }

        let ch = match self.next_cursor_non_eol() {
            Some((_, ch)) => ch,
            None => return self.unclosed_delimiter_err(),
        };

        if ch != 'ي' {
            return self.check_is_escape_sequence();
        }

        let start = self.cursor.stopped_at.0;

        let mut code_point_str = String::new();

        for _ in 0..4 {
            match self.next_cursor_non_eol() {
                Some((_, ch)) => code_point_str.push(ch),
                None => return self.unclosed_delimiter_err(),
            }
        }

        if code_point_str.len() != 4 || !code_point_str.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(LexerError {
                token_idx: self.current_token_idx,
                col: start.col + 1, // To start marking after `ي`
                len: code_point_str.len(),
                kind: LexerErrorKind::UnicodeCodePointHexDigitOnly,
            });
        }

        let code_point = u32::from_str_radix(&code_point_str, 16).unwrap();

        match char::from_u32(code_point) {
            Some(ch) => self.check_is_kufr_or_unsupported_char_unicode(ch, start),
            None => Err(LexerError {
                token_idx: self.current_token_idx,
                col: start.col + 1, // To start marking after `ي`
                len: 4,             // The 4 digits
                kind: LexerErrorKind::InvalidUnicodeCodePoint,
            }),
        }
    }

    #[inline]
    fn unclosed_delimiter_err(&self) -> Result<Option<char>, LexerError> {
        Err(LexerError {
            token_idx: self.current_token_idx,
            col: self.cursor.stopped_at.0.col,
            len: 1,
            kind: LexerErrorKind::UnclosedStr,
        })
    }

    #[inline]
    fn check_is_kufr_or_unsupported_char_unicode(
        &self,
        ch: char,
        start: SpanCursor,
    ) -> Result<Option<char>, LexerError> {
        if is_kufr_or_unsupported_character(ch) {
            Err(LexerError {
                token_idx: self.current_token_idx,
                col: start.col + 1, // To start marking after `ي`
                len: 4,             // The 4 digits
                kind: LexerErrorKind::KufrOrInvalidChar,
            })
        } else {
            Ok(Some(ch))
        }
    }

    #[inline]
    fn check_is_escape_sequence(&self) -> Result<Option<char>, LexerError> {
        let (start, ch) = self.cursor.stopped_at;

        match to_escape_sequence(ch) {
            None => Err(LexerError {
                token_idx: self.current_token_idx,
                col: start.col,
                len: 1,
                kind: LexerErrorKind::UnknownEscapeSequence,
            }),
            some => Ok(some),
        }
    }
}

fn to_escape_sequence(c: char) -> Option<char> {
    match c {
        'خ' => Some('\x08'), // مسافة للخلف
        'ر' => Some('\x0b'), // مسافة رأسية
        'ص' => Some('\x0c'), // الصفحة التالية
        'ف' => Some('\t'),   // مسافة أفقية
        'س' => Some('\n'),   // سطر جديد
        'ج' => Some('\r'),   // إرجاع المؤشر إلى بداية السطر، وبدء الكتابة منه
        '\\' => Some('\\'),
        '\'' => Some('\''),
        '\"' => Some('\"'),
        '0' => Some('\0'),
        _ => None,
    }
}
