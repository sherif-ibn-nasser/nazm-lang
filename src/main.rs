mod cli;

use cli::print_err;
use nazmc_data_pool::DataPool;
use owo_colors::{OwoColorize, XtermColors};
use std::{
    fs,
    io::{self, Write},
    process::{exit, Command},
};
use yaml_rust2::{Yaml, YamlLoader};

fn collect_paths(paths: &Vec<Yaml>, prefix: &str, collected_paths: &mut Vec<String>) {
    for path in paths {
        match path {
            Yaml::String(s) => {
                if prefix.is_empty() {
                    collected_paths.push(s.clone());
                } else {
                    collected_paths.push(format!("{}/{}", prefix, s));
                }
            }
            Yaml::Hash(map) => {
                for (key, value) in map {
                    if let Yaml::String(key_str) = key {
                        let new_prefix = if prefix.is_empty() {
                            key_str.clone()
                        } else {
                            format!("{}/{}", prefix, key_str)
                        };

                        if let Yaml::Array(nested_paths) = value {
                            collect_paths(nested_paths, &new_prefix, collected_paths);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // let (file_path, file_content) = cli::read_file();

    let config = match fs::read_to_string("config.yaml") {
        Ok(content) => content,
        Err(_) => {
            print_err(format!("{}", "لم يتم العثور على ملف config.yaml".bold(),));
            exit(1);
        }
    };

    let config = &YamlLoader::load_from_str(&config).unwrap()[0];

    let name = config["الاسم"].as_str().unwrap_or("");

    let version = config["الإصدار"].as_str().unwrap_or("0.0.0");

    // Initialize a vector to hold the paths
    let mut collected_paths = Vec::new();

    // Extract and collect paths recursively
    if let Some(paths) = config["المسارات"].as_vec() {
        collect_paths(paths, "", &mut collected_paths);
    }

    // Output the collected paths
    for path in collected_paths {
        println!("{}", path);
    }

    // RTL printing
    let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
    io::stdout()
        .write_all(&output.stdout[1..output.stdout.len() - 1])
        .unwrap();

    // let mut id_pool = DataPool::new();
    // let mut str_pool = DataPool::new();

    // nazmc_parser::parse_file(&file_path, &file_content, &mut id_pool, &mut str_pool);

    // for Token { span, val, kind } in tokens {
    //     let color = match kind {
    //         TokenKind::LineComment | TokenKind::DelimitedComment => XtermColors::BrightTurquoise,
    //         TokenKind::Symbol(_) => XtermColors::UserBrightYellow,
    //         TokenKind::Id => XtermColors::LightAnakiwaBlue,
    //         TokenKind::Keyword(_) | TokenKind::Literal(LiteralKind::Bool(_)) => {
    //             XtermColors::FlushOrange
    //         }
    //         TokenKind::Literal(LiteralKind::Str(_) | LiteralKind::Char(_)) => {
    //             XtermColors::PinkSalmon
    //         }
    //         TokenKind::Literal(_) => XtermColors::ChelseaCucumber,
    //         _ => XtermColors::UserWhite,
    //     };

    //     let mut val = format!("{}", val.color(color));

    //     if matches!(kind, TokenKind::Keyword(_) | TokenKind::Symbol(_)) {
    //         val = format!("{}", val.bold());
    //     }

    //     print!("{}", val);
    // }
}
