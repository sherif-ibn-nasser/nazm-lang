mod diagnostics;
mod cli;
mod lexer;

use std::collections::HashMap;
use diagnostics::Diagnostics;
use lexer::Lexer;

fn main() {

    let mut files: HashMap<String, Vec<String>> = HashMap::new();

    let mut diagnostics = Diagnostics::new();

    cli::read_files(&mut files);

    for (file_path,file_lines) in files.iter(){
        let lexer = Lexer::new(file_path, file_lines, &mut diagnostics);
        let tokens = lexer.lex();
    }
    println!("Hello, world!");
}
