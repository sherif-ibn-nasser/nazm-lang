mod cli;
use cli::print_err;
use nazmc_data_pool::DataPool;
use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::span::Span;
use nazmc_lexer::LexerIter;
use nazmc_parser::parse;
use owo_colors::OwoColorize;
use serde::Deserialize;
use serde_yaml::Value;
use std::io;
use std::io::Write;
use std::{
    collections::HashMap,
    fs, panic,
    process::{exit, Command},
};
use thin_vec::ThinVec;

fn collect_paths(paths: Vec<Value>, prefix: &str, collected_paths: &mut Vec<String>) {
    for path in paths {
        match path {
            Value::String(s) => {
                if prefix.is_empty() {
                    collected_paths.push(s.clone());
                } else {
                    collected_paths.push(format!("{}/{}", prefix, s));
                }
            }
            Value::Mapping(mapping) => {
                for (key, value) in mapping {
                    if let Value::String(key_str) = key {
                        let new_prefix = if prefix.is_empty() {
                            key_str.clone()
                        } else {
                            format!("{}/{}", prefix, key_str)
                        };

                        if let Value::Sequence(nested_paths) = value {
                            collect_paths(nested_paths, &new_prefix, collected_paths);
                        }
                    }
                }
            }
            s => todo!("{:?}", s),
        }
    }
}

#[derive(Deserialize)]
struct NazmYaml {
    الاسم: Option<String>,
    الإصدار: Option<String>,
    المسارات: Vec<Value>,
}

fn get_file_paths() -> Vec<String> {
    let nazm_yaml = match fs::read_to_string("nazm.yaml") {
        Ok(content) => content,
        Err(_) => {
            print_err(format!("{}", "لم يتم العثور على ملف nazm.yaml".bold(),));
            exit(1);
        }
    };

    let mut val = serde_yaml::from_str::<Value>(&nazm_yaml).unwrap();

    val.apply_merge().unwrap();

    let Ok(NazmYaml {
        الاسم,
        الإصدار,
        المسارات,
    }) = serde_yaml::from_value(val)
    else {
        print_err(format!(
            "{}",
            "ملف nazm.yaml يجب أن يحتوي على خاصية `المسارات` مع مسار ملف واحد على الأقل".bold(),
        ));
        exit(1)
    };

    let mut collected_paths = Vec::new();

    collect_paths(المسارات, "", &mut collected_paths);

    if collected_paths.is_empty() {
        print_err(format!(
            "{}",
            "ملف nazm.yaml يجب أن يحتوي على خاصية `المسارات` مع مسار ملف واحد على الأقل".bold(),
        ));
        exit(1)
    };

    // println!("الاسم: {}", الاسم.unwrap_or("".to_string()));
    // println!("الإصدار: {}", الإصدار.unwrap_or("".to_string()));
    // println!("المسارات:");
    // for path in &collected_paths {
    //     println!("\t{}", path);
    // }

    collected_paths
}

fn main() {
    // RTL printing
    let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
    let output = &output.stdout[1..output.stdout.len() - 1];
    io::stdout().write_all(output).unwrap();
    io::stderr().write_all(output).unwrap();

    let files_paths = get_file_paths();
    let mut id_pool = DataPool::new();
    let mut str_pool = DataPool::new();
    let mut packages = HashMap::new();
    let mut parsed_files = vec![];
    let mut packages_to_parsed_files = vec![];
    let mut diagnostics: Vec<String> = vec![];
    let mut fail_after_parsing = false;
    // let mut ast_items_counter = ASTItemsCounter::default();
    // let mut items_to_mods: HashMap<PoolIdx, Vec<_>> = HashMap::new();

    // Register the main fn id to index 0 and the implicit lambda param name to index 1
    id_pool.get("البداية");
    id_pool.get("س");

    files_paths.into_iter().for_each(|file_path| {
        let mut package_path = file_path
            .split_terminator('/')
            .map(|s| id_pool.get(s))
            .collect::<ThinVec<_>>();

        package_path.pop(); // remove the actual file

        let package_idx = packages.len();
        let package_idx = *packages.entry(package_path).or_insert(package_idx);

        let path = format!("{file_path}.نظم");
        let Ok(file_content) = fs::read_to_string(&path) else {
            panic::set_hook(Box::new(|_| {}));
            print_err(format!(
                "{} {}{}",
                "لا يمكن قراءة الملف".bold(),
                path.bright_red().bold(),
                " أو أنه غير موجود".bold()
            ));
            panic!()
        };

        let (tokens, lines, lexer_errors) =
            LexerIter::new(&file_content, &mut id_pool, &mut str_pool).collect_all();

        let ast = parse(tokens, &path, &file_content, &lines, lexer_errors);

        match ast {
            Ok(ast) => {
                if package_idx >= packages_to_parsed_files.len() {
                    packages_to_parsed_files.resize(package_idx + 1, vec![]);
                }

                let file_idx = parsed_files.len();
                parsed_files.push(nazmc_resolve::ParsedFile { path, lines, ast });
                packages_to_parsed_files[package_idx].push(file_idx);
            }
            Err(d) => {
                diagnostics.push(d);
                fail_after_parsing = true;
            }
        }
    });

    if fail_after_parsing {
        let last_idx = diagnostics.len() - 1;
        for (i, d) in diagnostics.iter().enumerate() {
            eprint!("{d}");
            if i != last_idx {
                eprintln!();
            }
        }
        exit(1)
    }

    let id_pool = id_pool.build();
    let str_pool = str_pool.build();

    let mut packages_names = ThinVec::with_capacity(packages.len());
    for (pkg, idx) in &packages {
        if *idx >= packages_names.len() {
            for _ in packages_names.len()..=*idx {
                packages_names.push(ThinVec::default());
            }
        }
        packages_names[*idx] = pkg.clone();
    }

    let resolver = nazmc_resolve::NameResolver::new(
        &id_pool,
        &packages,
        &packages_names,
        &packages_to_parsed_files,
        &parsed_files,
    );

    resolver.resolve();

    // let (file_path, file_content) = cli::read_file();

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
