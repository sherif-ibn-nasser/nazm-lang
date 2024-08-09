mod diagnostics;
mod cli;
mod lexer;
mod span;

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