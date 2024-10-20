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
            .collect::<Vec<_>>();

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

    nazmc_resolve::check_conflicts(packages_to_parsed_files, parsed_files, &id_pool);

    // let jhs = files_paths
    //     .into_iter()
    //     .map(|file_path| {
    //         let mut mod_path = file_path
    //             .split_terminator('/')
    //             .map(|s| id_pool.get(s))
    //             .collect::<Vec<_>>();

    //         mod_path.pop(); // remove the actual file

    //         let mod_idx = mods.len();
    //         let mod_idx = *mods.entry(mod_path).or_insert(mod_idx);

    //         std::thread::spawn(move || {})
    //     })
    //     .collect::<Vec<_>>();

    // let mut parsed_files = ThinVec::with_capacity(jhs.len());
    // let mut parsed_mods: ThinVec<ThinVec<_>> = ThinVec::with_capacity(mods.len());
    // for _ in 0..mods.len() {
    //     parsed_mods.push(ThinVec::new());
    // }

    // // Wait for threads to finish
    // jhs.into_iter().for_each(|jh| {
    //     let r = jh.join();
    //     match r {
    //         Ok((mod_idx, path, lines, Ok(ast))) => {
    //             let file_idx = parsed_files.len();
    //             parsed_files.push(ParsedFile { path, lines, ast });
    //             parsed_mods[mod_idx].push(file_idx);
    //         }
    //         Ok((.., Err(dd))) => {
    //             diagnostics.push(dd);
    //             fail_after_parsing = true
    //         }
    //         Err(_) => fail_after_parsing = true,
    //     }
    // });

    // for file_path in files_paths {
    //     let mut mod_path = file_path
    //         .split_terminator('/')
    //         .map(|s| id_pool.get(s))
    //         .collect::<Vec<_>>();

    //     mod_path.pop(); // remove the actual file

    //     let mod_idx = mods.len();
    //     let mod_idx = *mods.entry(mod_path).or_insert(mod_idx);

    //     let file_path = format!("{file_path}.نظم");
    //     let Ok(file_content) = fs::read_to_string(&file_path) else {
    //         panic::set_hook(Box::new(|_| {}));
    //         print_err(format!(
    //             "{} {}{}",
    //             "لا يمكن قراءة الملف".bold(),
    //             file_path.bright_red().bold(),
    //             " أو أنه غير موجود".bold()
    //         ));
    //         panic!()
    //     };

    //     let (tokens, file_lines, lexer_errors) =
    //         LexerIter::new(&file_content, &mut id_pool, &mut str_pool).collect_all();

    //     std::thread::spawn(move || {
    //         let file = parse(&file_path, tokens, &file_lines, lexer_errors);

    //         (mod_idx, file_path, file_lines, file)
    //     });
    // }

    // let iter = files_paths
    //     .into_iter()
    //     // Lex and parse
    //     .map(|file_path| {
    //         let mut mod_path = file_path
    //             .split_terminator('/')
    //             .map(|s| id_pool.get(s))
    //             .collect::<Vec<_>>();

    //         mod_path.pop(); // remove the actual file

    //         let mod_idx = mods.len();
    //         let mod_idx = *mods.entry(mod_path).or_insert(mod_idx);

    //         let file_path = format!("{file_path}.نظم");
    //         let Ok(file_content) = fs::read_to_string(&file_path) else {
    //             panic::set_hook(Box::new(|_| {}));
    //             print_err(format!(
    //                 "{} {}{}",
    //                 "لا يمكن قراءة الملف".bold(),
    //                 file_path.bright_red().bold(),
    //                 " أو أنه غير موجود".bold()
    //             ));
    //             panic!()
    //         };

    //         let (tokens, file_lines, lexer_errors) =
    //             LexerIter::new(&file_content, &mut id_pool, &mut str_pool).collect_all();

    //         std::thread::spawn(move || {
    //             let file = parse(&file_path, tokens, &file_lines, lexer_errors);

    //             (mod_idx, file_path, file_lines, file)
    //         })
    //     })
    //     .collect::<Vec<_>>()
    //     .into_iter()
    //     // Wait for thread to finish
    //     .map(|jh| jh.join())
    //     .collect::<Vec<_>>()
    //     .into_iter();

    // let id_pool = id_pool.build();

    // // Map files, mods and items in each mod and encoding them
    // iter.for_each(|r| {
    //     let Ok((mod_idx, file_path, file_lines, file)) = r else {
    //         exit(1)
    //     };

    //     let file_idx = files.len();

    //     file.content
    //         .items
    //         .iter()
    //         .enumerate()
    //         .for_each(|(idx_in_file, item)| {
    //             let Ok(item) = item else {
    //                 unreachable!();
    //             };

    //             let item = match item {
    //                 syntax::FileItem::WithVisModifier(item_with_vis) => {
    //                     let Ok(item) = &item_with_vis.item else {
    //                         unreachable!()
    //                     };

    //                     item
    //                 }
    //                 syntax::FileItem::WithoutModifier(item) => item,
    //                 _ => return, // i.e. continue
    //             };

    //             let Ok(name) = (match item {
    //                 syntax::Item::Struct(s) => &s.name,
    //                 syntax::Item::Fn(f) => &f.name,
    //             }) else {
    //                 unreachable!()
    //             };

    //             let name_pool_idx = name.data.val;

    //             items_to_mods
    //                 .entry(name_pool_idx)
    //                 .or_default()
    //                 .push(ItemMapToMod {
    //                     mod_idx,
    //                     file_idx,
    //                     idx_in_file,
    //                 });
    //         });

    //     files.push((file_path, file_lines, file, mod_idx));
    // });

    // let mut diagnostics = vec![];

    // // Check duplicate items across mod files
    // // FIXME: Could we multithread that?!
    // for (item_name_idx, item_to_mods) in items_to_mods.iter_mut() {
    //     item_to_mods.sort_by(|a, b| a.mod_idx.cmp(&b.mod_idx));

    //     let name = &id_pool[*item_name_idx];

    //     item_to_mods
    //         .chunk_by(|a, b| a.mod_idx == b.mod_idx)
    //         .for_each(|slice| {
    //             if slice.len() == 1 {
    //                 return;
    //             }
    //             let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الحزمة", name);

    //             let mut diagnostic = Diagnostic::error(msg, vec![]);

    //             let mut occurunces = 1;

    //             let mut slice = slice.to_vec();

    //             slice.sort_by(|a, b| a.file_idx.cmp(&b.file_idx));

    //             slice
    //                 .chunk_by(|a, b| a.file_idx == b.file_idx)
    //                 .for_each(|slice2| {
    //                     let (file_path, file_lines, file, file_mod_idx) =
    //                         &files[slice2[0].file_idx];

    //                     let get_item_name_span_by_idx = |idx: usize| {
    //                         let Ok(item_syntax_tree) = &file.content.items[slice2[idx].idx_in_file]
    //                         else {
    //                             unreachable!()
    //                         };

    //                         let item = match item_syntax_tree {
    //                             syntax::FileItem::WithVisModifier(item_with_vis) => {
    //                                 let Ok(item) = &item_with_vis.item else {
    //                                     unreachable!()
    //                                 };

    //                                 item
    //                             }
    //                             syntax::FileItem::WithoutModifier(item) => item,
    //                         };

    //                         let Ok(name) = (match item {
    //                             syntax::Item::Struct(s) => &s.name,
    //                             syntax::Item::Fn(f) => &f.name,
    //                         }) else {
    //                             unreachable!()
    //                         };
    //                         name.span
    //                     };

    //                     let mut code_window = CodeWindow::new(
    //                         file_path,
    //                         file_lines,
    //                         get_item_name_span_by_idx(0).start,
    //                     );

    //                     for (i, _) in slice2.iter().enumerate() {
    //                         let span = get_item_name_span_by_idx(i);
    //                         let occurence_str = match occurunces {
    //                             1 => "هنا تم العثور على أول عنصر بهذا الاسم".to_string(),
    //                             2 => "هنا تم العثور على نفس الاسم للمرة الثانية".to_string(),
    //                             3 => "هنا تم العثور على نفس الاسم للمرة الثالثة".to_string(),
    //                             4 => "هنا تم العثور على نفس الاسم للمرة الرابعة".to_string(),
    //                             5 => "هنا تم العثور على نفس الاسم للمرة الخامسة".to_string(),
    //                             6 => "هنا تم العثور على نفس الاسم للمرة السادسة".to_string(),
    //                             7 => "هنا تم العثور على نفس الاسم للمرة السابعة".to_string(),
    //                             8 => "هنا تم العثور على نفس الاسم للمرة الثامنة".to_string(),
    //                             9 => "هنا تم العثور على نفس الاسم للمرة التاسعة".to_string(),
    //                             10 => "هنا تم العثور على نفس الاسم للمرة العاشرة".to_string(),
    //                             o => format!("هنا تم العثور على نفس الاسم للمرة {}", o),
    //                         };
    //                         if occurunces == 1 {
    //                             code_window.mark_error(span, vec![occurence_str]);
    //                         } else {
    //                             code_window.mark_secondary(span, vec![occurence_str]);
    //                         }
    //                         occurunces += 1;
    //                     }

    //                     diagnostic.push_code_window(code_window);
    //                 });
    //             diagnostics.push(diagnostic);
    //         });
    // }

    // let mut unresolved_imports = vec![];
    // let mut resolved_imports = vec![];
    // let mut resolved_star_imports = vec![];

    // // Resolve imports
    // for (file_idx, (file_path, file_lines, file, file_mod_idx)) in files.iter().enumerate() {
    //     for import_stm in &file.imports {
    //         let mut mod_path = vec![];
    //         let mut path_spans = vec![];
    //         let mut import_all = false;

    //         // Init
    //         if let Ok(id) = &import_stm.top {
    //             mod_path.push(id.data.val);
    //             path_spans.push(id.span);
    //         } else {
    //             unreachable!()
    //         };

    //         if let Ok(s) = &import_stm.sec {
    //             match s.seg.as_ref().unwrap() {
    //                 syntax::PathSegInImportStm::Id(id) => {
    //                     mod_path.push(id.data.val);
    //                     path_spans.push(id.span);
    //                 }
    //                 syntax::PathSegInImportStm::Star(_) => import_all = true,
    //             }
    //         } else {
    //             unreachable!()
    //         };

    //         for s in &import_stm.segs {
    //             match s.seg.as_ref().unwrap() {
    //                 syntax::PathSegInImportStm::Id(id) => {
    //                     mod_path.push(id.data.val);
    //                     path_spans.push(id.span);
    //                 }
    //                 syntax::PathSegInImportStm::Star(_) => import_all = true,
    //             }
    //         }

    //         // Resolve
    //         if import_all {
    //             match mods.get(&mod_path) {
    //                 Some(mod_idx) => {
    //                     // The mod is found with mod_idx
    //                     resolved_star_imports.push(ResolvedStarImport {
    //                         file_idx,
    //                         mod_idx: *mod_idx,
    //                     });
    //                 }
    //                 None => {
    //                     while let Some(first_invalid_seg) = mod_path.pop() {
    //                         let first_invalid_seg_span = path_spans.pop().unwrap();

    //                         if mods.contains_key(&mod_path) {
    //                             unresolved_imports.push(UnresolvedImport {
    //                                 first_invalid_seg,
    //                                 first_invalid_seg_span,
    //                                 file_idx,
    //                             });
    //                             break;
    //                         }
    //                     }
    //                 }
    //             }
    //         } else {
    //             let item_id = mod_path.pop().unwrap();
    //             let item_span = path_spans.pop().unwrap();
    //             match mods.get(&mod_path) {
    //                 Some(mod_idx) => {
    //                     // The mod is found with mod_idx
    //                     let item_to_mods = match items_to_mods.get(&item_id) {
    //                         Some(item_to_mods) => item_to_mods,
    //                         None => {
    //                             unresolved_imports.push(UnresolvedImport {
    //                                 first_invalid_seg: item_id,
    //                                 first_invalid_seg_span: item_span,
    //                                 file_idx,
    //                             });
    //                             continue;
    //                         }
    //                     };

    //                     match item_to_mods.binary_search_by(|probe| probe.mod_idx.cmp(mod_idx)) {
    //                         Ok(item_to_mod_idx) => {
    //                             let item_to_mod = &item_to_mods[item_to_mod_idx];

    //                             if item_to_mod.mod_idx == *file_mod_idx {
    //                                 resolved_imports.push(ResolvedImport {
    //                                     file_idx,
    //                                     item_id,
    //                                     item_to_mod_idx,
    //                                 });
    //                             } else {
    //                                 // The import stm and the item are not in the same mods
    //                                 let (_item, vis) = get_file_item(
    //                                     files[item_to_mod.file_idx].2.content.items
    //                                         [item_to_mod.idx_in_file]
    //                                         .as_ref()
    //                                         .unwrap(),
    //                                 );
    //                                 // Check resolved item visibility
    //                                 if let VisModifier::Default = vis {
    //                                     // TODO
    //                                 }
    //                             }
    //                         }
    //                         Err(_) => {
    //                             unresolved_imports.push(UnresolvedImport {
    //                                 first_invalid_seg: item_id,
    //                                 first_invalid_seg_span: item_span,
    //                                 file_idx,
    //                             });
    //                         }
    //                     }
    //                 }
    //                 None => {
    //                     while let Some(first_invalid_seg) = mod_path.pop() {
    //                         let first_invalid_seg_span = path_spans.pop().unwrap();

    //                         if mods.contains_key(&mod_path) {
    //                             unresolved_imports.push(UnresolvedImport {
    //                                 first_invalid_seg,
    //                                 first_invalid_seg_span,
    //                                 file_idx,
    //                             });
    //                             break;
    //                         }
    //                     }
    //                 }
    //             };
    //         }
    //     }
    // }

    // if !unresolved_imports.is_empty() {
    //     for unresolved_import in unresolved_imports {
    //         let name = &id_pool[unresolved_import.first_invalid_seg];
    //         let msg = format!("لم يتم العثور على الاسم `{}` في المسار", name);
    //         let (file_path, file_lines, ..) = &files[unresolved_import.file_idx];
    //         let mut code_window = CodeWindow::new(
    //             file_path,
    //             file_lines,
    //             unresolved_import.first_invalid_seg_span.start,
    //         );
    //         code_window.mark_error(
    //             unresolved_import.first_invalid_seg_span,
    //             vec!["هذا الاسم غير موجود داخل المسار المحدد".to_string()],
    //         );
    //         let diagnostic = Diagnostic::error(msg, vec![code_window]);
    //         diagnostics.push(diagnostic);
    //     }
    // }

    // if !diagnostics.is_empty() {
    //     eprint_diagnostics(diagnostics);
    //     exit(1)
    // }

    // let mut fns = vec![];

    // for (item_name_idx, item_to_mods) in items_to_mods {
    //     for item_to_mod in item_to_mods {
    //         let (file_path, file_lines, file, file_mod_idx) = &files[item_to_mod.file_idx];

    //         let Ok(item) = &file.content.items[item_to_mod.idx_in_file] else {
    //             unreachable!()
    //         };

    //         let (item, vis) = get_file_item(item);

    //         match item {
    //             syntax::Item::Struct(s) => {
    //                 let Ok(name) = &s.name else { unreachable!() };
    //                 let name = nazmc_ast::ASTId {
    //                     span: name.span,
    //                     id: item_name_idx,
    //                 };
    //                 todo!()
    //             }
    //             syntax::Item::Fn(f) => {
    //                 let Ok(name) = &f.name else { unreachable!() };
    //                 let name = nazmc_ast::ASTId {
    //                     span: name.span,
    //                     id: item_name_idx,
    //                 };
    //                 todo!();
    //                 fns.push(nazmc_ast::Fn {
    //                     mod_index: item_to_mod.mod_idx,
    //                     name,
    //                     params: todo!(),
    //                     return_ty: todo!(),
    //                     scope: todo!(),
    //                 });
    //             }
    //         }
    //     }
    // }

    // FIXME: Optimize
    // let mut mods_vec = Vec::with_capacity(mods.len());
    // for (mod_path, idx) in mods {
    //     if idx >= mods_vec.len() {
    //         for _ in mods_vec.len()..=idx {
    //             mods_vec.push(vec![]);
    //         }
    //     }
    //     mods_vec[idx] = mod_path;
    // }

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
