mod token;
mod cursor;
mod lexing_methods;
mod error;

use std::{cell::RefCell, char, slice::Iter, str::Chars, usize};
use error::{LexerError, LexerErrorType};
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
            return Some(TokenType::Symbol(SymbolType::DOT));
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
        Some((Span { start: start, end: end}, typ, val))
    }
}

impl<'a> LexerIter<'a> {

    fn new(text: &'a str) -> Self {
        let mut _self = Self { text: text, cursor: CharsCursor::new(text), stopped_at_bidx: 0 };
        _self.cursor.next(); // Init cursor::stopped_at with first char
        _self
    }

    fn next_token_type(&mut self) -> TokenType{

        match self.cursor.stopped_at.1 {
            '\'' | '\"' => self.next_str_or_char_token(),
            '/' => self.next_token_with_slash(),
            '،' => { self.next_cursor(); TokenType::Symbol(SymbolType::Comma) }
            '؛' => { self.next_cursor(); TokenType::Symbol(SymbolType::Semicolon) }
            '؟' => { self.next_cursor(); TokenType::Symbol(SymbolType::QuestionMark) }
            '.' => { self.next_cursor(); TokenType::Symbol(SymbolType::DOT) }
            '(' => { self.next_cursor(); TokenType::Symbol(SymbolType::OpenParenthesis) }
            ')' => { self.next_cursor(); TokenType::Symbol(SymbolType::CloseParenthesis) }
            '{' => { self.next_cursor(); TokenType::Symbol(SymbolType::OpenCurlyBraces) }
            '}' => { self.next_cursor(); TokenType::Symbol(SymbolType::CloseCurlyBraces) }
            '[' => { self.next_cursor(); TokenType::Symbol(SymbolType::OpenSquareBracket) }
            ']' => { self.next_cursor(); TokenType::Symbol(SymbolType::CloseSquareBracket) }
            '<' => match self.next_cursor() {
                Some((_, '<')) => match self.next_cursor() {
                    Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::ShrEqual) }
                    _ => TokenType::Symbol(SymbolType::Shr),
                }
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::LessEqual) }
                _ => TokenType::Symbol(SymbolType::OpenAngleBracketOrLess),
            }
            '>' => match self.next_cursor() {
                Some((_, '>')) => match self.next_cursor() {
                    Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::ShlEqual) }
                    _ => TokenType::Symbol(SymbolType::Shl),
                }
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::GreaterEqual) }
                _ => TokenType::Symbol(SymbolType::CloseAngleBracketOrGreater),
            }
            '*' => match self.next_cursor() {
                Some((_, '*')) => match self.next_cursor() {
                    Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::PowerEqual) }
                    _ => TokenType::Symbol(SymbolType::Power),
                }
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::StarEqual) }
                _ => TokenType::Symbol(SymbolType::Star),
            }
            '+' => match self.next_cursor() {
                Some((_, '+')) => { self.next_cursor(); TokenType::Symbol(SymbolType::PlusPlus) }
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::PLusEqual) }
                _ => TokenType::Symbol(SymbolType::Plus),
            }
            '-' => match self.next_cursor() {
                Some((_, '-')) => { self.next_cursor(); TokenType::Symbol(SymbolType::MinusMinus) }
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::MinusEqual) }
                _ => TokenType::Symbol(SymbolType::Minus),
            }
            '|' => match self.next_cursor() {
                Some((_, '|')) => { self.next_cursor(); TokenType::Symbol(SymbolType::LogicalOr) }
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::BitOrEqual) }
                _ => TokenType::Symbol(SymbolType::BitOr),
            }
            '&' => match self.next_cursor() {
                Some((_, '&')) => { self.next_cursor(); TokenType::Symbol(SymbolType::LogicalAnd) }
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::BitAndEqual) }
                _ => TokenType::Symbol(SymbolType::BitAnd),
            }
            '%' => match self.next_cursor() {
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::ModuloEqual) }
                _ => TokenType::Symbol(SymbolType::Modulo),
            }
            '~' => match self.next_cursor() {
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::BitNotEqual) }
                _ => TokenType::Symbol(SymbolType::BitNot),
            }
            '^' => match self.next_cursor() {
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::XorEqual) }
                _ => TokenType::Symbol(SymbolType::Xor),
            }
            '=' => match self.next_cursor() {
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::EqualEqual) }
                _ => TokenType::Symbol(SymbolType::Equal),
            }
            '!' => match self.next_cursor() {
                Some((_, '=')) => { self.next_cursor(); TokenType::Symbol(SymbolType::NotEqual) }
                _ => TokenType::Symbol(SymbolType::ExclamationMark),
            }
            ':' => match self.next_cursor() {
                Some((_, ':')) => { self.next_cursor(); TokenType::Symbol(SymbolType::DoubleColons) }
                _ => TokenType::Symbol(SymbolType::Colon),
            }
            _ => TokenType::EOF,
        }

    }

    fn next_cursor(&mut self) -> Option<(SpanCursor, char)> {
        self.stopped_at_bidx += self.cursor.stopped_at.1.len_utf8();
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
        match self.chars.next() {
            Some(ch) => {
                if self.stopped_at.1 != '\0' {  // Not the first time to call the `next` method
                    self.stopped_at.0.col += 1; // Update the column
                }
                if self.stopped_at.1 == '\n' { // Last was a new line character
                    self.stopped_at.0.line += 1; // Update the line
                    self.stopped_at.0.col = 0; // Reset the column
                }
                self.stopped_at.1 = ch; // Update the character
                Some(self.stopped_at)
            }
            None => None
        }
    }
    
}

#[cfg(test)]

mod tests{

    #[test]
    fn test() {
        let mut it="شريف".chars().enumerate();
        for i in 0..2 {
            println!("{}: {}", i, it.next().unwrap().1);
        }
    }
}