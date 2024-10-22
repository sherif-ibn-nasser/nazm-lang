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
pub struct ASTItemsCounter {
    pub unit_structs: usize,
    pub tuple_structs: usize,
    pub fields_structs: usize,
    pub fns: usize,
}

#[derive(Clone, Copy)]
pub struct FileItemKindAndIdx(u64);

impl FileItemKindAndIdx {
    const KIND_BITS: u64 = 4;
    const KIND_SHIFT: u64 = 64 - Self::KIND_BITS;
    const KIND_MASK: u64 = 0b11 << Self::KIND_SHIFT;
    const INDEX_MASK: u64 = !Self::KIND_MASK;

    // Possible kinds
    pub const UNIT_STRUCT: u64 = 0 << Self::KIND_SHIFT;
    pub const TUPLE_STRUCT: u64 = 1 << Self::KIND_SHIFT;
    pub const FIELDS_STRUCT: u64 = 2 << Self::KIND_SHIFT;
    pub const FN: u64 = 3 << Self::KIND_SHIFT;

    // Create a new encoded value for a given kind and index
    pub fn new(kind: u64, index: usize) -> Self {
        Self(kind | index as u64)
    }

    // Decode the kind of the expression
    pub fn kind(self) -> u64 {
        self.0 & Self::KIND_MASK
    }

    // Decode the index of the expression
    pub fn index(self) -> usize {
        (self.0 & Self::INDEX_MASK) as usize
    }
}

#[derive(Clone, Copy)]
pub struct ItemInFile {
    /// The kind and index in the NIR
    pub kind_and_idx: FileItemKindAndIdx,
    /// The file index where the item is defined
    pub file_idx: usize,
    /// The item index in the list of file items
    pub item_idx: usize,
}

#[derive(Clone, Copy)]
pub struct ResolvedImport {
    /// The pkg idx of the resolved item
    pub pkg_idx: usize,
    /// The resolved item
    pub item: ItemInFile,
    /// The alias of the resolved item
    pub alias: nazmc_ast::ASTId,
}

pub struct NameResolver<'a> {
    /// The pool used to preserve ids string values
    id_pool: &'a DataPool<Built>,
    /// A map from pkgs ids segments to the pkgs indexes
    packages: &'a HashMap<ThinVec<PoolIdx>, usize>,
    /// A map from the pkgs indexes to their segments
    packages_names: &'a [ThinVec<PoolIdx>],
    /// A map from the pkgs indexes to the inner files indexes
    packages_to_parsed_files: &'a [Vec<usize>],
    /// The parsed filese array
    parsed_files: &'a [ParsedFile],
    /// The diagnostics which will be filled in different phases
    diagnostics: Vec<Diagnostic<'a>>,
    nrt: NameResolutionTree,
}

pub struct NameResolutionTree {
    /// Each pkg has a map of ids to their occurrence in the package (the file index and the item index in that file)
    pub packages_to_items: Vec<HashMap<PoolIdx, ItemInFile>>,
    /// Each pkg will have HashMap<usize, Vec<ResolvedImport>>,
    /// which is the map of file idx to its resolved imports
    pub resolved_imports: Vec<HashMap<usize, Vec<ResolvedImport>>>,
    /// Each pkg will have HashMap<usize, Vec<usize>>,
    /// which is the map of file idx to its resolved pkgs indexes
    pub resolved_star_imports: Vec<HashMap<usize, Vec<usize>>>,
    /// The counter for items (used to construct NIR)
    pub ast_counter: ASTItemsCounter,
}

impl<'a> NameResolver<'a> {
    pub fn new(
        id_pool: &'a DataPool<Built>,
        packages: &'a HashMap<ThinVec<PoolIdx>, usize>,
        packages_names: &'a [ThinVec<PoolIdx>],
        packages_to_parsed_files: &'a [Vec<usize>],
        parsed_files: &'a [ParsedFile],
    ) -> Self {
        Self {
            id_pool,
            packages,
            packages_names,
            packages_to_parsed_files,
            parsed_files,
            diagnostics: vec![],
            nrt: NameResolutionTree {
                packages_to_items: vec![HashMap::new(); packages.len()],
                resolved_imports: vec![HashMap::new(); packages.len()],
                resolved_star_imports: vec![HashMap::new(); packages.len()],
                ast_counter: ASTItemsCounter::default(),
            },
        }
    }

