// use std::collections::HashMap;

// use errors::NIRBuilderErrors;
// use nazmc_ast::ItemPath;
// use nazmc_data_pool::{Built, DataPool, PoolIdx};
// use nazmc_nir::{ItemInPkg, TypeKindAndIndex};
// use nazmc_resolve::{FileItemKindAndIdx, NameResolutionTree, ParsedFile};
// use thin_vec::ThinVec;
// mod errors;

// pub struct NIRBuilder<'a> {
//     /// The pool used to preserve ids string values
//     id_pool: &'a DataPool<Built>,
//     /// A map from pkgs ids segments to the pkgs indexes
//     packages: HashMap<ThinVec<PoolIdx>, usize>,
//     /// A map from the pkgs indexes to their segments
//     packages_names: ThinVec<ThinVec<PoolIdx>>,
//     /// A map from the pkgs indexes to the inner files indexes
//     packages_to_parsed_files: Vec<Vec<usize>>,
//     /// The parsed filese array
//     parsed_files: Vec<ParsedFile>,
//     /// The nrt produced by name resolver
//     nrt: NameResolutionTree,
//     /// The final NIR
//     nir: nazmc_nir::NIR,
//     current_pkg_idx: usize,
//     current_file_idx: usize,
//     /// stores the available names and their index in the let stms
//     local_bindings_stack: Vec<(PoolIdx, usize)>,
//     errs: NIRBuilderErrors,
// }

// impl<'a> NIRBuilder<'a> {
//     pub fn new(
//         id_pool: &'a DataPool<Built>,
//         packages: HashMap<ThinVec<PoolIdx>, usize>,
//         packages_names: ThinVec<ThinVec<PoolIdx>>,
//         packages_to_parsed_files: Vec<Vec<usize>>,
//         parsed_files: Vec<ParsedFile>,
//         nrt: NameResolutionTree,
//     ) -> Self {
//         Self {
//             id_pool,
//             packages,
//             packages_names,
//             packages_to_parsed_files,
//             parsed_files,
//             nrt,
//             nir: nazmc_nir::NIR::default(),
//             current_pkg_idx: 0,
//             current_file_idx: 0,
//             local_bindings_stack: vec![],
//             errs: NIRBuilderErrors::default(),
//         }
//     }

//     pub fn build(mut self) -> nazmc_nir::NIR {
//         for (pkg_idx, parsed_files_in_package) in
//             self.packages_to_parsed_files.into_iter().enumerate()
//         {
//             self.current_pkg_idx = pkg_idx;

//             for parsed_file_idx in parsed_files_in_package {
//                 self.current_file_idx = parsed_file_idx;
//                 let parsed_file = &self.parsed_files[self.current_file_idx];

//                 for (item_idx, item) in parsed_file.ast.items.iter().enumerate() {
//                     match &item.kind {
//                         nazmc_ast::ItemKind::UnitStruct => todo!(),
//                         nazmc_ast::ItemKind::TupleStruct(tuple_struct) => todo!(),
//                         nazmc_ast::ItemKind::FieldsStruct(fields_struct) => todo!(),
//                         nazmc_ast::ItemKind::Fn(_fn) => todo!(),
//                     }
//                 }
//             }
//         }
//         self.nir
//     }

//     #[inline]
//     fn current_file(&self) -> &nazmc_resolve::ParsedFile {
//         &self.parsed_files[self.current_file_idx]
//     }

//     fn lower_pkg_path_with_item_with_no_pkgs(
//         &mut self,
//         item: &nazmc_ast::ASTId,
//     ) -> (usize, nazmc_resolve::FileItemKindAndIdx) {
//         if let Some(Some(item)) = self.nrt.resolved_imports[self.current_pkg_idx]
//             .get(&self.current_file_idx)
//             .map(|imports| imports.get(&item.id))
//         {
//             (item.pkg_idx, item.item.kind_and_idx)
//         } else if let Some(item) = self.nrt.packages_to_items[self.current_pkg_idx].get(&item.id) {
//             (self.current_pkg_idx, item.kind_and_idx)
//         } else {
//             todo!("Import from star imports of the file")
//         }
//     }

//     fn lower_pkg_path_with_item(
//         &mut self,
//         ItemPath { pkg_path, item }: &nazmc_ast::ItemPath,
//     ) -> (usize, nazmc_resolve::FileItemKindAndIdx) {
//         if pkg_path.ids.is_empty() {
//             return self.lower_pkg_path_with_item_with_no_pkgs(&item);
//         }

