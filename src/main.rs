mod cli;

use bpaf::doc;
use cli::{format_err, print_err};
use nazmc_data_pool::DataPool;
use nazmc_data_pool::Init;
use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::span::Span;
use nazmc_diagnostics::CodeWindow;
use nazmc_diagnostics::Diagnostic;
use nazmc_diagnostics::FileDiagnostics;
use nazmc_parser::parse;
use nazmc_parser::parse_methods::ParseResult;
use nazmc_parser::syntax;
use nazmc_parser::syntax::Id;
use nazmc_parser::syntax::IdToken;
use nazmc_parser::syntax::Terminal;
use owo_colors::{OwoColorize, XtermColors};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::borrow::Borrow;
use std::{
    arch::x86_64::__m128i,
    cell::RefCell,
    collections::HashMap,
    fmt::format,
    fs,
    io::{self, stderr, Write},
    panic::{self, panic_any},
    path::{Path, PathBuf},
    process::{abort, exit, Command, ExitCode, Termination},
    sync::{Arc, Mutex},
};

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
    io::stdout()
        .write_all(&output.stdout[1..output.stdout.len() - 1])
        .unwrap();

    let files_paths = get_file_paths();
    let mut id_pool = DataPool::new();
    let mut str_pool = DataPool::new();

    #[derive(Default)]
    struct ASTItemsCounter {
        unit_structs: usize,
        tuple_structs: usize,
        fields_structs: usize,
        fns: usize,
    }

    let mut ast_items_counter = ASTItemsCounter::default();

    files_paths
        .into_iter()
        .map(|file_path| {
            let mod_path = file_path
                .split_terminator('/')
                .map(|s| id_pool.get(s))
                .collect::<Vec<_>>();

            std::thread::spawn(move || {
                let file_path = format!("{file_path}.نظم");
                let Ok(file_content) = fs::read_to_string(&file_path) else {
                    panic::set_hook(Box::new(|_| {}));
                    print_err(format!(
                        "{} {}{}",
                        "لا يمكن قراءة الملف".bold(),
                        file_path.bright_red().bold(),
                        " أو أنه غير موجود".bold()
                    ));
                    panic!()
                };

                let (file, file_lines) = parse(&file_path, &file_content);

                (mod_path, file_path, file_lines, file)
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|jh| jh.join())
        .collect::<Vec<_>>()
        .into_iter()
        .map(|r| {
            let Ok((mod_path, file_path, file_lines, file)) = r else {
                exit(1)
            };

            let mut file_diagnostics = FileDiagnostics::new(&file_path, &file_lines);

            let mut mod_items_map = HashMap::new();

            let _ = &file
                .content
                .items
                .iter()
                .map(|item| {
                    let Ok(item) = item else {
                        unreachable!();
                    };

                    enum VisModifier {
                        Default = 0,
                        Pub = 1,
                        Priv = 2,
                    }

                    enum ItemKind {
                        UnitSruct = 0,
                        TupleSruct = 1,
                        FieldsSruct = 2,
                        Fn = 3,
                    }

                    struct ModItemsEncoderDecoder;

                    impl ModItemsEncoderDecoder {
                        pub fn encode(kind: ItemKind, vis: VisModifier, idx: usize) -> u64 {
                            (kind as u64) << 62 | (vis as u64) << 60 | (idx as u64)
                        }
                    }

                    fn get_item_kind_and_name(item: &syntax::Item) -> (ItemKind, &ParseResult<Id>) {
                        match item {
                            syntax::Item::Struct(s) => (
                                match &s.kind {
                                    Ok(syntax::StructKind::Unit(_)) => ItemKind::UnitSruct,
                                    Ok(syntax::StructKind::Tuple(_)) => ItemKind::TupleSruct,
                                    Ok(syntax::StructKind::Fields(_)) => ItemKind::FieldsSruct,
                                    _ => unreachable!(),
                                },
                                &s.name,
                            ),
                            syntax::Item::Fn(f) => (ItemKind::Fn, &f.name),
                        }
                    }

                    let (item, vis) = match item {
                        syntax::FileItem::WithVisModifier(item_with_vis) => {
                            let Ok(item) = &item_with_vis.item else {
                                unreachable!()
                            };

                            (
                                item,
                                match item_with_vis.visibility.data {
                                    syntax::VisModifierToken::Public => VisModifier::Pub,
                                    syntax::VisModifierToken::Private => VisModifier::Priv,
                                },
                            )
                        }
                        syntax::FileItem::WithoutModifier(item) => (item, VisModifier::Default),
                    };

                    let (kind, name) = get_item_kind_and_name(item);

                    let Ok(name) = name else { unreachable!() };

                    let name_pool_idx = id_pool.get(&name.data.val);

                    let ast_counter = match kind {
                        ItemKind::UnitSruct => &mut ast_items_counter.unit_structs,
                        ItemKind::TupleSruct => &mut ast_items_counter.tuple_structs,
                        ItemKind::FieldsSruct => &mut ast_items_counter.fields_structs,
                        ItemKind::Fn => &mut ast_items_counter.fns,
                    };

                    let idx = *ast_counter;

                    *ast_counter += 1;

                    match mod_items_map.get(&name_pool_idx) {
                        None => {
                            mod_items_map.insert(
                                name_pool_idx,
                                (ModItemsEncoderDecoder::encode(kind, vis, idx), name.span),
                            );
                        }
                        Some((_, found_span)) => {
                            let cursor = found_span.start;
                            let mut code_window = CodeWindow::new(cursor);

                            code_window
                                .mark_secondary(*found_span, vec!["هنا أول عنصر".to_string()]);
                            code_window.mark_secondary(
                                name.span,
                                vec!["هنا ثاني عنصر بنفس الاسم".to_string()],
                            );

                            let msg =
                                format!("يُوجد عنصران بنفس الاسم `{}` في نفس الملف", name.data.val);

                            file_diagnostics.push(Diagnostic::error(msg, Some(code_window)));
                        }
                    }
                })
                .collect::<Vec<()>>();

            if file_diagnostics.has_disgnostics() {
                eprintln!("{}", file_diagnostics);
                Err(())
            } else {
                Ok(())
            }
        })
        .collect::<Vec<_>>()
        .iter()
        .for_each(|r| {
            if r.is_err() {
                exit(1)
            }
        });

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
