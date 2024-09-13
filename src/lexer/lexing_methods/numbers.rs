use error::*;

use crate::lexer::*;

impl<'a> LexerIter<'a> {
    pub(crate) fn next_num_token(&mut self) -> TokenKind {
        let prefix = &self.content[self.stopped_at_bidx..];

        if prefix.starts_with("2#") {
            self.next_cursor();
            self.next_cursor();
            self.next_cursor(); // Skip "2#" and stop on next digit
            return self.next_num_sys_token(Base::Bin, |d| matches!(d, b'0' | b'1'));
        }

        if prefix.starts_with("8#") {
            self.next_cursor();
            self.next_cursor();
            self.next_cursor(); // Skip "8#" and stop on next digit
            return self.next_num_sys_token(Base::Oct, |d| matches!(d, b'0'..=b'7'));
        }

        if prefix.starts_with("10#") {
            self.next_cursor();
            self.next_cursor();
            self.next_cursor();
            self.next_cursor(); // Skip "10#" and stop on next digit
            return self.next_num_sys_token(Base::Dec, |d| matches!(d, b'0'..=b'9'));
        }

        if prefix.starts_with("16#") {
            self.next_cursor();
            self.next_cursor();
            self.next_cursor();
            self.next_cursor(); // Skip "16#" and stop on next digit
            return self.next_hex_num_token();
        }

        let start_col = self.cursor.stopped_at.0.col;

        let mut digits = self.next_digits_array();

        if self.cursor.stopped_at.1 == '#' {
            let digits_len = self.cursor.stopped_at.0.col - start_col;
            self.next_hex_num_token(); // Fabricate and skip any hex digits and any suffixes
            return TokenKind::Bad(vec![LexerError {
                col: start_col,
                len: digits_len,
                kind: LexerErrorKind::InvalidIntBasePrefix,
            }]);
        }

        let dot_or_exp = &self.content[self.stopped_at_bidx..];

        if !dot_or_exp.starts_with("^^") && !dot_or_exp.starts_with(".") {
            let digits_len = self.cursor.stopped_at.0.col - start_col;

            return match self.next_valid_num_suffix() {
                Ok(suffix_str) => {
                    // Maybe an int or maybe a float depending on the suffix
                    let int_token =
                        to_int_token(&digits, suffix_str, start_col, digits_len, Base::Dec);
                    if !matches!(int_token, TokenKind::Bad(_)) {
                        return int_token;
                    }
                    let float_token = to_float_token(&digits, suffix_str, start_col, digits_len);
                    if let TokenKind::Bad(mut errs) = float_token {
                        errs[0].kind = LexerErrorKind::InvalidNumSuffix;
                        return TokenKind::Bad(errs);
                    }
                    return float_token;
                }
                Err(err) => TokenKind::Bad(vec![err]),
            };
        }

        if dot_or_exp.starts_with(".") {
            let after_dot = &self.content[self.stopped_at_bidx + 1..];

            // Number before dot may be treated as an int object, so check if after the dot is a digit to build the float
            if !after_dot.starts_with(|ch: char| ch.is_ascii_digit()) {
                let digits_len = self.cursor.stopped_at.0.col - start_col;

                // After the dot is not a digit, so treat it as an int with no suffix
                return to_int_token(&digits, "", start_col, digits_len, Base::Dec);
            }

            digits.push('.');

            // Append digits after the dot
            while let Some((_, ch)) = self.next_cursor_non_eol() {
                if ch.is_ascii_digit() {
                    digits.push(ch);
                } else if ch != ',' {
                    break;
                } // Skip commas
            }
        }

        let dot_or_exp = &self.content[self.stopped_at_bidx..];

        if dot_or_exp.starts_with("^^") {
            digits.push('E');

            self.next_cursor();
            self.next_cursor(); // Skip the "^^"

            if matches!(self.cursor.stopped_at.1, '+' | '-') {
                digits.push(self.cursor.stopped_at.1); // Append the sign
                self.next_cursor(); // Skip the sign
            }

            match self.cursor.stopped_at.1 {
                '0'..='9' => {
                    digits.push(self.cursor.stopped_at.1); // Append the first digit
                                                           // Append digits after the exponent
                    while let Some((_, ch)) = self.next_cursor_non_eol() {
                        if ch.is_ascii_digit() {
                            digits.push(ch);
                        } else if ch != ',' {
                            break;
                        } // Skip commas
                    }
                }
                _ => {
                    return TokenKind::Bad(vec![LexerError {
                        col: start_col,
                        len: 0, // Mark the number instead
                        kind: LexerErrorKind::MissingDigitsAfterExponent,
                    }]);
                }
            }
        }

        let digits_len = self.cursor.stopped_at.0.col - start_col;

        match self.next_valid_num_suffix() {
            Ok(suffix_str) => to_float_token(&digits, suffix_str, start_col, digits_len),
            Err(err) => TokenKind::Bad(vec![err]),
        }
    }