//         let Some(resolved_pkg_idx) = self.packages.get(&pkg_path.ids) else {
//             self.errs.report_pkg_path_err(
//                 self.current_file_idx,
//                 pkg_path.ids.clone(),
//                 pkg_path.spans.clone(),
//             );
//             return (
//                 self.current_pkg_idx,
//                 nazmc_resolve::FileItemKindAndIdx::default(),
//             );
//         };

//         let Some(resolved_item) = self.nrt.packages_to_items[*resolved_pkg_idx].get(&item.id)
//         else {
//             self.errs
//                 .report_unresolved_item(self.current_file_idx, item.id, item.span);
//             return (
//                 self.current_pkg_idx,
//                 nazmc_resolve::FileItemKindAndIdx::default(),
//             );
//         };

//         let item_resolved_file = &self.parsed_files[resolved_item.file_idx];

//         let resolved_item_ast = &item_resolved_file.ast.items[resolved_item.item_idx];

//         if self.current_pkg_idx != *resolved_pkg_idx
//             && matches!(resolved_item_ast.vis, nazmc_ast::VisModifier::Default)
//         {
//             self.errs.report_encapsulation_err(
//                 self.current_file_idx,
//                 resolved_item.file_idx,
//                 resolved_item.item_idx,
//                 item.id,
//                 item.span,
//             );
//         }

//         (*resolved_pkg_idx, resolved_item.kind_and_idx)
//     }

//     fn lower_type(&mut self, typ: &nazmc_ast::Type) -> nazmc_nir::Type {
//         match typ {
//             nazmc_ast::Type::Path(pkg_path_with_item) => {
//                 let (item_pkg_idx, item_kind_and_index) =
//                     self.lower_pkg_path_with_item(pkg_path_with_item);

//                 match item_kind_and_index.kind() {
//                     FileItemKindAndIdx::UNIT_STRUCT
//                     | FileItemKindAndIdx::TUPLE_STRUCT
//                     | FileItemKindAndIdx::FIELDS_STRUCT => {}
//                     _ => {
//                         self.errs.report_wrong_file_item_found_err(
//                             self.current_file_idx,
//                             pkg_path_with_item.item.span,
//                             item_kind_and_index.kind(),
//                         );
//                     }
//                 }

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     TypeKindAndIndex::PATH,
//                     self.nir.types.paths.len(),
//                 );

//                 self.nir.types.paths.push(nazmc_nir::ItemInPkg {
//                     pkg_idx: item_pkg_idx,
//                     id: pkg_path_with_item.item.id,
//                 });

//                 let span = pkg_path_with_item
//                     .pkg_path
//                     .spans
//                     .first()
//                     .unwrap_or(&pkg_path_with_item.item.span)
//                     .merged_with(&pkg_path_with_item.item.span);

//                 nazmc_nir::Type { kind_and_idx, span }
//             }
//             nazmc_ast::Type::Paren(typ, _span) => self.lower_type(typ),
//             nazmc_ast::Type::Unit(span) => nazmc_nir::Type {
//                 kind_and_idx: nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::PATH,
//                     0,
//                 ),
//                 span: *span,
//             },
//             nazmc_ast::Type::Tuple(types, span) => {
//                 let types = types
//                     .iter()
//                     .map(|typ| self.lower_type(typ))
//                     .collect::<ThinVec<_>>();

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::TUPLE,
//                     self.nir.types.tuples.len(),
//                 );

//                 self.nir.types.tuples.push(nazmc_nir::TupleType {
//                     types,
//                     parens_span: *span,
//                 });

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//             nazmc_ast::Type::Slice(typ, span) => {
//                 let typ = self.lower_type(typ);

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::SLICE,
//                     self.nir.types.slices.len(),
//                 );

//                 self.nir.types.slices.push(typ);

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//             nazmc_ast::Type::Ptr(typ, span) => {
//                 let typ = self.lower_type(typ);

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::PTR,
//                     self.nir.types.ptrs.len(),
//                 );

//                 self.nir.types.ptrs.push(typ);

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//             nazmc_ast::Type::Ref(typ, span) => {
//                 let typ = self.lower_type(typ);

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::REF,
//                     self.nir.types.refs.len(),
//                 );

//                 self.nir.types.refs.push(typ);

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//             nazmc_ast::Type::PtrMut(typ, span) => {
//                 let typ = self.lower_type(typ);

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::PTR_MUT,
//                     self.nir.types.ptrs_mut.len(),
//                 );

//                 self.nir.types.ptrs_mut.push(typ);

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//             nazmc_ast::Type::RefMut(typ, span) => {
//                 let typ = self.lower_type(typ);

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::REF_MUT,
//                     self.nir.types.refs_mut.len(),
//                 );

