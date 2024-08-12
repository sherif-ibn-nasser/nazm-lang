use error::*;

use crate::lexer::*;

impl<'a> Lexer<'a> {
    
    pub(crate) fn find_string_or_char_token(&mut self) -> Option<TokenType> {

        if !self.cursor.select_if_starts_with("\"") && !self.cursor.select_if_starts_with("\'"){
            return None;
        }

        let mut rust_lit = String::new(); // The literal as rust literal

        let quote = self.cursor.last_selected();

        let is_char = quote == '\'';

        // The error in chars and their starts and their lengths (To error them only instead of the whole token)
        let mut lit_errors: Vec<LexerError> = vec![];

        let mut closed = false;

        self.cursor.select_while(|remaining: &str, remaining_start: usize|{
            
            let mut chars=remaining.chars();

            let c = chars.next().unwrap_or_default();

            if closed {
                return 0; // Stop selecting any characters
            }

            if c == quote {
                closed = true;
                return 1; // Select the closing quote
            }

            // A normal char
            if c != '\\'{
                if is_kufr_or_unsupported_character(c) {
                    lit_errors.push(
                        LexerError{
                            col: remaining_start,
                            len: c.len_utf8(),
                            typ: LexerErrorType::KufrOrInvalidChar,
                        }
                    );
                    return 1;
                }
                rust_lit.push(c);
                return 1; // Select this only char
            }

            let c = chars.next().unwrap_or_default(); // The char after the slash

            // A unicode char
            if c == 'ي' {

                let next_4_chars=chars.take(4);

                if !next_4_chars.clone().all(|c|{ c.is_ascii_hexdigit() }){
                    let next_4_chars_size=next_4_chars.fold(0, |acc, c|{ acc+c.len_utf8() });
                    lit_errors.push(
                        LexerError{
                            col: remaining_start,
                            len: next_4_chars_size,
                            typ: LexerErrorType::UnicodeCodePointHexDigitOnly,
                        }
                    );
                    return 6;
                }

                let code_point_str = next_4_chars.collect::<String>();

                let code_point=u32::from_str_radix(&code_point_str,16).unwrap();

                match char::from_u32(code_point){
                    Some(c) => {
                        if is_kufr_or_unsupported_character(c) {
                            lit_errors.push(
                                LexerError{
                                    col: remaining_start,
                                    len: 7, // The length is 7 bytes as for (`\`, `ي` and the four digits
                                    typ: LexerErrorType::KufrOrInvalidChar,
                                }
                            );
                        }
                        rust_lit.push(c);
                    }
                    None =>
                        lit_errors.push(
                            LexerError{
                                col: remaining_start,
                                len: 7, // The length is 7 bytes as for (`\`, `ي` and the four digits
                                typ: LexerErrorType::InvalidUnicodeCodePoint,
                            }
                        )
                }

                return 6; // Select the `\` and `ي` and the 4 digits of the code-point
            }

            // An escape sequence
            match to_escape_sequence(c) {
                Some(c) => rust_lit.push(c),
                None => 
                    lit_errors.push(
                        LexerError{
                            col: remaining_start,
                            len: c.len_utf8() + 1, // The length is 1 byte for `\` and the char's size
                            typ: LexerErrorType::UnknownEscapeSequence,
                        }
                    )
            }

            return 2; // Select the `\` and the next char
        });

        if !closed {
            lit_errors.push(
                LexerError{
                    col: self.cursor.get_start_remainder(),
                    len: 1,
                    typ: if is_char { LexerErrorType::UnclosedChar } else { LexerErrorType::UnclosedStr },
                }
            );
        }


        if is_char && rust_lit.chars().count() != 1 {
            lit_errors.push(
                LexerError{
                    col: self.cursor.get_start(),
                    len: self.cursor.get_selceted().len(),
                    typ: LexerErrorType::ManyChars,
                }
            );
        }

        if !lit_errors.is_empty() {
            return Some(TokenType::Bad(lit_errors));
        }

        Some(
            TokenType::Literal(
                if is_char {
                    LiteralTokenType::Char(rust_lit.chars().next().unwrap())
                }
                else {
                    LiteralTokenType::Str(rust_lit)
                }
            )
        )
    }
}

impl<'a> LexerIter<'a> {

