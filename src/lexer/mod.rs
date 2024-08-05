mod token;

use std::cell::{Cell, RefCell};

use token::*;
use crate::diagnostics::Diagnostics;

pub struct Lexer<'a>{
    file_path: &'a str,
    file_lines: &'a Vec<String>,
    diagnostics: &'a RefCell<Diagnostics>,
    current_line_idx: Cell<usize>,
    current_col_idx: Cell<usize>,
}

impl<'a> Lexer<'a> {

    pub fn new(file_path: &'a str, file_lines: &'a Vec<String>, diagnostics: &'a RefCell<Diagnostics>,) -> Self{
        Lexer {
            file_path: file_path,
            file_lines: file_lines,
            diagnostics: diagnostics,
            current_line_idx: Cell::new(0),
            current_col_idx: Cell::new(0),
        }
    }
    
    pub fn lex(&self) -> Vec<Token> {

        let mut tokens = vec![];

        while self.current_line_idx.get() < self.file_lines.len() {
            let token = self.next_token();
            tokens.push(token);
            if token.typ == TokenType::EOL {
                self.current_line_idx.set(self.current_line_idx.get()+1);
            }
        }

        tokens.push(Token{val:"", typ: TokenType::EOF});

        return tokens;
    }

    fn next_token(&self) -> Token {

        if let Some(token) = self.find_string_or_char_token() {
            return token;
        }

        if let Some(token) = self.find_comment_token() {
            return token;
        }

        Token { val: "", typ: TokenType::Bad }
    }
    
    fn find_string_or_char_token(&self) -> Option<Token> {
        todo!()
    }
    
    fn find_comment_token(&self) -> Option<Token> {
        todo!()
    }

    fn diagnostics_mut(&self) -> std::cell::RefMut<Diagnostics> {
        self.diagnostics.borrow_mut()
    }

}