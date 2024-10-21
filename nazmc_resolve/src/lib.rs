use nazmc_data_pool::{Built, DataPool, PoolIdx};
use nazmc_diagnostics::{eprint_diagnostics, span::Span, CodeWindow, Diagnostic};
use std::{collections::HashMap, process::exit};
use thin_vec::ThinVec;

#[derive(Clone)]
pub struct ParsedFile {
    pub path: String,
    pub lines: Vec<String>,
    pub ast: nazmc_ast::File,
}

#[derive(Default)]
struct ASTItemsCounter {
    unit_structs: usize,
    tuple_structs: usize,
    fields_structs: usize,
    fns: usize,
}

#[derive(Clone)]
struct ItemMapToMod {
    mod_idx: usize,
    file_idx: usize,
    idx_in_file: usize,
}

struct ResolvedStarImport {
    file_idx: usize,
    mod_idx: usize,
}

#[derive(Clone, Copy)]
pub struct ItemInFile {
    pub file_idx: usize,
    pub item_idx: usize,
}

#[derive(Clone, Copy)]
struct ResolvedImport {
    /// The pkg idx of the resolved item
    pkg_idx: usize,
    /// The resolved item
    item: ItemInFile,
    /// The alias of the resolved item
    alias: nazmc_ast::ASTId,
}

