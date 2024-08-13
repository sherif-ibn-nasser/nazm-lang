mod token;
mod cursor;
mod lexing_methods;
mod error;

use std::{cell::RefCell, str::Chars};
use documented::DocumentedVariants;
use error::{LexerError, LexerErrorType};
use strum::IntoEnumIterator;
use token::*;
use cursor::Cursor;
use crate::{diagnostics::Diagnostics, span::{Span, SpanCursor}};

pub struct Lexer<'a>{
    file_path: &'a str,
    file_lines: &'a Vec<String>,
    diagnostics: &'a RefCell<Diagnostics>,
    current_line_idx: usize,
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {

    pub fn new(file_path: &'a str, file_lines: &'a Vec<String>, diagnostics: &'a RefCell<Diagnostics>,) -> Self{
        Lexer {
            file_path: file_path,
            file_lines: file_lines,
            diagnostics: diagnostics,
            current_line_idx: 0,
            cursor: Cursor::new(&file_lines[0]),
        }
    }
    
    pub fn lex(&mut self) -> Vec<Token>{

        self.current_line_idx = 0;

        let mut tokens = vec![];

        while self.current_line_idx < self.file_lines.len() {
            
            self.cursor = Cursor::new(&self.file_lines[self.current_line_idx]);

            while self.cursor.has_remaining() {
                
                let typ = self.next_token_type();
                
                let (val ,start, end) = self.cursor.cut();
                
                let token = Token {
                    val: val,
                    span: Span {
                        start: SpanCursor{ line: self.current_line_idx, col: start },
                        end  : SpanCursor{ line: self.current_line_idx, col: end   },
                    },
                    typ: typ
                };

                tokens.push(token);
    
            }

            self.current_line_idx += 1;

        }

        tokens.push(
            Token {
                val:"\0",
                span: Span {
                    start: SpanCursor{ line: self.current_line_idx, col: self.cursor.get_start_remainder() },
                    end  : SpanCursor{ line: self.current_line_idx, col: self.cursor.get_start_remainder() },
                },
                typ: TokenType::EOF
            }
        );

        return tokens;
    }

    fn get_diagnostics(&self) -> std::cell::RefMut<Diagnostics> {
        self.diagnostics.borrow_mut()
    }

    fn next_token_type(&mut self) -> TokenType{
        if let Some(typ) = self.find_string_or_char_token() {
            typ
        }
        else if let Some(typ) = self.find_comment_token()  {
            typ
        }
        else if let Some(typ) = self.find_symbol_token()  {
            typ
        }
        else if let Some(typ) = self.find_number_token()  {
            typ
        }
        else {
            TokenType::Bad(vec![])
        }
        
    }
    
    fn find_comment_token(&mut self) -> Option<TokenType> {
        if self.cursor.select_if_starts_with("\\") {
            self.cursor.select_remainder();
            return Some(TokenType::LineComment);
        }
        return None;
    }
    
    fn find_symbol_token(&mut self) -> Option<TokenType> {

        // Lex longer symbols first  as they may start with another smaller symbol

        if self.cursor.select_if_starts_with("<<="){
            return Some(TokenType::Symbol(SymbolType::ShrEqual));
        }
        if self.cursor.select_if_starts_with(">>="){
            return Some(TokenType::Symbol(SymbolType::ShlEqual));
        }
        if self.cursor.select_if_starts_with("**="){
            return Some(TokenType::Symbol(SymbolType::PowerEqual));
        }
        if self.cursor.select_if_starts_with("<<"){
            return Some(TokenType::Symbol(SymbolType::Shr));
        }
        if self.cursor.select_if_starts_with(">>"){
            return Some(TokenType::Symbol(SymbolType::Shl));
        }
        if self.cursor.select_if_starts_with("**"){
            return Some(TokenType::Symbol(SymbolType::Power));
        }
        if self.cursor.select_if_starts_with("++"){
            return Some(TokenType::Symbol(SymbolType::PlusPlus));
        }
        if self.cursor.select_if_starts_with("--"){
            return Some(TokenType::Symbol(SymbolType::MinusMinus));
        }
        if self.cursor.select_if_starts_with(">="){
            return Some(TokenType::Symbol(SymbolType::GreaterEqual));
        }
        if self.cursor.select_if_starts_with("<="){
            return Some(TokenType::Symbol(SymbolType::LessEqual));
        }
        if self.cursor.select_if_starts_with("=="){
            return Some(TokenType::Symbol(SymbolType::EqualEqual));
        }
        if self.cursor.select_if_starts_with("!="){
            return Some(TokenType::Symbol(SymbolType::NotEqual));
        }
        if self.cursor.select_if_starts_with("&&"){
            return Some(TokenType::Symbol(SymbolType::LogicalAnd));
        }
        if self.cursor.select_if_starts_with("||"){
            return Some(TokenType::Symbol(SymbolType::LogicalOr));
        }
        if self.cursor.select_if_starts_with("+="){
            return Some(TokenType::Symbol(SymbolType::PLusEqual));
        }
        if self.cursor.select_if_starts_with("-="){
            return Some(TokenType::Symbol(SymbolType::MinusEqual));
        }
        if self.cursor.select_if_starts_with("*="){
            return Some(TokenType::Symbol(SymbolType::StarEqual));
        }
        if self.cursor.select_if_starts_with("/="){
            return Some(TokenType::Symbol(SymbolType::SlashEqual));
        }
        if self.cursor.select_if_starts_with("%="){
            return Some(TokenType::Symbol(SymbolType::ModuloEqual));
        }
        if self.cursor.select_if_starts_with("~="){
            return Some(TokenType::Symbol(SymbolType::BitNotEqual));
        }
        if self.cursor.select_if_starts_with("&="){
            return Some(TokenType::Symbol(SymbolType::BitAndEqual));
        }
        if self.cursor.select_if_starts_with("^="){
            return Some(TokenType::Symbol(SymbolType::XorEqual));
        }
        if self.cursor.select_if_starts_with("|="){
            return Some(TokenType::Symbol(SymbolType::BitOrEqual));
        }
        if self.cursor.select_if_starts_with("::"){
            return Some(TokenType::Symbol(SymbolType::DoubleColons));
        }
        if self.cursor.select_if_starts_with("،"){
            return Some(TokenType::Symbol(SymbolType::Comma));
        }
        if self.cursor.select_if_starts_with("؛"){
            return Some(TokenType::Symbol(SymbolType::Semicolon));
        }
        if self.cursor.select_if_starts_with("؟"){
            return Some(TokenType::Symbol(SymbolType::QuestionMark));
        }
        if self.cursor.select_if_starts_with("<"){
            return Some(TokenType::Symbol(SymbolType::OpenAngleBracketOrLess));
        }
        if self.cursor.select_if_starts_with(">"){
            return Some(TokenType::Symbol(SymbolType::CloseAngleBracketOrGreater));
        }
        if self.cursor.select_if_starts_with("("){
            return Some(TokenType::Symbol(SymbolType::OpenParenthesis));
        }
        if self.cursor.select_if_starts_with(")"){
            return Some(TokenType::Symbol(SymbolType::CloseParenthesis));
        }
        if self.cursor.select_if_starts_with("{"){
            return Some(TokenType::Symbol(SymbolType::OpenCurlyBraces));
        }
        if self.cursor.select_if_starts_with("}"){
            return Some(TokenType::Symbol(SymbolType::CloseCurlyBraces));
        }
        if self.cursor.select_if_starts_with("["){
            return Some(TokenType::Symbol(SymbolType::OpenSquareBracket));
        }
        if self.cursor.select_if_starts_with("]"){
            return Some(TokenType::Symbol(SymbolType::CloseSquareBracket));
        }
        if self.cursor.select_if_starts_with(":"){
            return Some(TokenType::Symbol(SymbolType::Colon));
        }
        if self.cursor.select_if_starts_with("!"){
            return Some(TokenType::Symbol(SymbolType::ExclamationMark));
        }
        if self.cursor.select_if_starts_with("~"){
            return Some(TokenType::Symbol(SymbolType::BitNot));
        }
        if self.cursor.select_if_starts_with("&"){
            return Some(TokenType::Symbol(SymbolType::BitAnd));
        }
        if self.cursor.select_if_starts_with("^"){
            return Some(TokenType::Symbol(SymbolType::Xor));
        }
        if self.cursor.select_if_starts_with("|"){
            return Some(TokenType::Symbol(SymbolType::BitOr));
        }
        if self.cursor.select_if_starts_with("."){
            return Some(TokenType::Symbol(SymbolType::Dot));
        }
        if self.cursor.select_if_starts_with("+"){
            return Some(TokenType::Symbol(SymbolType::Plus));
        }
        if self.cursor.select_if_starts_with("-"){
            return Some(TokenType::Symbol(SymbolType::Minus));
        }
        if self.cursor.select_if_starts_with("*"){
            return Some(TokenType::Symbol(SymbolType::Star));
        }
        if self.cursor.select_if_starts_with("/"){
            return Some(TokenType::Symbol(SymbolType::Slash));
        }
        if self.cursor.select_if_starts_with("="){
            return Some(TokenType::Symbol(SymbolType::Equal));
        }
        if self.cursor.select_if_starts_with("%"){
            return Some(TokenType::Symbol(SymbolType::Modulo));
        }
        return None;
    }

}

