use token::Token;

mod token;
use crate::diagnostics::Diagnostics;

pub struct Lexer<'a>{
    file_path: &'a str,
    file_lines: &'a Vec<String>,
    diagnostics: &'a mut Diagnostics,
}

impl<'a> Lexer<'a> {

    pub fn new(file_path: &'a str, file_lines: &'a Vec<String>, diagnostics: &'a mut Diagnostics,) -> Self{
        Lexer {
            file_path: file_path,
            file_lines: file_lines,
            diagnostics: diagnostics
        }
    }

    pub fn lex(&self) -> Vec<Token<'a>>{
        todo!()
    }

}