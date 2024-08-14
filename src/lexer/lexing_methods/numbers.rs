use std::num::{IntErrorKind, ParseIntError};

use error::*;

use crate::lexer::*;


impl<'a> LexerIter<'a> {

    pub(crate) fn next_num_token(&mut self) -> TokenType {

        let prefix = &self.text[self.stopped_at_bidx..];

        if prefix.starts_with("2#") {
            self.next_cursor(); self.next_cursor(); self.next_cursor(); // Skip "2#" and stop on next digit
            return self.next_num_sys_token(Base::Bin, |d| matches!(d, b'0' | b'1') );
        }

        if prefix.starts_with("8#") {
            self.next_cursor(); self.next_cursor(); self.next_cursor(); // Skip "8#" and stop on next digit
            return self.next_num_sys_token(Base::Oct, |d| matches!(d, b'0'..=b'7') );
        }

        if prefix.starts_with("10#") {
            self.next_cursor(); self.next_cursor(); self.next_cursor(); self.next_cursor(); // Skip "10#" and stop on next digit
            return self.next_num_sys_token(Base::Dec, |d| matches!(d, b'0'..=b'9') );
        }

        if prefix.starts_with("16#") {
            self.next_cursor(); self.next_cursor(); self.next_cursor(); self.next_cursor(); // Skip "16#" and stop on next digit
            return self.next_hex_num_token();
        }

        let digits = self.next_digits_array();

        match self.cursor.stopped_at.1 {
            '#' => {}
            '.' => {}
            _ => {}
        }
        todo!()
    }

    fn next_digits_array(&mut self) -> String {
        let stopped_char = self.cursor.stopped_at.1;
        let mut digits = String::new();

        if !stopped_char.is_ascii_digit() { return digits; }
        
        digits.push(stopped_char);

        while let Some((_, ch)) = self.next_cursor_non_eol(){
            if ch.is_ascii_digit() { digits.push(ch); }
            else if ch != ',' { break; } // Skip commas
        }

        digits
    }

    fn next_num_sys_token(&mut self, base: Base, match_digit: impl Fn(u8) -> bool) -> TokenType {

        let prefix_end_col = self.cursor.stopped_at.0.col;

        let digits = self.next_digits_array();

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
            Ok(suffix_str) => to_int_token(&digits, suffix_str, prefix_end_col, base),
            Err(mut err) => {
                err.typ = LexerErrorType::InvalidIntSuffixForBase(base);
                TokenType::Bad(vec![err])
            },
        }
    }

    fn next_hex_num_token(&mut self) -> TokenType {

        let prefix_end_col = self.cursor.stopped_at.0.col;
        let stopped_char = self.cursor.stopped_at.1;
        let mut digits = String::new();

        if !stopped_char.is_ascii_hexdigit() {
            let _ = self.next_valid_num_suffix(); // Skip any suffix
            return missing_digits_after_base_prefix_bad_token(prefix_end_col);
        }
        
        digits.push(stopped_char);

        while let Some((_, ch)) = self.next_cursor_non_eol(){
            if ch.is_ascii_hexdigit() { digits.push(ch); }
            else if ch != ',' { break; } // Skip commas
        }

        let suffix = self.next_valid_num_suffix();
        
        match suffix {
            Ok(suffix_str) => to_int_token(&digits, suffix_str, prefix_end_col, Base::Hex),
            Err(mut err) => {
                err.typ = LexerErrorType::InvalidIntSuffixForBase(Base::Hex);
                TokenType::Bad(vec![err])
            },
        }

    }

    fn next_valid_num_suffix(&mut self) -> Result<&str, LexerError> {

        if !self.cursor.stopped_at.1.is_alphabetic() {
            return Ok("");
        }

        let start_col = self.cursor.stopped_at.0.col;
        let start = self.stopped_at_bidx;

        while self.next_cursor_non_eol().is_some_and(|(_, ch)| ch.is_alphanumeric() || ch == '_' ) {}

        let end_col = self.cursor.stopped_at.0.col;
        let end = self.stopped_at_bidx;
        
        let id = &self.text[start..end];

        match id {
            "ص1" | "ص2" | "ص4" | "ص8" | "ص" |
            "م1" | "م2" | "م4" | "م8" | "م" |
            "ع4" | "ع8"
            => Ok(id),
            _ => Err(
                LexerError {
                    col: start_col,
                    len: end_col - start_col,
                    typ: LexerErrorType::InvalidNumSuffix,
                }
            ),
        }

    }

}

fn to_int_token(digits: &str, suffix_str: &str, prefix_end_col: usize, base: Base) -> TokenType {

    let radix = base.clone() as u32;

    if suffix_str.is_empty() {
        return match u64::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::UnspecifiedInt(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::UnspecifiedInt(0)
            )
        };
    }

    if suffix_str == "ص" {
        return match isize::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::I(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::I(0)
            )
        };
    }

    if suffix_str == "ص1" {
        return match i8::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::I1(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::I1(0)
            )
        };
    }

    if suffix_str == "ص2" {
        return match i16::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::I2(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::I2(0)
            )
        };
    }

    if suffix_str == "ص4" {
        return match i32::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::I4(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::I4(0)
            )
        };
    }

    if suffix_str == "ص8" {
        return match i64::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::I8(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::I8(0)
            )
        };
    }

    if suffix_str == "م" {
        return match usize::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::U(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::U(0)
            )
        };
    }

    if suffix_str == "م1" {
        return match u8::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::U1(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::U1(0)
            )
        };
    }

    if suffix_str == "م2" {
        return match u16::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::U2(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::U2(0)
            )
        };
    }

    if suffix_str == "م4" {
        return match u32::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::U4(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::U4(0)
            )
        };
    }

    if suffix_str == "م8" {
        return match u64::from_str_radix(&digits, radix) {
            Ok(num) => num_lit_token(NumType::U8(num)),
            Err(_) => num_is_out_of_range_bad_token(
                prefix_end_col,
                digits.len(),
                NumType::U8(0)
            )
        };
    }

    TokenType::Bad(vec![
        LexerError {
            col: prefix_end_col + digits.len(),
            len: suffix_str.len(),
            typ: LexerErrorType::InvalidIntSuffixForBase(base),
        }
    ])
}

#[inline]
fn missing_digits_after_base_prefix_bad_token(col: usize) -> TokenType {
    TokenType::Bad(vec![
        LexerError {
            col: col,
            len: 0, // Mark the prefix instead
            typ: LexerErrorType::MissingDigitsAfterBasePrefix,
        }
    ])
}

#[inline]
fn invalid_digit_for_base_prefix_bad_token(base: Base, col: usize) -> TokenType {
    TokenType::Bad(vec![
        LexerError {
            col: col,
            len: 1, // The first digit
            typ: LexerErrorType::InvalidDigitForBase(base),
        }
    ])
}

#[inline]
fn num_is_out_of_range_bad_token(col: usize, len: usize, num_type: NumType) -> TokenType {
    TokenType::Bad(vec![
        LexerError {
            col: col,
            len: len,
            typ: LexerErrorType::NumIsOutOfRange(num_type),
        }
    ])
}

#[inline]
fn num_lit_token(num_type: NumType) -> TokenType {
    TokenType::Literal(
        LiteralTokenType::Num(num_type)
    )
}