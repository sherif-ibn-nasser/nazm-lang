mod cli;

use cli::print_err;
use nazmc_data_pool::DataPool;
use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::eprint_diagnostics;
use nazmc_diagnostics::span::Span;
use nazmc_diagnostics::CodeWindow;
use nazmc_diagnostics::Diagnostic;
use nazmc_parser::parse;
use nazmc_parser::parse_methods::ParseResult;
use nazmc_parser::syntax;
use nazmc_parser::syntax::FileItem;
use nazmc_parser::syntax::Id;
use nazmc_parser::syntax::Item;
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

#[derive(Default)]
struct ASTItemsCounter {
    unit_structs: usize,
    tuple_structs: usize,
    fields_structs: usize,
    fns: usize,
}

enum VisModifier {
    Default = 0,
    Pub = 1,
    Priv = 2,
}

#[derive(Clone)]
struct ItemMapToMod {
    mod_idx: usize,
    file_idx: usize,
    idx_in_file: usize,
}

// struct ModItemsEncoderDecoder;

// impl ModItemsEncoderDecoder {
//     pub fn encode(kind: ItemKind, vis: VisModifier, idx: usize) -> u64 {
//         (kind as u64) << 62 | (vis as u64) << 60 | (idx as u64)
//     }
// }

#[inline]
fn get_file_item(file_item: &FileItem) -> (&Item, VisModifier) {
    match file_item {
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
    }
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
    let mut ast_items_counter = ASTItemsCounter::default();
    let mut files = vec![]; // path, lines and the parse syntax tree
    let mut mods = HashMap::new();
    let mut items_to_mods: HashMap<PoolIdx, Vec<_>> = HashMap::new();

    files_paths
        .into_iter()
        // Lex and parse
        .map(|file_path| {
            let mut mod_path = file_path
                .split_terminator('/')
                .map(|s| id_pool.get(s))
                .collect::<Vec<_>>();

            mod_path.pop(); // remove the actual file

            let mod_idx = mods.len();
            let mod_idx = *mods.entry(mod_path).or_insert(mod_idx);

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

                (mod_idx, file_path, file_lines, file)
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        // Wait for thread to finish
        .map(|jh| jh.join())
        .collect::<Vec<_>>()
        .into_iter()
        // Map files, mods and items in each mod and encoding them
        .for_each(|r| {
            let Ok((mod_idx, file_path, file_lines, file)) = r else {
                exit(1)
            };

            let file_idx = files.len();

            file.content
                .items
                .iter()
                .enumerate()
                .for_each(|(idx_in_file, item)| {
                    let Ok(item) = item else {
                        unreachable!();
                    };

                    let item = match item {
                        syntax::FileItem::WithVisModifier(item_with_vis) => {
                            let Ok(item) = &item_with_vis.item else {
                                unreachable!()
                            };

                            item
                        }
                        syntax::FileItem::WithoutModifier(item) => item,
                        _ => return, // i.e. continue
                    };

                    let Ok(name) = (match item {
                        syntax::Item::Struct(s) => &s.name,
                        syntax::Item::Fn(f) => &f.name,
                    }) else {
                        unreachable!()
                    };

                    let name_pool_idx = id_pool.get(&name.data.val);

                    items_to_mods
                        .entry(name_pool_idx)
                        .or_default()
                        .push(ItemMapToMod {
                            mod_idx,
                            file_idx,
                            idx_in_file,
                        });
                });

            files.push((file_path, file_lines, file));
        });

    let id_pool = id_pool.build();

    let mut diagnostics = vec![];

    // Check duplicate items across mod files
    // FIXME: Could we multithread that?!
    for (item_name_idx, item_to_mods) in items_to_mods.iter_mut() {
        item_to_mods.sort_by(|a, b| a.mod_idx.cmp(&b.mod_idx));

        let name = &id_pool[*item_name_idx];

        item_to_mods
            .chunk_by(|a, b| a.mod_idx == b.mod_idx)
            .for_each(|slice| {
                let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الحزمة", name);

                let mut diagnostic = Diagnostic::error(msg, vec![]);

                let mut occurunces = 1;

                let mut slice = slice.to_vec();

                slice.sort_by(|a, b| a.file_idx.cmp(&b.file_idx));

                slice
                    .chunk_by(|a, b| a.file_idx == b.file_idx)
                    .for_each(|slice2| {
                        let (file_path, file_lines, file) = &files[slice2[0].file_idx];

                        let get_item_name_span_by_idx = |idx: usize| {
                            let Ok(item_syntax_tree) = &file.content.items[slice2[idx].idx_in_file]
                            else {
                                unreachable!()
                            };

                            let item = match item_syntax_tree {
                                syntax::FileItem::WithVisModifier(item_with_vis) => {
                                    let Ok(item) = &item_with_vis.item else {
                                        unreachable!()
                                    };

                                    item
                                }
                                syntax::FileItem::WithoutModifier(item) => item,
                            };

                            let Ok(name) = (match item {
                                syntax::Item::Struct(s) => &s.name,
                                syntax::Item::Fn(f) => &f.name,
                            }) else {
                                unreachable!()
                            };
                            name.span
                        };

                        let mut code_window = CodeWindow::new(
                            file_path,
                            file_lines,
                            get_item_name_span_by_idx(0).start,
                        );

                        for (i, _) in slice2.iter().enumerate() {
                            let span = get_item_name_span_by_idx(i);
                            let occurence_str = match occurunces {
                                1 => "هنا تم العثور على أول عنصر بهذا الاسم".to_string(),
                                2 => "هنا تم العثور على نفس الاسم للمرة الثانية".to_string(),
                                3 => "هنا تم العثور على نفس الاسم للمرة الثالثة".to_string(),
                                4 => "هنا تم العثور على نفس الاسم للمرة الرابعة".to_string(),
                                5 => "هنا تم العثور على نفس الاسم للمرة الخامسة".to_string(),
                                6 => "هنا تم العثور على نفس الاسم للمرة السادسة".to_string(),
                                7 => "هنا تم العثور على نفس الاسم للمرة السابعة".to_string(),
                                8 => "هنا تم العثور على نفس الاسم للمرة الثامنة".to_string(),
                                9 => "هنا تم العثور على نفس الاسم للمرة التاسعة".to_string(),
                                10 => "هنا تم العثور على نفس الاسم للمرة العاشرة".to_string(),
                                o => format!("هنا تم العثور على نفس الاسم للمرة {}", o),
                            };
                            if occurunces == 1 {
                                code_window.mark_error(span, vec![occurence_str]);
                            } else {
                                code_window.mark_secondary(span, vec![occurence_str]);
                            }
                            occurunces += 1;
                        }

                        diagnostic.push_code_window(code_window);
                    });
                diagnostics.push(diagnostic);
            });
    }

    if !diagnostics.is_empty() {
        eprint_diagnostics(diagnostics);
        exit(1)
    }

    // FIXME: Optimize
    let mut mods_vec = Vec::with_capacity(mods.len());
    for (mod_path, idx) in mods {
        if idx >= mods_vec.len() {
            for _ in mods_vec.len()..=idx {
                mods_vec.push(vec![]);
            }
        }
        mods_vec[idx] = mod_path;
    }

    let mut fns = vec![];

    for (item_name_idx, item_to_mods) in items_to_mods {
        for item_to_mod in item_to_mods {
            let (file_path, file_lines, file) = &files[item_to_mod.file_idx];

            let Ok(item) = &file.content.items[item_to_mod.idx_in_file] else {
                unreachable!()
            };

            let (item, vis) = get_file_item(item);

            match item {
                syntax::Item::Struct(s) => {
                    let Ok(name) = &s.name else { unreachable!() };
                    let name = nazmc_ast::ASTId {
                        span: name.span,
                        id: item_name_idx,
                    };
                    todo!()
                }
                syntax::Item::Fn(f) => {
                    let Ok(name) = &f.name else { unreachable!() };
                    let name = nazmc_ast::ASTId {
                        span: name.span,
                        id: item_name_idx,
                    };
                    todo!();
                    fns.push(nazmc_ast::Fn {
                        mod_index: item_to_mod.mod_idx,
                        name,
                        params: todo!(),
                        return_ty: todo!(),
                        scope: todo!(),
                    });
                }
            }
        }
    }

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