    fn next_digits_array(&mut self) -> String {
        let stopped_char = self.cursor.stopped_at.1;
        let mut digits = String::new();

        if !stopped_char.is_ascii_digit() {
            return digits;
        }

        digits.push(stopped_char);

        while let Some((_, ch)) = self.next_cursor_non_eol() {
            if ch.is_ascii_digit() {
                digits.push(ch);
            } else if ch != ',' {
                break;
            } // Skip commas
        }

        digits
    }

    fn next_num_sys_token(&mut self, base: Base, match_digit: impl Fn(u8) -> bool) -> TokenKind {
        let prefix_end_col = self.cursor.stopped_at.0.col;

        let digits = self.next_digits_array();

        let digits_len = self.cursor.stopped_at.0.col - prefix_end_col;

        let suffix = self.next_valid_num_suffix();

        if digits.is_empty() {
            return missing_digits_after_base_prefix_bad_token(prefix_end_col);
        }

        for (i, digit) in digits.bytes().enumerate() {
            if !match_digit(digit) {
                return invalid_digit_for_base_prefix_bad_token(Base::Bin, prefix_end_col + i);
            }
        }

        match suffix {
            Ok(suffix_str) => to_int_token(&digits, suffix_str, prefix_end_col, digits_len, base),
            Err(mut err) => {
                err.kind = LexerErrorKind::InvalidIntSuffixForBase(base);
                TokenKind::Bad(vec![err])
            }
        }
    }

    fn next_hex_num_token(&mut self) -> TokenKind {
        let prefix_end_col = self.cursor.stopped_at.0.col;
        let stopped_char = self.cursor.stopped_at.1;
        let mut digits = String::new();

        if !stopped_char.is_ascii_hexdigit() {
            let _ = self.next_valid_num_suffix(); // Skip any suffixes
            return missing_digits_after_base_prefix_bad_token(prefix_end_col);
        }

        digits.push(stopped_char);

        while let Some((_, ch)) = self.next_cursor_non_eol() {
            if ch.is_ascii_hexdigit() {
                digits.push(ch);
            } else if ch != ',' {
                break;
            } // Skip commas
        }

        let digits_len = self.cursor.stopped_at.0.col - prefix_end_col;
        let suffix = self.next_valid_num_suffix();

        match suffix {
            Ok(suffix_str) => {
                to_int_token(&digits, suffix_str, prefix_end_col, digits_len, Base::Hex)
            }
            Err(mut err) => {
                err.kind = LexerErrorKind::InvalidIntSuffixForBase(Base::Hex);
                TokenKind::Bad(vec![err])
            }
        }
    }

