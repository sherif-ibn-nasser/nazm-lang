use std::collections::HashMap;

mod cli;

fn main() {

    let mut files: HashMap<String, Vec<String>> = HashMap::new();

    cli::read_files(&mut files);

    for (file_path,file_lines) in files.into_iter(){
        println!("{}",file_path)
    }
    println!("Hello, world!");
}
