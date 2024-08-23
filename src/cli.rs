use std::{collections::HashMap, fs, path::PathBuf, process::exit};

use bpaf::Bpaf;

use owo_colors::OwoColorize;

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// The official compiler of Nazm programming language 
struct CLI{
    #[bpaf(positional("FILE"))]
    /// Nazm file to compile (with *.نظم extension)
    file: PathBuf,
}

fn print_err(msg: String){

    let err = "خطأ".bold();
    let err_col = ":".bold();
    let err_dot = ".".bold();

    println!("{}{} {}{}",err.bright_red(),err_col,msg,err_dot)
}

pub fn read_file() -> (PathBuf, String){

    let file_path=c_l_i().fallback_to_usage().run().file;
    
    let err_msg_str=format!(
        "{} {}{}",
        "يُتوقع ملف بامتداد".bold(),
        "*.نظم".bright_yellow().bold(),
        "، ولكن تم العثور على".bold()
    );

    let file_path_str = file_path.to_str().unwrap().to_string();


    match file_path.extension() {
        Some(ext) => {

            if ext != "نظم"{
                print_err(format!("{} {}", err_msg_str ,file_path_str.bright_red().bold()));
                exit(1);
            }

        },

        None => {
            print_err(format!("{} {}", err_msg_str ,file_path_str.bright_red().bold()));
            exit(1);
        },
    }
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            (file_path, content)
        },
        Err(_) => {
            print_err(format!("{} {} {}", "لا يمكن قراءة الملف".bold() ,file_path_str.bright_red().bold(), "أو أنه غير موجود".bold()));
            exit(1);
        },
    }

}