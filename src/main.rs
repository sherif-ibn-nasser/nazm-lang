mod diagnostics;
mod cli;
mod lexer;

use std::{cell::RefCell, collections::HashMap};
use diagnostics::Diagnostics;
use lexer::Lexer;

fn main() {

    let mut files: HashMap<String, Vec<String>> = HashMap::new();

    let diagnostics = RefCell::new(Diagnostics::new());

    cli::read_files(&mut files);

    for (file_path,file_lines) in files.iter(){
        let mut lexer = Lexer::new(file_path, file_lines, &diagnostics);
        let tokens = lexer.lex();
    }
    println!("Hello, world!");
}

struct LL<'a>{
    t: u16,
    u: u16,
    v: &'a str
}

impl<'a> LL <'a>{
    fn lex(&mut self) {
        while self.t < self.u {
            self.next_token();
            self.t += 1;
        }

        self.t=8;
        self.u=5;
    }

    fn next_token(&mut self){
        self.t=5;
    }
}