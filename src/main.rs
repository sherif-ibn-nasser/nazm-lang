use std::collections::HashMap;

mod cli;

mod lexer;

use lexer::Lexer;

fn main() {

    let mut files: HashMap<String, Vec<String>> = HashMap::new();

    cli::read_files(&mut files);

    for (file_path,file_lines) in files.into_iter(){
        let lexer = Lexer::new(&file_path, &file_lines);
    }
    println!("Hello, world!");
}
