pub struct Lexer<'a>{
    file_path: &'a str,
    file_lines: &'a Vec<String>,
}

impl<'a> Lexer<'a> {

    pub fn new(file_path: &'a str, file_lines: &'a Vec<String>,) -> Self{
        Lexer { file_path: file_path, file_lines: file_lines }
    }
    
}