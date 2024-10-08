use crate::*;

impl<'a> LexerIter<'a> {
    pub fn next_valid_nazm_rust_char_in_str(
        &mut self,
        mut chars: Chars,
        mut start_col: usize,
    ) -> String {
        let mut str = String::new();

        loop {
            let Some(ch) = chars.next() else {
                break;
            };

            start_col += 1;

            if ch != '\\' {
                if is_kufr_or_unsupported_character(ch) {
                    self.errs.push(LexerError {
                        token_idx: self.current_token_idx,
                        col: start_col,
                        len: 1,
                        kind: LexerErrorKind::KufrOrInvalidChar,
                    });
                } else {
                    str.push(ch);
                };
                continue;
            }

            start_col += 1;

            match chars.next() {
                Some('خ') => str.push('\x08'), // مسافة للخلف
                Some('ر') => str.push('\x0b'), // مسافة رأسية
                Some('ص') => str.push('\x0c'), // الصفحة التالية
                Some('ف') => str.push('\t'),   // مسافة أفقية
                Some('س') => str.push('\n'),   // سطر جديد
                Some('ج') => str.push('\r'),   // إرجاع المؤشر إلى بداية السطر، وبدء الكتابة منه
                Some('\\') => str.push('\\'),
                Some('\'') => str.push('\''),
                Some('\"') => str.push('\"'),
                Some('0') => str.push('\0'),
                Some('ي') => {
                    start_col += 1;
                    let ch = self.next_unicode_char(&mut chars, start_col);
                    start_col += 3;
                    str.push(ch);
                }
                _ => {
                    str.push('\0');
                    self.errs.push(LexerError {
                        token_idx: self.current_token_idx,
                        col: start_col,
                        len: 1,
                        kind: LexerErrorKind::UnknownEscapeSequence,
                    });
                }
            }
        }

        return str;
    }

    fn next_unicode_char(&mut self, chars: &mut Chars, start_col: usize) -> char {
        let mut code_point_str = String::new();
        let mut err = false;
        let mut len = 0;

        for _ in 0..4 {
            match chars.next() {
                Some(ch) if ch.is_ascii_hexdigit() => code_point_str.push(ch),
                Some(_) => {
                    err = true;
                }
                None => {
                    err = true;
                    break;
                }
            }
            len += 1;
        }

        if err {
            let (col, len) = if len == 0 {
                // Mark starting from the slash if no digits found
                (start_col - 2, 2)
            } else {
                (start_col, len)
            };

            self.errs.push(LexerError {
                token_idx: self.current_token_idx,
                col,
                len,
                kind: LexerErrorKind::UnicodeCodePointHexDigitOnly,
            });
            return '\0';
        }

        let code_point = u32::from_str_radix(&code_point_str, 16).unwrap();

        return match char::from_u32(code_point) {
            Some(ch) if is_kufr_or_unsupported_character(ch) => {
                self.errs.push(LexerError {
                    token_idx: self.current_token_idx,
                    col: start_col - 2, // mark starting from the slash
                    len: 6,
                    kind: LexerErrorKind::KufrOrInvalidChar,
                });
                '\0'
            }
            Some(ch) => ch,
            None => {
                self.errs.push(LexerError {
                    token_idx: self.current_token_idx,
                    col: start_col - 2, // mark starting from the slash
                    len: 6,
                    kind: LexerErrorKind::InvalidUnicodeCodePoint,
                });
                '\0'
            }
        };
    }
}