    pub(crate) fn next_str_or_char_token(&mut self) -> TokenType {

        let (start, quote) = self.cursor.stopped_at;

        let is_char = quote == '\'';

        let mut rust_str_lit = String::new();

        let mut errs = vec![];

        loop {

            match self.next_valid_nazm_rust_char_in_str() {
                Err(mut err) => {
                    if err.typ == LexerErrorType::UnclosedStr {
                        if is_char {
                            err.typ = LexerErrorType::UnclosedChar;
                        }
                        return TokenType::Bad(vec![err]); // Return unclosed delimiter errors early before validation of typed chars
                    }
    
                    errs.push(err);
                }
                Ok(option) => match option {
                    Some(ch) => rust_str_lit.push(ch),
                    None => break, // The string is closed
                }
            }

        }

        if !is_char {
            if !errs.is_empty() {
                return TokenType::Bad(errs);
            }

            return TokenType::Literal(LiteralTokenType::Str(rust_str_lit));
        }

        let mut iter = rust_str_lit.chars();

        let ch = match iter.next() {
            None => return TokenType::Bad(vec![LexerError{
                col: start.col,
                len: 1,
                typ: LexerErrorType::ZeroChars
            }]),
            Some(ch) => ch
        };

        if iter.next().is_some() {
            return TokenType::Bad(vec![LexerError{
                col: start.col,
                len: 1,
                typ: LexerErrorType::ManyChars
            }])
        }

        if !errs.is_empty() {
            return TokenType::Bad(errs);
        }

        return TokenType::Literal(LiteralTokenType::Char(ch));

    }

    fn next_valid_nazm_rust_char_in_str(&mut self) -> Result<Option<char>, LexerError> {

        let ch = match self.next_cursor_non_eol() {
            Some((_, '\"')) => return Ok(None),
            Some((_, ch)) => ch,
            None => return self.unclosed_delimiter_err(),
        };

        if ch != '\\' {
            return self.check_is_kufr_or_unsupported_char()
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

        if code_point_str.len() != 4 || !code_point_str.chars().all(|ch| { ch.is_ascii_hexdigit() } ){
            return Err(
                LexerError {
                    col: start.col + 1, // To start marking after `ي` 
                    len: code_point_str.len(),
                    typ: LexerErrorType::UnicodeCodePointHexDigitOnly,
                }
            );
        }

        let code_point = u32::from_str_radix(&code_point_str,16).unwrap();

        match char::from_u32(code_point){
            Some(ch) => self.check_is_kufr_or_unsupported_char_unicode(ch, start),
            None => Err(
                LexerError{
                    col: start.col + 1, // To start marking after `ي` 
                    len: 4, // The 4 digits
                    typ: LexerErrorType::InvalidUnicodeCodePoint,
                }
            )
        }
    }

    #[inline]
    fn unclosed_delimiter_err(&self) -> Result<Option<char>, LexerError> {
        Err(
            LexerError {
                col: self.cursor.stopped_at.0.col,
                len: 1,
                typ: LexerErrorType::UnclosedStr,
            }
        )
    }

    #[inline]
    fn check_is_kufr_or_unsupported_char_unicode(&self, ch: char, start: SpanCursor) -> Result<Option<char>, LexerError> {
        if is_kufr_or_unsupported_character(ch) {
            Err(
                LexerError {
                    col: start.col + 1, // To start marking after `ي`
                    len: 4, // The 4 digits
                    typ: LexerErrorType::KufrOrInvalidChar,
                }
            )
        }
        else { Ok(Some(ch)) }
    }

    #[inline]
    fn check_is_kufr_or_unsupported_char(&self) -> Result<Option<char>, LexerError> {
        let (start, ch) = self.cursor.stopped_at;

        if is_kufr_or_unsupported_character(ch) {
            Err(
                LexerError {
                    col: start.col,
                    len: 1,
                    typ: LexerErrorType::KufrOrInvalidChar,
                }
            )
        }
        else { Ok(Some(ch)) }
    }

    #[inline]
    fn check_is_escape_sequence(&self) -> Result<Option<char>, LexerError> {

        let (start, ch) = self.cursor.stopped_at;

        match to_escape_sequence(ch) {
            None => Err(
                LexerError {
                    col: start.col,
                    len: 1,
                    typ: LexerErrorType::UnknownEscapeSequence,
                }
            ),
            some => Ok(some)
        }

    }

}

fn is_kufr_or_unsupported_character(c:char) -> bool{
    let chars=[
        '\u{03EE}','\u{03EF}','\u{058d}','\u{058e}',
        '\u{05EF}', // yod triangle
        '\u{07D9}','\u{093B}','\u{13D0}','\u{16BE}','\u{165C}','\u{16ED}',
        '\u{17D2}','\u{1D7B}','\u{2020}','\u{2021}','\u{256A}','\u{256B}',
        '\u{256C}','\u{2616}','\u{2617}','\u{269C}','\u{269E}','\u{269F}',
        '\u{26AF}','\u{26B0}','\u{26B1}','\u{26F3}','\u{26F9}','\u{26FB}',
        '\u{26FF}','\u{27CA}','\u{29FE}','\u{2CFE}',
    ];

    if chars.contains(&c){
        return true
    }

    let ranges=[
        /*  from  ,    to  */
        ('\u{0900}','\u{109F}'),//HinduEurope
        ('\u{1100}','\u{1C7F}'),//HinduEurope
        ('\u{253C}','\u{254B}'),
        ('\u{2624}','\u{2638}'),//Kufr
        ('\u{263D}','\u{2653}'),//Kufr
        ('\u{2654}','\u{2667}'),
        ('\u{2669}','\u{2671}'),//Music and kufr crosses
        ('\u{2680}','\u{268F}'),
        ('\u{2680}','\u{268F}'),
        ('\u{26A2}','\u{26A9}'),// Pride
        ('\u{26B3}','\u{26BC}'),// Kufr
        ('\u{26BF}','\u{26EC}'),
        ('\u{2719}','\u{2725}'),// Kufr crosses
        ('\u{2BF0}','\u{2C5F}'),// Includes astrology
        ('\u{2D80}','\u{AB2F}'),
        ('\u{AB70}','\u{FAFF}'),
    ];

    for (r1,r2) in ranges{
        if c>=r1 && c<=r2{
            return true
        }
    }

    return false
}

fn to_escape_sequence(c:char) -> Option<char>{
    match c {
        'خ' => Some('\x08'),   // مسافة للخلف
        'ر' => Some('\x0b'),   // مسافة رأسية
        'ص' => Some('\x0c'),   // الصفحة التالية
        'ف' => Some('\t')  ,   // مسافة أفقية
        'س' => Some('\n')  ,   // سطر جديد
        'ج' => Some('\r')  ,   // إرجاع المؤشر إلى بداية السطر، وبدء الكتابة منه
        '\\'=> Some('\\')  ,
        '\''=> Some('\'')  ,
        '\"'=> Some('\"')  ,
        '0' => Some('\0')  ,
        _   => None
    }
}


#[cfg(test)]
mod tests{