//                 self.nir.types.refs_mut.push(typ);

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//             nazmc_ast::Type::Array(typ, size_expr, span) => {
//                 let typ = self.lower_type(typ);

//                 let size = self.lower_expr(size_expr);

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::ARRAY,
//                     self.nir.types.arrays.len(),
//                 );

//                 self.nir
//                     .types
//                     .arrays
//                     .push(nazmc_nir::ArrayType { typ, size });

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//             nazmc_ast::Type::Lambda(params, return_type, span) => {
//                 let params = params
//                     .iter()
//                     .map(|typ| self.lower_type(typ))
//                     .collect::<ThinVec<_>>();

//                 let return_type = self.lower_type(&return_type);

//                 let kind_and_idx = nazmc_nir::TypeKindAndIndex::new(
//                     nazmc_nir::TypeKindAndIndex::LAMBDA,
//                     self.nir.types.lambdas.len(),
//                 );

//                 self.nir.types.lambdas.push(nazmc_nir::LambdaType {
//                     params,
//                     return_type,
//                 });

//                 nazmc_nir::Type {
//                     kind_and_idx,
//                     span: *span,
//                 }
//             }
//         }
//     }

//     fn lower_fn(&mut self, _fn: &nazmc_ast::Fn) {}

//     fn lower_stm(&mut self, stm: &nazmc_ast::Stm) -> nazmc_nir::Stm {
//         todo!()
//     }

//     fn lower_expr(&mut self, expr: &nazmc_ast::Expr) -> nazmc_nir::Expr {
//         let span = expr.span;

//         match &expr.kind {
//             nazmc_ast::ExprKind::Literal(literal_expr) => {
//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::LITERAL,
//                     self.nir.exprs.literals.len(),
//                 );