    pub fn resolve(mut self) -> NameResolutionTree {
        self.check_pkg_items_conflicts();

        if !self.diagnostics.is_empty() {
            eprint_diagnostics(self.diagnostics);
            exit(1)
        }

        self.resolve_imports();

        if !self.diagnostics.is_empty() {
            eprint_diagnostics(self.diagnostics);
            exit(1)
        }

        self.nrt
    }

    fn check_pkg_items_conflicts(&mut self) {
        let mut conflicts: HashMap<(usize, PoolIdx), HashMap<usize, Vec<Span>>> = HashMap::new();
        //                          ^^^^^  ^^^^^^^           ^^^^^  ^^^^^^^^^
        //                          |      |                 |      |
        //                          |      |                 |      span occurrences in the file
        //                          |      |                 parsed file idx
        //                          |      conflicting name
        //                          package idx

        for (pkg_idx, parsed_files_in_package) in self.packages_to_parsed_files.iter().enumerate() {
            for parsed_file_idx in parsed_files_in_package {
                self.check_conflicts_in_file(*parsed_file_idx, pkg_idx, &mut conflicts);
            }
        }

        for ((_pkg_idx, conflicting_name), name_conflicts_in_single_package) in conflicts {
            let name = &self.id_pool[conflicting_name];
            let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الحزمة", name);
            let mut diagnostic = Diagnostic::error(msg, vec![]);
            let mut occurrences = 1;

            for (file_idx, spans) in name_conflicts_in_single_package {
                let parsed_file = &self.parsed_files[file_idx];
                let code_window = occurrences_code_window(parsed_file, &mut occurrences, spans);
                diagnostic.push_code_window(code_window);
            }

            self.diagnostics.push(diagnostic);
        }
    }

    #[inline]
    fn check_conflicts_in_file(
        &mut self,
        parsed_file_idx: usize,
        pkg_idx: usize,
        conflicts: &mut HashMap<(usize, PoolIdx), HashMap<usize, Vec<Span>>>,
    ) {
        let items_in_package = &mut self.nrt.packages_to_items[pkg_idx];
        let parsed_file = &self.parsed_files[parsed_file_idx];

        for (item_idx, item) in parsed_file.ast.items.iter().enumerate() {
            match items_in_package.get(&item.name.id) {
                Some(first_occurrence) => {
                    conflicts
                        .entry((pkg_idx, item.name.id))
                        .or_insert_with(|| {
                            let first_occurrence_span =
                                self.parsed_files[first_occurrence.file_idx].ast.items
                                    [first_occurrence.item_idx]
                                    .name
                                    .span;

                            let mut h = HashMap::new();
                            h.insert(first_occurrence.file_idx, vec![first_occurrence_span]);
                            h
                        })
                        .entry(parsed_file_idx)
                        .or_default()
                        .push(item.name.span);
                }
                None => {
                    let (kind, index) = match item.kind {
                        nazmc_ast::ItemKind::UnitStruct => (
                            FileItemKindAndIdx::UNIT_STRUCT,
                            &mut self.nrt.ast_counter.unit_structs,
                        ),
                        nazmc_ast::ItemKind::TupleStruct(_) => (
                            FileItemKindAndIdx::TUPLE_STRUCT,
                            &mut self.nrt.ast_counter.tuple_structs,
                        ),
                        nazmc_ast::ItemKind::FieldsStruct(_) => (
                            FileItemKindAndIdx::FIELDS_STRUCT,
                            &mut self.nrt.ast_counter.fields_structs,
                        ),
                        nazmc_ast::ItemKind::Fn(_) => {
                            (FileItemKindAndIdx::FN, &mut self.nrt.ast_counter.fns)
                        }
                    };

                    let kind_and_idx = FileItemKindAndIdx::new(kind, *index);

                    *index += 1;

                    items_in_package.insert(
                        item.name.id,
                        ItemInFile {
                            kind_and_idx,
                            file_idx: parsed_file_idx,
                            item_idx,
                        },
                    );
                }
            }
        }
    }