struct LexerIter<'a>{
    text: &'a str,
    cursor: CharsCursor<'a>,
    /// The byte index the cursor stopped at
    stopped_at_bidx: usize,
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = (Span, TokenType, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.cursor.stopped_at.0;
        let start_byte = self.stopped_at_bidx;
        let typ = self.next_token_type();
        if let TokenType::EOF = typ {
            return None;
        }
        let end = self.cursor.stopped_at.0;
        let end_byte = self.stopped_at_bidx;
        let val = &self.text[start_byte..end_byte];
        Some((Span { start: start, end: end }, typ, val))
    }
}

impl<'a> LexerIter<'a> {

    fn new(text: &'a str) -> Self {
        let mut _self = Self { text: text, cursor: CharsCursor::new(text), stopped_at_bidx: 0 };
        _self.cursor.next(); // Init cursor::stopped_at with first char
        _self
    }

    fn next_token_type(&mut self) -> TokenType {

        match self.cursor.stopped_at.1 {
            '/' => self.next_token_with_slash(),
            '،' => { self.next_cursor(); TokenType::Symbol(SymbolType::Comma) }
            '؛' => { self.next_cursor(); TokenType::Symbol(SymbolType::Semicolon) }
            '؟' => { self.next_cursor(); TokenType::Symbol(SymbolType::QuestionMark) }
            '\n' => { self.next_cursor(); TokenType::EOL }
            '0'..='9' => self.next_num_token(),
            '\'' | '\"' => self.next_str_or_char_token(),
            '\t' | '\x0C' | '\r' | ' ' => {
                while self.next_cursor().is_some_and(|(_, ch)| ch.is_ascii_whitespace() && ch != '\n') {} // Skip whitespaces
                TokenType::Space
            }
            any => {

                if self.stopped_at_bidx == self.text.len() {
                    return TokenType::EOF;
                }

                let text = &self.text[self.stopped_at_bidx..];

                for symbol in SymbolType::iter() {
                    if symbol.get_variant_docs().is_ok_and(|val| text.starts_with(val)){
                        for _ in 0..symbol.get_variant_docs().unwrap().len() { // The multibyte symbols are checked above
                            self.next_cursor();
                        }
                        return TokenType::Symbol(symbol);
                    }
                }
                
                if !any.is_alphabetic() {
                    self.next_cursor();
                    return TokenType::Bad(vec![]);
                }

                self.next_id_or_keyword()
            },
        }

    }

