use std::{collections::HashMap, process::exit};

use nazmc_data_pool::{Built, DataPool, PoolIdx};
use nazmc_diagnostics::{eprint_diagnostics, span::Span, CodeWindow, Diagnostic};

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

struct UnresolvedImport {
    first_invalid_seg: PoolIdx,
    first_invalid_seg_span: Span,
    file_idx: usize,
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

pub struct ItemIdxInFile {
    pub file_idx: usize,
    pub item_idx: usize,
}

pub fn check_conflicts(
    packages_to_parsed_files: Vec<Vec<usize>>,
    parsed_files: Vec<ParsedFile>,
    id_pool: &DataPool<Built>,
) -> Vec<HashMap<PoolIdx, Vec<ItemIdxInFile>>> {
    // Vector of maps for items ids in each package mapped to to occurence of that id in each file and its index in the file
    // The vector of (usize, usize) should be at the end have the length 1
    // As the item id should not repeated across the package
    let mut packages_to_items: Vec<HashMap<PoolIdx, Vec<ItemIdxInFile>>> =
        Vec::with_capacity(packages_to_parsed_files.len());

    for (package_idx, parsed_files_in_package) in packages_to_parsed_files.iter().enumerate() {
        packages_to_items.push(HashMap::default());

        let items_in_package = &mut packages_to_items[package_idx];

        for parsed_file_idx in parsed_files_in_package.iter() {
            let parsed_file = &parsed_files[*parsed_file_idx];

            for (item_idx, item) in parsed_file.ast.items.iter().enumerate() {
                items_in_package
                    .entry(item.name.id)
                    .or_default()
                    .push(ItemIdxInFile {
                        file_idx: *parsed_file_idx,
                        item_idx,
                    });
            }
        }
    }

    let mut diagnostics = vec![];

    for package_to_items in &mut packages_to_items {
        for (item_id, occurrences_vec) in package_to_items {
            if occurrences_vec.len() <= 1 {
                continue;
            }

            let name = &id_pool[*item_id];
            let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الحزمة", name);
            let mut diagnostic = Diagnostic::error(msg, vec![]);
            let mut occurrences = 1;

            occurrences_vec.sort_by(|a, b| a.file_idx.cmp(&b.file_idx));
            occurrences_vec
                .chunk_by(|a, b| a.file_idx == b.file_idx)
                .for_each(|occurunces_vec| {
                    let ItemIdxInFile {
                        file_idx,
                        item_idx: first_item_idx,
                    } = occurrences_vec[0];

                    let file = &parsed_files[file_idx];
                    let first_item_span_cursor = file.ast.items[first_item_idx].name.span.start;

                    let mut code_window =
                        CodeWindow::new(&file.path, &file.lines, first_item_span_cursor);

                    for occurunce in occurunces_vec {
                        let item_span = file.ast.items[occurunce.item_idx].name.span;

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
                            code_window.mark_error(item_span, vec![occurrence_str]);
                        } else {
                            code_window.mark_secondary(item_span, vec![occurrence_str]);
                        }

                        occurrences += 1;
                    }

                    diagnostic.push_code_window(code_window);
                });

            diagnostics.push(diagnostic);
        }
    }

    if !diagnostics.is_empty() {
        eprint_diagnostics(diagnostics);
        exit(1)
    }

    todo!()
}