    use std::cell::RefCell;

    use crate::diagnostics::Diagnostics;

    use super::*;

    #[test]
    fn test_lexing_chars_pass(){
        let lines = vec![
            "'ن'".to_owned(),
            "'ظ'".to_owned(),
            "'م'".to_owned(),
            "' '".to_owned(),
            "'N'".to_owned(),
            "'\\ي0041'".to_owned(), // A
            "'\\خ'".to_owned(),
            "'\\ر'".to_owned(),
            "'\\ص'".to_owned(),
            "'\\ف'".to_owned(),
            "'\\س'".to_owned(),
            "'\\ج'".to_owned(),
            "'\\\\'".to_owned(),
            "'\\\''".to_owned(),
            "'\\\"'".to_owned(),
        ];

        let expected_chars = vec![
            'ن',
            'ظ',
            'م',
            ' ',
            'N',
            '\u{0041}', // A
            '\x08',
            '\x0b',
            '\x0c',
            '\t',
            '\n',
            '\r',
            '\\',
            '\'',
            '\"',
        ];

        let diagnostics = RefCell::new(Diagnostics::new());

        let mut lexer = Lexer::new(
            "test.نظم",
            &lines,
            &diagnostics
        );

        let tokens = lexer.lex();

        for (i, line) in lines.iter().enumerate() {
            assert_eq!(line, &tokens[i].val);
            assert_eq!(
                tokens[i].typ,
                TokenType::Literal(
                    LiteralTokenType::Char(expected_chars[i])
                )
            );
        }

        assert_eq!(tokens.last().unwrap().typ, TokenType::EOF);

    }

    #[test]
    fn test_lexing_strings_pass(){
        let lines = vec![
            "\"string \\خ\\ر\\ص\\ف\\س\\ج\\\\ نص literal with\\\'\\\' \\\"QUOTES\\\" \\س and \\ي0041\\ي0042\"".to_owned(),
        ];

        let expected = "string \x08\x0b\x0c\t\n\r\\ نص literal with\'\' \"QUOTES\" \n and \u{0041}\u{0042}".to_owned();

        let diagnostics = RefCell::new(Diagnostics::new());

        let mut lexer = Lexer::new(
            "test.نظم",
            &lines,
            &diagnostics
        );

        let tokens = lexer.lex();
        assert_eq!(lines[0], tokens[0].val);

        assert_eq!(
            tokens[0].typ,
            TokenType::Literal(
                LiteralTokenType::Str(expected)
            )
        );

        assert_eq!(tokens.last().unwrap().typ, TokenType::EOF);

    }
}