pub fn check_conflicts(
    packages_to_parsed_files: &[Vec<usize>],
    parsed_files: &[ParsedFile],
    id_pool: &DataPool<Built>,
) -> Vec<HashMap<PoolIdx, ItemInFile>> {
    // The index of this vector is the package index
    // Each package has a map of ids to their occurrence in the package (the file index and the item index in that file)
    let mut packages_to_items: Vec<HashMap<PoolIdx, ItemInFile>> = vec![];

    let mut conflicts: HashMap<(usize, PoolIdx), HashMap<usize, Vec<Span>>> = HashMap::new();
    //                          ^^^^^  ^^^^^^^           ^^^^^  ^^^^^^^^^
    //                          |      |                 |      |
    //                          |      |                 |      span occurrences in the file
    //                          |      |                 parsed file idx
    //                          |      conflicting name
    //                          package idx

    for (pkg_idx, parsed_files_in_package) in packages_to_parsed_files.iter().enumerate() {
        packages_to_items.push(HashMap::default());

        let items_in_package = &mut packages_to_items[pkg_idx];

        for parsed_file_idx in parsed_files_in_package {
            let parsed_file = &parsed_files[*parsed_file_idx];

            for (item_idx, item) in parsed_file.ast.items.iter().enumerate() {
                match items_in_package.get(&item.name.id) {
                    Some(first_occurrence) => {
                        conflicts
                            .entry((pkg_idx, item.name.id))
                            .or_default()
                            .entry(*parsed_file_idx)
                            .or_insert_with(|| {
                                let first_occurrence_span = parsed_files[first_occurrence.file_idx]
                                    .ast
                                    .items[first_occurrence.item_idx]
                                    .name
                                    .span;

                                vec![first_occurrence_span]
                            })
                            .push(item.name.span);
                    }
                    None => {
                        items_in_package.insert(
                            item.name.id,
                            ItemInFile {
                                file_idx: *parsed_file_idx,
                                item_idx,
                            },
                        );
                    }
                }
            }
        }
    }

    let mut diagnostics = vec![];

    for ((_pkg_idx, conflicting_name), name_conflicts_in_single_package) in conflicts {
        let name = &id_pool[conflicting_name];
        let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الحزمة", name);
        let mut diagnostic = Diagnostic::error(msg, vec![]);
        let mut occurrences = 1;

        for (file_idx, mut spans) in name_conflicts_in_single_package {
            let file = &parsed_files[file_idx];

            let mut code_window = CodeWindow::new(&file.path, &file.lines, spans[0].start);

            nazmc_diagnostics::span::sort_spans(&mut spans);

            for span in spans {
                let occurrence_str = match occurrences {
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

                if occurrences == 1 {
                    code_window.mark_error(span, vec![occurrence_str]);
                } else {
                    code_window.mark_secondary(span, vec![occurrence_str]);
                }

                occurrences += 1;
            }

            diagnostic.push_code_window(code_window);
        }

        diagnostics.push(diagnostic);
    }

    if !diagnostics.is_empty() {
        eprint_diagnostics(diagnostics);
        exit(1)
    }

    packages_to_items
}

pub fn resolve_imports<'a, 'b>(
    id_pool: &DataPool<Built>,
    packages_names: &[ThinVec<PoolIdx>],
    packages: &HashMap<ThinVec<PoolIdx>, usize>,
    packages_to_parsed_files: &[Vec<usize>],
    packages_to_items: &[HashMap<PoolIdx, ItemInFile>],
    parsed_files: &'a [ParsedFile],
) {
    let mut diagnostics = vec![];

    let unresolved_import_err = |file: &'a ParsedFile, id: PoolIdx, span: Span| {
        let name = &id_pool[id];
        let msg = format!("لم يتم العثور على الاسم `{}` في المسار", name);

        let mut code_window = CodeWindow::new(&file.path, &file.lines, span.start);

        code_window.mark_error(
            span,
            vec!["هذا الاسم غير موجود داخل المسار المحدد".to_string()],
        );

        let mut diagnostic = Diagnostic::error(msg, vec![code_window]);

        let mut possible_paths = vec![];

        for (pkg_idx, pkg_to_items) in packages_to_items.iter().enumerate() {
            if let Some(found_item) = pkg_to_items.get(&id) {
                let item_file = &parsed_files[found_item.file_idx];
                let item_ast = &item_file.ast.items[found_item.item_idx];
                let item_span_cursor = item_ast.name.span.start;
                let item_kind_str = match item_ast.kind {
                    nazmc_ast::ItemKind::UnitStruct |
                    nazmc_ast::ItemKind::TupleStruct(_)|
                    nazmc_ast::ItemKind::FieldsStruct(_) => "الهيكل",
                    nazmc_ast::ItemKind::Fn(_) => "الدالة",
                };
                let pkg_name = &packages_names[pkg_idx].iter().map(|id| &id_pool[*id]).collect::<Vec<_>>().join("::");
                let name = &id_pool[id];
                let item_path = format!("{}:{}:{}", &item_file.path, item_span_cursor.line +1 , item_span_cursor.col +1);
                let path = format!("\t- {} {}::{} في {}", item_kind_str, pkg_name, name, item_path);

                possible_paths.push(path);
            }
        }

        if !possible_paths.is_empty(){
            let mut help = Diagnostic::help(format!("تم العثور على عناصر مشابهة بنفس الاسم في المسارات التالية:"), vec![]);

            for t in possible_paths {
                help.chain_free_text(t);
            }

            diagnostic.chain(help);
        }

        diagnostic
    };

    let pkg_path_err = |file: &'a ParsedFile,
                        mut pkg_path: ThinVec<PoolIdx>,
                        mut pkg_path_spans: ThinVec<Span>| {
        while let Some(first_invalid_seg) = pkg_path.pop() {
            let first_invalid_seg_span = pkg_path_spans.pop().unwrap();

            if packages.contains_key(&pkg_path) {
                return unresolved_import_err(&file,first_invalid_seg, first_invalid_seg_span);
            }
        }
        unreachable!()
    };

    let mut resolved_imports: Vec<Vec<_>> = vec![vec![]; packages.len()];

    for (pkg_idx, parsed_files_in_package) in packages_to_parsed_files.iter().enumerate() {
        for (i, parsed_file_idx) in parsed_files_in_package.iter().enumerate() {
            let parsed_file = &parsed_files[*parsed_file_idx];
            resolved_imports[pkg_idx].push(vec![]);
            let resolved_imports_in_file = &mut resolved_imports[pkg_idx][i];

            for (import, item_alias) in &parsed_file.ast.imports {
                match packages.get(&import.pkg_path.ids) {
                    Some(resolved_package_idx) => {
                        match packages_to_items[*resolved_package_idx].get(&import.item.id) {
                            Some(resolved_item) => {

                                let item_resolved_file = &parsed_files[resolved_item.file_idx];

                                let resolved_item_ast =
                                    &item_resolved_file.ast.items[resolved_item.item_idx];

                                if !matches!(resolved_item_ast.vis, nazmc_ast::VisModifier::Default) || pkg_idx == *resolved_package_idx {
                                    resolved_imports_in_file.push(ResolvedImport {
                                        pkg_idx: *resolved_package_idx,
                                        item: *resolved_item,
                                        alias: *item_alias,
                                    });
                                    continue;
                                }

                                let name = &id_pool[import.item.id];
                                let msg = match resolved_item_ast.kind {
                                    nazmc_ast::ItemKind::UnitStruct
                                    | nazmc_ast::ItemKind::TupleStruct(_)
                                    | nazmc_ast::ItemKind::FieldsStruct(_) => {
                                        format!("لا يمكن الوصول إلى هيكل `{}` لأنه خاص بالحزمة التابع لها" ,name)
                                    }
                                    nazmc_ast::ItemKind::Fn(_) => 
                                    format!("لا يمكن الوصول إلى دالة `{}` لأنها خاصة بالحزمة التابعة لها" ,name),
                                };
                                let mut code_window = CodeWindow::new(
                                    &parsed_file.path,
                                    &parsed_file.lines,
                                    import.item.span.start,
                                );
                                code_window.mark_error(import.item.span, vec![]);
                                let mut diagnostic =
                                    Diagnostic::error(msg, vec![code_window]);

                                let item_kind_str = match resolved_item_ast.kind {
                                    nazmc_ast::ItemKind::UnitStruct
                                    | nazmc_ast::ItemKind::TupleStruct(_)
                                    | nazmc_ast::ItemKind::FieldsStruct(_) => {
                                        "الهيكل".to_string()
                                    }
                                    nazmc_ast::ItemKind::Fn(_) => "الدالة".to_string(),
                                };
                                let help_msg =
                                    format!("تم العثور على {} هنا", item_kind_str);
                                let mut help_code_window = CodeWindow::new(
                                    &item_resolved_file.path,
                                    &item_resolved_file.lines,
                                    resolved_item_ast.name.span.start,
                                );
                                help_code_window
                                    .mark_note(resolved_item_ast.name.span, vec![]);
                                let help =
                                    Diagnostic::note(help_msg, vec![help_code_window]);
                                diagnostic.chain(help);
                                diagnostics.push(diagnostic);
                                
                            }
                            None => {
                                diagnostics.push(unresolved_import_err(
                                    &parsed_file,
                                    import.item.id,
                                    import.item.span,
                                ));
                            }
                        }
                    }
                    None => {
                        diagnostics.push(pkg_path_err(
                            &parsed_file,
                            import.pkg_path.ids.clone(),
                            import.pkg_path.spans.clone(),
                        ));
                    }
                }
            }

            for import in &parsed_file.ast.star_imports {
                match packages.get(&import.ids) {
                    Some(resolved_package_idx) => todo!(), // TODO
                    None => {
                        diagnostics.push(pkg_path_err(
                            &parsed_file,
                            import.ids.clone(),
                            import.spans.clone(),
                        ));
                    }
                }
            }
        }
    }

    let mut conflicts: HashMap<usize, HashMap<PoolIdx, Vec<Span>>> = HashMap::new();
    //                         ^^^^^          ^^^^^^^  ^^^^^^^^^
    //                         |              |        |
    //                         |              |        span occurrences in the file
    //                         |              conflicting name
    //                         file idx

    for (pkg_idx, files_in_pkg) in resolved_imports.iter().enumerate() {
        for (i, resolved_imports) in files_in_pkg.iter().enumerate() {
            let parsed_file_idx = packages_to_parsed_files[pkg_idx][i];
            let parsed_file = &parsed_files[parsed_file_idx];
            for resolved_import in resolved_imports {
                let alias = &resolved_import.alias;
                if let Some(item_with_same_id) = packages_to_items[pkg_idx].get(&alias.id) {
                    conflicts
                        .entry(parsed_file_idx)
                        .or_default()
                        .entry(alias.id)
                        .or_insert_with(|| {
                            let first_occurrence_span =
                                parsed_file.ast.items[item_with_same_id.item_idx].name.span;

                            vec![first_occurrence_span]
                        })
                        .push(alias.span);
                }
            }
        }
    }

    for (file_idx, name_conflicts_in_single_file) in conflicts {
        let file = &parsed_files[file_idx];

        for (conflicting_name, mut spans) in name_conflicts_in_single_file {
            let name = &id_pool[conflicting_name];
            let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الملف", name);
            let mut diagnostic = Diagnostic::error(msg, vec![]);
            let mut occurrences = 1;

            let mut code_window = CodeWindow::new(&file.path, &file.lines, spans[0].start);

            nazmc_diagnostics::span::sort_spans(&mut spans);

            for span in spans {
                let occurrence_str = match occurrences {
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

                if occurrences == 1 {
                    code_window.mark_error(span, vec![occurrence_str]);
                } else {
                    code_window.mark_secondary(span, vec![occurrence_str]);
                }

                occurrences += 1;
            }

            diagnostic.push_code_window(code_window);
            diagnostics.push(diagnostic);
        }
    }

    if !diagnostics.is_empty() {
        eprint_diagnostics(diagnostics);
        exit(1)
    }
}