    fn next_valid_num_suffix(&mut self) -> Result<&str, LexerError> {
        if !self.cursor.stopped_at.1.is_alphabetic() {
            return Ok("");
        }

        let start_col = self.cursor.stopped_at.0.col;
        let start = self.stopped_at_bidx;

        while self
            .next_cursor_non_eol()
            .is_some_and(|(_, ch)| ch.is_alphanumeric() || ch == '_')
        {}

        let end_col = self.cursor.stopped_at.0.col;
        let end = self.stopped_at_bidx;

        let id = &self.content[start..end];

        match id {
            "ص1" | "ص2" | "ص4" | "ص8" | "ص" | "م1" | "م2" | "م4" | "م8" | "م" | "ع4" | "ع8" => {
                Ok(id)
            }
            _ => Err(LexerError {
                col: start_col,
                len: end_col - start_col,
                kind: LexerErrorKind::InvalidNumSuffix,
            }),
        }
    }
}

fn to_int_token(
    digits: &str,
    suffix_str: &str,
    start_col: usize,
    len: usize,
    base: Base,
) -> TokenKind {
    let radix = base.clone() as u32;

    if suffix_str.is_empty() {
        return match u64::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::UnspecifiedInt(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::UnspecifiedInt(0)),
        };
    }

    if suffix_str == "ص" {
        return match isize::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::I(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::I(0)),
        };
    }

    if suffix_str == "ص1" {
        return match i8::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::I1(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::I1(0)),
        };
    }

    if suffix_str == "ص2" {
        return match i16::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::I2(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::I2(0)),
        };
    }

    if suffix_str == "ص4" {
        return match i32::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::I4(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::I4(0)),
        };
    }

    if suffix_str == "ص8" {
        return match i64::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::I8(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::I8(0)),
        };
    }

    if suffix_str == "م" {
        return match usize::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::U(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::U(0)),
        };
    }

    if suffix_str == "م1" {
        return match u8::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::U1(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::U1(0)),
        };
    }

    if suffix_str == "م2" {
        return match u16::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::U2(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::U2(0)),
        };
    }

    if suffix_str == "م4" {
        return match u32::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::U4(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::U4(0)),
        };
    }

    if suffix_str == "م8" {
        return match u64::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumKind::U8(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::U8(0)),
        };
    }

    TokenKind::Bad(vec![LexerError {
        col: start_col + len,
        len: suffix_str.len(),
        kind: LexerErrorKind::InvalidIntSuffixForBase(base),
    }])
}

fn to_float_token(digits: &str, suffix_str: &str, start_col: usize, len: usize) -> TokenKind {
    if suffix_str.is_empty() {
        return match digits.parse() {
            Ok(num) => num_lit_token(NumKind::UnspecifiedFloat(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::UnspecifiedFloat(0.0)),
        };
    }

    if suffix_str == "ع4" {
        return match digits.parse() {
            Ok(num) => num_lit_token(NumKind::F4(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::F4(0.0)),
        };
    }

    if suffix_str == "ع8" {
        return match digits.parse() {
            Ok(num) => num_lit_token(NumKind::F8(num)),
            Err(_) => num_is_out_of_range_bad_token(start_col, len, NumKind::F8(0.0)),
        };
    }

    TokenKind::Bad(vec![LexerError {
        col: start_col + len,
        len: suffix_str.len(),
        kind: LexerErrorKind::InvalidFloatSuffix,
    }])
}

#[inline]
fn missing_digits_after_base_prefix_bad_token(col: usize) -> TokenKind {
    TokenKind::Bad(vec![LexerError {
        col: col,
        len: 0, // Mark the prefix instead
        kind: LexerErrorKind::MissingDigitsAfterBasePrefix,
    }])
}

#[inline]
fn invalid_digit_for_base_prefix_bad_token(base: Base, col: usize) -> TokenKind {
    TokenKind::Bad(vec![LexerError {
        col: col,
        len: 1, // The first digit
        kind: LexerErrorKind::InvalidDigitForBase(base),
    }])
}

#[inline]
fn num_is_out_of_range_bad_token(col: usize, len: usize, num_type: NumKind) -> TokenKind {
    TokenKind::Bad(vec![LexerError {
        col: col,
        len: len,
        kind: LexerErrorKind::NumIsOutOfRange(num_type),
    }])
}

#[inline]
fn num_lit_token(num_type: NumKind) -> TokenKind {
    TokenKind::Literal(LiteralKind::Num(num_type))
}