    fn next_cursor(&mut self) -> Option<(SpanCursor, char)> {
        let size = self.cursor.stopped_at.1.len_utf8();

        self.stopped_at_bidx += size;

        self.cursor.next()
    }

    fn next_cursor_non_eol(&mut self) -> Option<(SpanCursor, char)> {
        match self.next_cursor() {
            Some((_, '\n')) => None,
            any => any,
        }
    }

    fn next_token_with_slash(&mut self) -> TokenType {

        match self.next_cursor() {
            Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::SlashEqual) }
            Some((_, '/')) => {
                while self.next_cursor_non_eol().is_some(){} // Skip all until first EOL
                TokenType::LineComment
            }
            Some((_, '*')) => {
                let mut opened_delimted_comments = 1;

                while let Some((_, ch)) = self.next_cursor() {

                    if opened_delimted_comments == 0 {
                        break;
                    }

                    if ch == '/' && self.next_cursor().is_some_and(|(_, ch)| ch == '*') {
                        opened_delimted_comments += 1;
                    }

                    else if ch == '*' && self.next_cursor().is_some_and(|(_, ch)| ch == '/') {
                        opened_delimted_comments -= 1;
                    }

                }

                if opened_delimted_comments == 0 {
                    TokenType::DelimitedComment
                }
                else {
                    TokenType::Bad(vec![
                        LexerError {
                            col: self.cursor.stopped_at.0.col,
                            len: 1,
                            typ: LexerErrorType::UnclosedDelimitedComment,
                        }
                    ])
                }
            }
            _ => TokenType::Symbol(SymbolType::Slash),
        }


    }

    fn next_id_or_keyword(&mut self) -> TokenType {

        let start = self.stopped_at_bidx;

        while self.next_cursor_non_eol().is_some_and(|(_, ch)| ch.is_alphanumeric() || ch == '_' ) {}

        let end = self.stopped_at_bidx;
        
        let id = &self.text[start..end];

        for keyword_typ in KeywordType::iter() {
            if keyword_typ.get_variant_docs().is_ok_and(|val| id == val) {
                return TokenType::Keyword(keyword_typ);
            }
        }

        return TokenType::Id;

    }

}

#[derive(Clone)]
struct CharsCursor<'a> {
    stopped_at: (SpanCursor, char),
    chars: Chars<'a>,
}