    fn resolve_imports(&mut self) {
        for (pkg_idx, parsed_files_in_package) in self.packages_to_parsed_files.iter().enumerate() {
            for parsed_file_idx in parsed_files_in_package.iter() {
                self.resolve_file_imports(pkg_idx, *parsed_file_idx);
                self.resolve_file_star_imports(pkg_idx, *parsed_file_idx);
            }
        }

        let mut conflicts: HashMap<usize, HashMap<PoolIdx, Vec<Span>>> = HashMap::new();
        //                         ^^^^^          ^^^^^^^  ^^^^^^^^^
        //                         |              |        |
        //                         |              |        span occurrences in the file
        //                         |              conflicting name
        //                         file idx

        for (pkg_idx, files_in_pkg) in self.nrt.resolved_imports.iter().enumerate() {
            for (parsed_file_idx, resolved_imports) in files_in_pkg.iter() {
                for resolved_import in resolved_imports {
                    let alias = &resolved_import.alias;

                    let Some(item_with_same_id) =
                        self.nrt.packages_to_items[pkg_idx].get(&alias.id)
                    else {
                        continue;
                    };

                    let parsed_file = &self.parsed_files[*parsed_file_idx];

                    conflicts
                        .entry(*parsed_file_idx)
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

        for (file_idx, name_conflicts_in_single_file) in conflicts {
            let parsed_file = &self.parsed_files[file_idx];

            for (conflicting_name, spans) in name_conflicts_in_single_file {
                let name = &self.id_pool[conflicting_name];
                let msg = format!("يوجد أكثر من عنصر بنفس الاسم `{}` في نفس الملف", name);
                let mut diagnostic = Diagnostic::error(msg, vec![]);
                let mut occurrences = 1;
                let code_window = occurrences_code_window(parsed_file, &mut occurrences, spans);
                diagnostic.push_code_window(code_window);
                self.diagnostics.push(diagnostic);
            }
        }
    }

    #[inline]
    fn resolve_file_star_imports(&mut self, pkg_idx: usize, parsed_file_idx: usize) {
        let parsed_file = &self.parsed_files[parsed_file_idx];
        for import in &parsed_file.ast.star_imports {
            let Some(resolved_package_idx) = self.packages.get(&import.ids) else {
                self.add_pkg_path_err(&parsed_file, import.ids.clone(), import.spans.clone());
                continue;
            };

            self.nrt.resolved_star_imports[pkg_idx]
                .entry(parsed_file_idx)
                .or_default()
                .push(*resolved_package_idx);
        }
    }

    #[inline]
    fn resolve_file_imports(&mut self, pkg_idx: usize, parsed_file_idx: usize) {
        let parsed_file = &self.parsed_files[parsed_file_idx];
        for (import, item_alias) in &parsed_file.ast.imports {
            let Some(resolved_package_idx) = self.packages.get(&import.pkg_path.ids) else {
                self.add_pkg_path_err(
                    &parsed_file,
                    import.pkg_path.ids.clone(),
                    import.pkg_path.spans.clone(),
                );
                continue;
            };

            let Some(resolved_item) =
                self.nrt.packages_to_items[*resolved_package_idx].get(&import.item.id)
            else {
                self.add_unresolved_import_err(&parsed_file, import.item.id, import.item.span);
                continue;
            };

            let item_resolved_file = &self.parsed_files[resolved_item.file_idx];

            let resolved_item_ast = &item_resolved_file.ast.items[resolved_item.item_idx];

            if pkg_idx != *resolved_package_idx
                && matches!(resolved_item_ast.vis, nazmc_ast::VisModifier::Default)
            {
                self.add_encapsulation_err(
                    parsed_file,
                    item_resolved_file,
                    import,
                    resolved_item_ast,
                );
            } else {
                self.nrt.resolved_imports[pkg_idx]
                    .entry(parsed_file_idx)
                    .or_default()
                    .push(ResolvedImport {
                        pkg_idx: *resolved_package_idx,
                        item: *resolved_item,
                        alias: *item_alias,
                    });
            }
        }
    }

    fn add_encapsulation_err(
        &mut self,
        parsed_file: &'a ParsedFile,
        item_resolved_file: &'a ParsedFile,
        import: &nazmc_ast::PkgPathWithItem,
        resolved_item_ast: &nazmc_ast::Item,
    ) {
        let name = &self.id_pool[import.item.id];
        let item_kind_str = item_kind_to_str(&resolved_item_ast.kind);
        let msg = match resolved_item_ast.kind {
            nazmc_ast::ItemKind::UnitStruct
            | nazmc_ast::ItemKind::TupleStruct(_)
            | nazmc_ast::ItemKind::FieldsStruct(_) => {
                format!(
                    "لا يمكن الوصول إلى هيكل `{}` لأنه خاص بالحزمة التابع لها",
                    name
                )
            }
            nazmc_ast::ItemKind::Fn(_) => format!(
                "لا يمكن الوصول إلى دالة `{}` لأنها خاصة بالحزمة التابعة لها",
                name
            ),
        };

        let mut code_window = CodeWindow::new(
            &parsed_file.path,
            &parsed_file.lines,
            import.item.span.start,
        );
        code_window.mark_error(import.item.span, vec![]);
        let mut diagnostic = Diagnostic::error(msg, vec![code_window]);

        let help_msg = format!("تم العثور على {} هنا", item_kind_str);
        let mut help_code_window = CodeWindow::new(
            &item_resolved_file.path,
            &item_resolved_file.lines,
            resolved_item_ast.name.span.start,
        );
        help_code_window.mark_note(resolved_item_ast.name.span, vec![]);
        let help = Diagnostic::note(help_msg, vec![help_code_window]);
        diagnostic.chain(help);

        self.diagnostics.push(diagnostic);
    }

    fn add_unresolved_import_err(&mut self, file: &'a ParsedFile, id: PoolIdx, span: Span) {
        let name = &self.id_pool[id];
        let msg = format!("لم يتم العثور على الاسم `{}` في المسار", name);

        let mut code_window = CodeWindow::new(&file.path, &file.lines, span.start);

        code_window.mark_error(
            span,
            vec!["هذا الاسم غير موجود داخل المسار المحدد".to_string()],
        );

        let mut diagnostic = Diagnostic::error(msg, vec![code_window]);

        let mut possible_paths = vec![];

        for (pkg_idx, pkg_to_items) in self.nrt.packages_to_items.iter().enumerate() {
            if let Some(found_item) = pkg_to_items.get(&id) {
                let item_file = &self.parsed_files[found_item.file_idx];
                let item_ast = &item_file.ast.items[found_item.item_idx];
                let item_span_cursor = item_ast.name.span.start;
                let item_kind_str = item_kind_to_str(&item_ast.kind);
                let pkg_name = self.fmt_pkg_name(pkg_idx);
                let name = &self.id_pool[id];
                let item_path = format!(
                    "{}:{}:{}",
                    &item_file.path,
                    item_span_cursor.line + 1,
                    item_span_cursor.col + 1
                );
                let path = format!(
                    "\t- {} {}::{} في {}",
                    item_kind_str, pkg_name, name, item_path
                );

                possible_paths.push(path);
            }
        }

        if !possible_paths.is_empty() {
            let mut help = Diagnostic::help(
                format!("تم العثور على عناصر مشابهة بنفس الاسم في المسارات التالية:"),
                vec![],
            );

            for t in possible_paths {
                help.chain_free_text(t);
            }

            diagnostic.chain(help);
        }

        self.diagnostics.push(diagnostic);
    }

    fn add_pkg_path_err(
        &mut self,
        file: &'a ParsedFile,
        mut pkg_path: ThinVec<PoolIdx>,
        mut pkg_path_spans: ThinVec<Span>,
    ) {
        while let Some(first_invalid_seg) = pkg_path.pop() {
            let first_invalid_seg_span = pkg_path_spans.pop().unwrap();

            if self.packages.contains_key(&pkg_path) {
                self.add_unresolved_import_err(&file, first_invalid_seg, first_invalid_seg_span);
            }
        }
    }

    fn fmt_pkg_name(&self, pkg_idx: usize) -> String {
        self.packages_names[pkg_idx]
            .iter()
            .map(|id| &self.id_pool[*id])
            .collect::<Vec<_>>()
            .join("::")
    }
}

#[inline]
fn item_kind_to_str(kind: &nazmc_ast::ItemKind) -> &'static str {
    match kind {
        nazmc_ast::ItemKind::UnitStruct
        | nazmc_ast::ItemKind::TupleStruct(_)
        | nazmc_ast::ItemKind::FieldsStruct(_) => "الهيكل",
        nazmc_ast::ItemKind::Fn(_) => "الدالة",
    }
}

fn occurrences_code_window<'a>(
    parsed_file: &'a ParsedFile,
    occurrences: &mut usize,
    mut spans: Vec<Span>,
) -> CodeWindow<'a> {
    let mut code_window = CodeWindow::new(&parsed_file.path, &parsed_file.lines, spans[0].start);

    nazmc_diagnostics::span::sort_spans(&mut spans);

    for span in spans {
        let occurrence_str = match *occurrences {
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

        if *occurrences == 1 {
            code_window.mark_error(span, vec![occurrence_str]);
        } else {
            code_window.mark_secondary(span, vec![occurrence_str]);
        }

        *occurrences += 1;
    }

    code_window
}
