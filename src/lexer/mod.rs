mod token;
mod cursor;
mod lexing_methods;

use std::cell::RefCell;
use token::*;
use cursor::Cursor;
use crate::{diagnostics::Diagnostics, span::Span};

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

        for (i, line) in self.file_lines.iter().enumerate() {
            
            self.cursor = Cursor::new(&line);

            while self.cursor.has_remaining() {
                
                let typ = self.next_token_type();
                
                let (val ,start, end) = self.cursor.cut();
                
                let token = Token {
                    val: val,
                    span: Span {
                        line: i,
                        start: start,
                        end: end,
                    },
                    typ: typ
                };

                tokens.push(token);
    
            }

        }

        tokens.push(
            Token {
                val:"\0",
                span: Span {
                    line: self.current_line_idx+1,
                    start: 0,
                    end: 0,
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
        else {
            TokenType::Bad
        }
        
    }
    
    fn find_comment_token(&mut self) -> Option<TokenType> {
        todo!()
    }

}