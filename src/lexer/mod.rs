mod token;
mod cursor;
mod lexing_methods;
mod error;

use std::cell::RefCell;
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
            return Some(TokenType::Comment);
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
            return Some(TokenType::Symbol(SymbolType::SHR));
        }
        if self.cursor.select_if_starts_with(">>"){
            return Some(TokenType::Symbol(SymbolType::SHL));
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
            return Some(TokenType::Symbol(SymbolType::OpenAngleBracket));
        }
        if self.cursor.select_if_starts_with(">"){
            return Some(TokenType::Symbol(SymbolType::CloseAngleBracket));
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
            return Some(TokenType::Symbol(SymbolType::COLON));
        }
        if self.cursor.select_if_starts_with("!"){
            return Some(TokenType::Symbol(SymbolType::ExclamationMark));
        }
        if self.cursor.select_if_starts_with("~"){
            return Some(TokenType::Symbol(SymbolType::BitNot));
        }
        if self.cursor.select_if_starts_with("&"){
            return Some(TokenType::Symbol(SymbolType::AMPERSAND));
        }
        if self.cursor.select_if_starts_with("^"){
            return Some(TokenType::Symbol(SymbolType::XOR));
        }
        if self.cursor.select_if_starts_with("|"){
            return Some(TokenType::Symbol(SymbolType::BAR));
        }
        if self.cursor.select_if_starts_with("."){
            return Some(TokenType::Symbol(SymbolType::DOT));
        }
        if self.cursor.select_if_starts_with("\""){
            return Some(TokenType::Symbol(SymbolType::DoubleQuote));
        }
        if self.cursor.select_if_starts_with("\'"){
            return Some(TokenType::Symbol(SymbolType::SingleQuote));
        }
        if self.cursor.select_if_starts_with("\\"){
            return Some(TokenType::Symbol(SymbolType::BackSlash));
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