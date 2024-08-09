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
        else {
            TokenType::Bad(vec![])
        }
        
    }
    
    fn find_comment_token(&mut self) -> Option<TokenType> {
        todo!()
    }

}