impl<'a> CharsCursor<'a> {

    fn new(text: &'a str) -> Self {
        Self {
            stopped_at: (
                SpanCursor { line: 0, col: 0},
                '\0'
            ),
            chars: text.chars(),
        }
    }

}

impl<'a> Iterator for CharsCursor<'a> {

    type Item = (SpanCursor, char);

    fn next(&mut self) -> Option<Self::Item> {

        if self.stopped_at.1 != '\0' {  // Not the first time to call the `next` method
            self.stopped_at.0.col += 1; // Update the column
        }
        if self.stopped_at.1 == '\n' { // Last was a new line character
            self.stopped_at.0.line += 1; // Update the line
            self.stopped_at.0.col = 0; // Reset the column
        }

        match self.chars.next() {
            Some(ch) => {
                self.stopped_at.1 = ch; // Update the character
                Some(self.stopped_at)
            }
            None => {
                self.stopped_at.1 = '\0'; // Update the character
                None
            }
        }
    }
    
}

#[cfg(test)]

mod tests{
    use documented::DocumentedVariants;
    use strum::IntoEnumIterator;
    use crate::{lexer::TokenType, span::{Span, SpanCursor}};
    use super::{KeywordType, LexerIter, SymbolType};

    #[test]
    fn test_symbols_lexing() {
        for symbol in SymbolType::iter() {
            let symbol_val = symbol.get_variant_docs().unwrap();
            let (span, typ, val) = LexerIter::new(&symbol_val).next().unwrap();
            assert_eq!(
                span,
                Span { 
                    start: SpanCursor { line:0, col: 0 },
                    end:   SpanCursor { line:0, col: symbol_val.chars().count() },
                }
            );
            assert_eq!(val, symbol_val);
            assert_eq!(typ, TokenType::Symbol(symbol));
        }

        let mut symbols_line = String::new();
        for symbol in SymbolType::iter() {
            let symbol_val = symbol.get_variant_docs().unwrap();
            symbols_line.push_str(symbol_val);
        }

        let mut tokens = LexerIter::new(&symbols_line);
        let mut symbols_iter = SymbolType::iter();
        let mut columns = 0;

        while let Some((span, typ, val)) = tokens.next() {
            let symbol = symbols_iter.next().unwrap();
            let symbol_val = symbol.get_variant_docs().unwrap();

            assert_eq!(
                span,
                Span { 
                    start: SpanCursor { line:0, col: columns },
                    end:   SpanCursor { line:0, col: columns + symbol_val.chars().count() },
                },
                "Maybe the tokens are overlapping for left: `{}`, right: `{}`",val, symbol_val
            );

            columns += symbol_val.chars().count();

            assert_eq!(val, symbol_val);
            assert_eq!(typ, TokenType::Symbol(symbol));
        }

        let mut symbols_line = String::new();
        for symbol in SymbolType::iter() {
            let symbol_val = symbol.get_variant_docs().unwrap();
            symbols_line.push_str(symbol_val);
            symbols_line.push('\n');
        }

        let mut tokens = LexerIter::new(&symbols_line);
        let mut symbols_iter = SymbolType::iter();
        let mut lines = 0;

        while let Some((span, typ, val)) = tokens.next() {
            let symbol = symbols_iter.next().unwrap();
            let symbol_val = symbol.get_variant_docs().unwrap();

            assert_eq!(
                span,
                Span { 
                    start: SpanCursor { line: lines, col: 0 },
                    end:   SpanCursor { line: lines, col: symbol_val.chars().count() },
                },
                "Maybe the tokens are overlapping for left: `{}`, right: `{}`",val, symbol_val
            );

            assert_eq!(val, symbol_val);
            assert_eq!(typ, TokenType::Symbol(symbol));

            let (span, typ, val) = tokens.next().unwrap();

            assert_eq!(
                span,
                Span { 
                    start: SpanCursor { line: lines, col: symbol_val.chars().count() },
                    end:   SpanCursor { line: lines + 1, col: 0 },
                },
                "On symbol `{}`", symbol_val
            );

            assert_eq!(val, "\n");
            assert_eq!(typ, TokenType::EOL);
            lines += 1;
        }
    }

    #[test]
    fn test_keywords_lexing() {
        for keyword in KeywordType::iter() {
            let keyword_val = keyword.get_variant_docs().unwrap();
            let (span, typ, val) = LexerIter::new(&keyword_val).next().unwrap();
            assert_eq!(
                span,
                Span { 
                    start: SpanCursor { line:0, col: 0 },
                    end:   SpanCursor { line:0, col: keyword_val.chars().count() },
                }
            );
            assert_eq!(val, keyword_val);
            assert_eq!(typ, TokenType::Keyword(keyword));
        }
    }
}