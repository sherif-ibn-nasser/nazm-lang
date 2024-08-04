
use std::{collections::HashMap, fs, path::PathBuf, process::exit};

use bpaf::Bpaf;

use owo_colors::{colors::*, OwoColorize};

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// The official compiler of Nazm programming language 
struct CLI{
    #[bpaf(positional("FILE"))]
    /// Nazm files to compile (with extensions *.نظم or *.نَظْم)
    files: Vec<PathBuf>,
}

fn print_err(msg: String){

    let err = "خطأ".bold();
    let err_col = ":".bold();
    let err_dot = ".".bold();

    println!("{}{} {}{}",err.bright_red(),err_col,msg,err_dot)
}

pub fn read_files() -> HashMap<String, Vec<String>>{

    let mut files: HashMap<String, Vec<String>> = HashMap::new();

    let files_paths=c_l_i().fallback_to_usage().run().files;

    let mut file_errors = false;
    
    let err_msg_str=format!(
        "{} {} {} {}{}",
        "يُتوقع ملف بامتداد".bold(),
        "*.نظم".bright_yellow().bold(),
        "أو".bold(),
        "*.نَظْم".bright_yellow().bold(),
        "، ولكن تم العثور على".bold()
    );

    for file_path in files_paths {

        let file_path_str = file_path.to_str().unwrap().to_string();


        match file_path.extension() {
            Some(ext) => {

                if ext != "نظم" && ext != "نَظْم"{
                    file_errors=true;
                    print_err(format!("{} {}", err_msg_str ,file_path_str.bright_red().bold()));
                    continue;
                }

            },

            None => {
                file_errors=true;
                print_err(format!("{} {}", err_msg_str ,file_path_str.bright_red().bold()));
                continue;
            },
        }
        
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let file_lines = content
                    .lines()
                    .map(String::from)
                    .collect();


                files.insert(file_path_str, file_lines);
            },
            Err(_) => {
                file_errors=true;
                print_err(format!("{} {} {}", "لا يمكن قراءة الملف".bold() ,file_path_str.bright_red().bold(), "أو أنه غير موجود".bold()));
                continue;
            },
        }

    }

    if file_errors {
        exit(1)
    }

    return files;
}

fn main() {
    let files = read_files();
    for (file_path,file_lines) in files.into_iter(){
        println!("{}",file_path)
    }
    println!("Hello, world!");
}