//                 let literal_expr = match *literal_expr {
//                     nazmc_ast::LiteralExpr::Str(pool_idx) => nazmc_nir::LiteralExpr::Str(pool_idx),
//                     nazmc_ast::LiteralExpr::Char(ch) => nazmc_nir::LiteralExpr::Char(ch),
//                     nazmc_ast::LiteralExpr::Bool(b) => nazmc_nir::LiteralExpr::Bool(b),
//                     nazmc_ast::LiteralExpr::Num(num_kind) => {
//                         nazmc_nir::LiteralExpr::Num(match num_kind {
//                             nazmc_ast::NumKind::F4(n) => nazmc_nir::NumKind::F4(n),
//                             nazmc_ast::NumKind::F8(n) => nazmc_nir::NumKind::F8(n),
//                             nazmc_ast::NumKind::I(n) => nazmc_nir::NumKind::I(n),
//                             nazmc_ast::NumKind::I1(n) => nazmc_nir::NumKind::I1(n),
//                             nazmc_ast::NumKind::I2(n) => nazmc_nir::NumKind::I2(n),
//                             nazmc_ast::NumKind::I4(n) => nazmc_nir::NumKind::I4(n),
//                             nazmc_ast::NumKind::I8(n) => nazmc_nir::NumKind::I8(n),
//                             nazmc_ast::NumKind::U(n) => nazmc_nir::NumKind::U(n),
//                             nazmc_ast::NumKind::U1(n) => nazmc_nir::NumKind::U1(n),
//                             nazmc_ast::NumKind::U2(n) => nazmc_nir::NumKind::U2(n),
//                             nazmc_ast::NumKind::U4(n) => nazmc_nir::NumKind::U4(n),
//                             nazmc_ast::NumKind::U8(n) => nazmc_nir::NumKind::U8(n),
//                             nazmc_ast::NumKind::UnspecifiedInt(n) => {
//                                 nazmc_nir::NumKind::UnspecifiedInt(n)
//                             }
//                             nazmc_ast::NumKind::UnspecifiedFloat(n) => {
//                                 nazmc_nir::NumKind::UnspecifiedFloat(n)
//                             }
//                         })
//                     }
//                 };
//                 self.nir.exprs.literals.push(literal_expr);

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Path(pkg_path_with_item) => {
//                 if pkg_path_with_item.pkg_path.ids.is_empty() {
//                     let mut found = false;
//                     let mut idx_in_let_stms = 0;
//                     for (id, idx_in_let_stms_) in self.local_bindings_stack.iter().rev() {
//                         if pkg_path_with_item.item.id == *id {
//                             found = true;
//                             idx_in_let_stms = *idx_in_let_stms_;
//                             break;
//                         }
//                     }

//                     if found {
//                         let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                             nazmc_nir::ExprKindAndIndex::LOCAL_VAR,
//                             idx_in_let_stms,
//                         );

//                         return nazmc_nir::Expr { kind_and_idx, span };
//                     }
//                 }
//                 let (item_pkg_idx, item_kind_and_index) =
//                     self.lower_pkg_path_with_item(&pkg_path_with_item);

//                 match item_kind_and_index.kind() {
//                     FileItemKindAndIdx::FN
//                     | FileItemKindAndIdx::CONST
//                     | FileItemKindAndIdx::STATIC => {}
//                     _ => {
//                         self.errs.report_wrong_file_item_found_err(
//                             self.current_file_idx,
//                             pkg_path_with_item.item.span,
//                             item_kind_and_index.kind(),
//                         );
//                     }
//                 }

//                 if item_kind_and_index.kind() != FileItemKindAndIdx::FN {
//                     self.errs.report_wrong_file_item_found_err(
//                         self.current_file_idx,
//                         pkg_path_with_item.item.span,
//                         item_kind_and_index.kind(),
//                     );
//                 }

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::PATH,
//                     self.nir.exprs.paths.len(),
//                 );

//                 self.nir.exprs.paths.push(nazmc_nir::ItemInPkg {
//                     pkg_idx: item_pkg_idx,
//                     id: pkg_path_with_item.item.id,
//                 });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Call(call_expr) => {
//                 let on = self.lower_expr(&call_expr.on);

//                 let args = call_expr
//                     .args
//                     .iter()
//                     .map(|expr| self.lower_expr(expr))
//                     .collect();

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::CALL,
//                     self.nir.exprs.calls.len(),
//                 );

//                 self.nir.exprs.calls.push(nazmc_nir::CallExpr {
//                     on,
//                     args,
//                     parens_span: call_expr.parens_span,
//                 });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::UnitStruct(pkg_path_with_item) => {
//                 let (pkg_idx, item_kind_and_index) =
//                     self.lower_pkg_path_with_item(&pkg_path_with_item);

//                 if item_kind_and_index.kind() != FileItemKindAndIdx::UNIT_STRUCT {
//                     self.errs.report_wrong_file_item_found_err(
//                         self.current_file_idx,
//                         pkg_path_with_item.item.span,
//                         item_kind_and_index.kind(),
//                     );
//                 }

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::UNIT_STRUCT,
//                     self.nir.exprs.unit_structs.len(),
//                 );

//                 self.nir.exprs.unit_structs.push(nazmc_nir::ItemInPkg {
//                     pkg_idx,
//                     id: pkg_path_with_item.item.id,
//                 });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::TupleStruct(tuple_struct_expr) => {
//                 let (pkg_idx, item_kind_and_index) =
//                     self.lower_pkg_path_with_item(&tuple_struct_expr.item_path_idx);

//                 if item_kind_and_index.kind() != FileItemKindAndIdx::TUPLE_STRUCT {
//                     self.errs.report_wrong_file_item_found_err(
//                         self.current_file_idx,
//                         tuple_struct_expr.item_path_idx.item.span,
//                         item_kind_and_index.kind(),
//                     );
//                 }

//                 let args = tuple_struct_expr
//                     .args
//                     .iter()
//                     .map(|arg_expr| self.lower_expr(arg_expr))
//                     .collect();

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::TUPLE_STRUCT,
//                     self.nir.exprs.tuple_structs.len(),
//                 );

//                 self.nir
//                     .exprs
//                     .tuple_structs
//                     .push(nazmc_nir::TupleStructExpr {
//                         path: nazmc_nir::ItemInPkg {
//                             pkg_idx,
//                             id: tuple_struct_expr.item_path_idx.item.id,
//                         },
//                         args,
//                     });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::FieldsStruct(fields_struct_expr) => {
//                 let (pkg_idx, item_kind_and_index) =
//                     self.lower_pkg_path_with_item(&fields_struct_expr.path);

//                 if item_kind_and_index.kind() != FileItemKindAndIdx::FIELDS_STRUCT {
//                     self.errs.report_wrong_file_item_found_err(
//                         self.current_file_idx,
//                         fields_struct_expr.path.item.span,
//                         item_kind_and_index.kind(),
//                     );
//                 }

//                 let fields = fields_struct_expr
//                     .fields
//                     .iter()
//                     .map(|(field_id, field_expr)| nazmc_nir::FieldInStructExpr {
//                         name: nazmc_nir::NIRId {
//                             span: field_id.span,
//                             id: field_id.id,
//                         },
//                         expr: self.lower_expr(field_expr),
//                     })
//                     .collect();

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::FIELDS_STRUCT,
//                     self.nir.exprs.fields_structs.len(),
//                 );

//                 self.nir
//                     .exprs
//                     .fields_structs
//                     .push(nazmc_nir::FieldsStructExpr {
//                         path: nazmc_nir::ItemInPkg {
//                             pkg_idx,
//                             id: fields_struct_expr.path.item.id,
//                         },
//                         fields,
//                     });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Field(field_expr) => {
//                 let on = self.lower_expr(&field_expr.on);

//                 let name = nazmc_nir::NIRId {
//                     span: field_expr.name.span,
//                     id: field_expr.name.id,
//                 };

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::FIELD,
//                     self.nir.exprs.fields.len(),
//                 );

//                 self.nir
//                     .exprs
//                     .fields
//                     .push(nazmc_nir::FieldExpr { on, name });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Idx(idx_expr) => {
//                 let on = self.lower_expr(&idx_expr.on);

//                 let idx = self.lower_expr(&idx_expr.idx);

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::INDEX,
//                     self.nir.exprs.indexes.len(),
//                 );

//                 self.nir.exprs.indexes.push(nazmc_nir::IndexExpr {
//                     on,
//                     idx,
//                     brackets_span: idx_expr.brackets_span,
//                 });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::TupleIdx(tuple_idx_expr) => {
//                 let on = self.lower_expr(&tuple_idx_expr.on);

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::TUPLE_INDEX,
//                     self.nir.exprs.tuple_indexes.len(),
//                 );

//                 self.nir
//                     .exprs
//                     .tuple_indexes
//                     .push(nazmc_nir::TupleIndexExpr {
//                         on,
//                         idx: tuple_idx_expr.idx,
//                         idx_span: tuple_idx_expr.idx_span,
//                     });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Tuple(elements) => {
//                 let elements = elements
//                     .iter()
//                     .map(|arg_expr| self.lower_expr(arg_expr))
//                     .collect();

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::TUPLE,
//                     self.nir.exprs.tuples.len(),
//                 );

//                 self.nir
//                     .exprs
//                     .tuples
//                     .push(nazmc_nir::TupleExpr { elements });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::ArrayElemnts(elements) => {
//                 let elements = elements
//                     .iter()
//                     .map(|arg_expr| self.lower_expr(arg_expr))
//                     .collect();

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::ARRAY_ELEMENTS,
//                     self.nir.exprs.array_elements.len(),
//                 );

//                 self.nir
//                     .exprs
//                     .array_elements
//                     .push(nazmc_nir::ArrayElementsExpr { elements });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::ArrayElemntsSized(array_elements_sized_expr) => {
//                 let repeat = self.lower_expr(&array_elements_sized_expr.repeat);

//                 let size = self.lower_expr(&array_elements_sized_expr.size);

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::ARRAY_ELEMENTS_SIZED,
//                     self.nir.exprs.array_elements_sized.len(),
//                 );

//                 self.nir
//                     .exprs
//                     .array_elements_sized
//                     .push(nazmc_nir::ArrayElementsSizedExpr { repeat, size });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Parens(expr) => self.lower_expr(expr),
//             nazmc_ast::ExprKind::On => {
//                 let kind_and_idx =
//                     nazmc_nir::ExprKindAndIndex::new(nazmc_nir::ExprKindAndIndex::ON, 0);
//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Return(expr) => {
//                 let Some(expr) = expr else {
//                     let kind_and_idx =
//                         nazmc_nir::ExprKindAndIndex::new(nazmc_nir::ExprKindAndIndex::RETURN, 0);
//                     return nazmc_nir::Expr { kind_and_idx, span };
//                 };

//                 let expr = self.lower_expr(expr);

//                 let kind_and_idx = nazmc_nir::ExprKindAndIndex::new(
//                     nazmc_nir::ExprKindAndIndex::RETURN_WITH_VALUE,
//                     self.nir.exprs.returns.len(),
//                 );

//                 self.nir.exprs.returns.push(nazmc_nir::ReturWithValueExpr {
//                     expr_to_return: expr,
//                 });

//                 nazmc_nir::Expr { kind_and_idx, span }
//             }
//             nazmc_ast::ExprKind::Break(expr) => todo!(),
//             nazmc_ast::ExprKind::Continue => todo!(),
//             nazmc_ast::ExprKind::UnaryOp(unary_op_expr) => todo!(),
//             nazmc_ast::ExprKind::BinaryOp(binary_op_expr) => todo!(),
//             nazmc_ast::ExprKind::If(if_expr) => todo!(),
//             nazmc_ast::ExprKind::Lambda(lambda_expr) => todo!(),
//         }
//     }
// }
