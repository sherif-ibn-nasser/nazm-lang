use std::{collections::HashMap, process::exit};

use nazmc_data_pool::{Built, DataPool, PoolIdx};
use nazmc_diagnostics::{eprint_diagnostics, span::Span, CodeWindow, Diagnostic};
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

struct ResolvedImport {
    file_idx: usize,
    item_id: PoolIdx,
    item_to_mod_idx: usize,
}

struct ResolvedStarImport {
    file_idx: usize,
    mod_idx: usize,
}

pub struct ItemInFile {
    pub file_idx: usize,
    pub item_idx: usize,
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

    for (package_idx, parsed_files_in_package) in packages_to_parsed_files.iter().enumerate() {
        packages_to_items.push(HashMap::default());

        let items_in_package = &mut packages_to_items[package_idx];

        for parsed_file_idx in parsed_files_in_package {
            let parsed_file = &parsed_files[*parsed_file_idx];

            for (item_idx, item) in parsed_file.ast.items.iter().enumerate() {
                match items_in_package.get(&item.name.id) {
                    Some(first_occurrence) => {
                        conflicts
                            .entry((package_idx, item.name.id))
                            .or_insert_with(|| {
                                let first_occurrence_span = parsed_files[first_occurrence.file_idx]
                                    .ast
                                    .items[first_occurrence.item_idx]
                                    .name
                                    .span;
                                let mut h = HashMap::new();
                                h.insert(*parsed_file_idx, vec![first_occurrence_span]);
                                h
                            })
                            .entry(*parsed_file_idx)
                            .or_default()
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

    for ((_package_idx, conflicting_name), name_conflicts_in_single_package) in conflicts {
        let name = &id_pool[conflicting_name];
        let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الحزمة", name);
        let mut diagnostic = Diagnostic::error(msg, vec![]);
        let mut occurrences = 1;

        for (file_idx, spans) in name_conflicts_in_single_package {
            let file = &parsed_files[file_idx];
            let mut code_window = CodeWindow::new(&file.path, &file.lines, spans[0].start);

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

        Diagnostic::error(msg, vec![code_window])
    };

    let pkg_path_err = |file: &'a ParsedFile,
                        mut pkg_path: ThinVec<PoolIdx>,
                        mut pkg_path_spans: ThinVec<Span>| {
        while let Some(first_invalid_seg) = pkg_path.pop() {
            let first_invalid_seg_span = pkg_path_spans.pop().unwrap();

            if packages.contains_key(&pkg_path) {
                return unresolved_import_err(&file, first_invalid_seg, first_invalid_seg_span);
            }
        }
        unreachable!()
    };

    for (package_idx, parsed_files_in_package) in packages_to_parsed_files.iter().enumerate() {
        for parsed_file_idx in parsed_files_in_package {
            let file = &parsed_files[*parsed_file_idx];

            for import in &file.ast.imports {
                match packages.get(&import.pkg_path.ids) {
                    Some(resolved_package_idx) => {
                        match packages_to_items[*resolved_package_idx].get(&import.item.id) {
                            Some(ItemInFile { file_idx, item_idx }) => todo!(),
                            None => {
                                diagnostics.push(unresolved_import_err(
                                    &file,
                                    import.item.id,
                                    import.item.span,
                                ));
                            }
                        }
                    }
                    None => {
                        diagnostics.push(pkg_path_err(
                            &file,
                            import.pkg_path.ids.clone(),
                            import.pkg_path.spans.clone(),
                        ));
                    }
                }
            }

            for import in &file.ast.star_imports {
                match packages.get(&import.ids) {
                    Some(resolved_package_idx) => todo!(),
                    None => {
                        diagnostics.push(pkg_path_err(
                            &file,
                            import.ids.clone(),
                            import.spans.clone(),
                        ));
                    }
                }
            }
        }
    }

    if !diagnostics.is_empty() {
        eprint_diagnostics(diagnostics);
        exit(1)
    }
}
