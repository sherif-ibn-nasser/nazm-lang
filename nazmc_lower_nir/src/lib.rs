use std::collections::HashMap;

use errors::NIRBuilderErrors;
use nazmc_ast::PkgPathWithItem;
use nazmc_data_pool::{Built, DataPool, PoolIdx};
use nazmc_nir::{ItemInPkg, TypeKind};
use nazmc_resolve::{FileItemKindAndIdx, NameResolutionTree, ParsedFile};
use thin_vec::ThinVec;
mod errors;

pub struct NIRBuilder<'a> {
    /// The pool used to preserve ids string values
    id_pool: &'a DataPool<Built>,
    /// A map from pkgs ids segments to the pkgs indexes
    packages: HashMap<ThinVec<PoolIdx>, usize>,
    /// A map from the pkgs indexes to their segments
    packages_names: ThinVec<ThinVec<PoolIdx>>,
    /// A map from the pkgs indexes to the inner files indexes
    packages_to_parsed_files: Vec<Vec<usize>>,
    /// The parsed filese array
    parsed_files: Vec<ParsedFile>,
    /// The nrt produced by name resolver
    nrt: NameResolutionTree,
    /// The final NIR
    nir: nazmc_nir::NIR,
    current_pkg_idx: usize,
    current_file_idx: usize,
    errs: NIRBuilderErrors,
}

impl<'a> NIRBuilder<'a> {
    pub fn new(
        id_pool: &'a DataPool<Built>,
        packages: HashMap<ThinVec<PoolIdx>, usize>,
        packages_names: ThinVec<ThinVec<PoolIdx>>,
        packages_to_parsed_files: Vec<Vec<usize>>,
        parsed_files: Vec<ParsedFile>,
        nrt: NameResolutionTree,
    ) -> Self {
        Self {
            id_pool,
            packages,
            packages_names,
            packages_to_parsed_files,
            parsed_files,
            nrt,
            nir: nazmc_nir::NIR::default(),
            current_pkg_idx: 0,
            current_file_idx: 0,
            errs: NIRBuilderErrors::default(),
        }
    }

    pub fn build(mut self) -> nazmc_nir::NIR {
        for (pkg_idx, parsed_files_in_package) in
            self.packages_to_parsed_files.into_iter().enumerate()
        {
            self.current_pkg_idx = pkg_idx;

            for parsed_file_idx in parsed_files_in_package {
                self.current_file_idx = parsed_file_idx;
                let parsed_file = &self.parsed_files[self.current_file_idx];

                for (item_idx, item) in parsed_file.ast.items.iter().enumerate() {
                    match &item.kind {
                        nazmc_ast::ItemKind::UnitStruct => todo!(),
                        nazmc_ast::ItemKind::TupleStruct(tuple_struct) => todo!(),
                        nazmc_ast::ItemKind::FieldsStruct(fields_struct) => todo!(),
                        nazmc_ast::ItemKind::Fn(_fn) => todo!(),
                    }
                }
            }
        }
        self.nir
    }

    #[inline]
    fn current_file(&self) -> &nazmc_resolve::ParsedFile {
        &self.parsed_files[self.current_file_idx]
    }

    fn lower_pkg_path_with_item_with_no_pkgs(
        &mut self,
        item: &nazmc_ast::ASTId,
    ) -> (usize, nazmc_resolve::FileItemKindAndIdx) {
        if let Some(Some(item)) = self.nrt.resolved_imports[self.current_pkg_idx]
            .get(&self.current_file_idx)
            .map(|imports| imports.get(&item.id))
        {
            (item.pkg_idx, item.item.kind_and_idx)
        } else if let Some(item) = self.nrt.packages_to_items[self.current_pkg_idx].get(&item.id) {
            (self.current_pkg_idx, item.kind_and_idx)
        } else {
            todo!("Star imports")
        }
    }

    fn lower_pkg_path_with_item(
        &mut self,
        PkgPathWithItem { pkg_path, item }: &nazmc_ast::PkgPathWithItem,
    ) -> (usize, nazmc_resolve::FileItemKindAndIdx) {
        if pkg_path.ids.is_empty() {
            return self.lower_pkg_path_with_item_with_no_pkgs(&item);
        }

        let Some(resolved_pkg_idx) = self.packages.get(&pkg_path.ids) else {
            self.errs.report_pkg_path_err(
                self.current_file_idx,
                pkg_path.ids.clone(),
                pkg_path.spans.clone(),
            );
            return (
                self.current_pkg_idx,
                nazmc_resolve::FileItemKindAndIdx::default(),
            );
        };

        let Some(resolved_item) = self.nrt.packages_to_items[*resolved_pkg_idx].get(&item.id)
        else {
            self.errs
                .report_unresolved_item(self.current_file_idx, item.id, item.span);
            return (
                self.current_pkg_idx,
                nazmc_resolve::FileItemKindAndIdx::default(),
            );
        };

        let item_resolved_file = &self.parsed_files[resolved_item.file_idx];

        let resolved_item_ast = &item_resolved_file.ast.items[resolved_item.item_idx];

        if self.current_pkg_idx != *resolved_pkg_idx
            && matches!(resolved_item_ast.vis, nazmc_ast::VisModifier::Default)
        {
            self.errs.report_encapsulation_err(
                self.current_file_idx,
                resolved_item.file_idx,
                resolved_item.item_idx,
                item.id,
                item.span,
            );
        }

        (*resolved_pkg_idx, resolved_item.kind_and_idx)
    }

    fn lower_type(&mut self, typ: &nazmc_ast::Type) -> nazmc_nir::Type {
        match typ {
            nazmc_ast::Type::Path(pkg_path_with_item) => {
                let (item_pkg_idx, item_kind_and_index) =
                    self.lower_pkg_path_with_item(pkg_path_with_item);

                if let FileItemKindAndIdx::FN = item_kind_and_index.kind() {
                    self.errs.report_struct_is_expected_in_path(
                        self.current_file_idx,
                        pkg_path_with_item.item.span,
                        FileItemKindAndIdx::FN,
                    );
                }

                let kind_and_idx =
                    nazmc_nir::TypeKind::new(TypeKind::PATH, self.nir.types.paths.len());

                self.nir.types.paths.push(nazmc_nir::ItemInPkg {
                    pkg_idx: item_pkg_idx,
                    id: pkg_path_with_item.item.id,
                });

                let span = pkg_path_with_item
                    .pkg_path
                    .spans
                    .first()
                    .unwrap_or(&pkg_path_with_item.item.span)
                    .merged_with(&pkg_path_with_item.item.span);

                nazmc_nir::Type { kind_and_idx, span }
            }
            nazmc_ast::Type::Paren(typ, _span) => self.lower_type(typ),
            nazmc_ast::Type::Unit(span) => nazmc_nir::Type {
                kind_and_idx: nazmc_nir::TypeKind::new(nazmc_nir::TypeKind::PATH, 0),
                span: *span,
            },
            nazmc_ast::Type::Tuple(types, span) => {
                let types = types
                    .iter()
                    .map(|typ| self.lower_type(typ))
                    .collect::<ThinVec<_>>();

                let kind_and_idx = nazmc_nir::TypeKind::new(
                    nazmc_nir::TypeKind::TUPLE,
                    self.nir.types.tuples.len(),
                );

                self.nir.types.tuples.push(nazmc_nir::TupleType {
                    types,
                    parens_span: *span,
                });

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
            nazmc_ast::Type::Slice(typ, span) => {
                let typ = self.lower_type(typ);

                let kind_and_idx = nazmc_nir::TypeKind::new(
                    nazmc_nir::TypeKind::SLICE,
                    self.nir.types.slices.len(),
                );

                self.nir.types.slices.push(typ);

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
            nazmc_ast::Type::Ptr(typ, span) => {
                let typ = self.lower_type(typ);

                let kind_and_idx =
                    nazmc_nir::TypeKind::new(nazmc_nir::TypeKind::PTR, self.nir.types.ptrs.len());

                self.nir.types.ptrs.push(typ);

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
            nazmc_ast::Type::Ref(typ, span) => {
                let typ = self.lower_type(typ);

                let kind_and_idx =
                    nazmc_nir::TypeKind::new(nazmc_nir::TypeKind::REF, self.nir.types.refs.len());

                self.nir.types.refs.push(typ);

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
            nazmc_ast::Type::PtrMut(typ, span) => {
                let typ = self.lower_type(typ);

                let kind_and_idx = nazmc_nir::TypeKind::new(
                    nazmc_nir::TypeKind::PTR_MUT,
                    self.nir.types.ptrs_mut.len(),
                );

                self.nir.types.ptrs_mut.push(typ);

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
            nazmc_ast::Type::RefMut(typ, span) => {
                let typ = self.lower_type(typ);

                let kind_and_idx = nazmc_nir::TypeKind::new(
                    nazmc_nir::TypeKind::REF_MUT,
                    self.nir.types.refs_mut.len(),
                );

                self.nir.types.refs_mut.push(typ);

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
            nazmc_ast::Type::Array(typ, size_expr, span) => {
                let typ = self.lower_type(typ);

                let size = self.lower_expr(size_expr);

                let kind_and_idx = nazmc_nir::TypeKind::new(
                    nazmc_nir::TypeKind::ARRAY,
                    self.nir.types.arrays.len(),
                );

                self.nir
                    .types
                    .arrays
                    .push(nazmc_nir::ArrayType { typ, size });

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
            nazmc_ast::Type::Lambda(params, return_type, span) => {
                let params = params
                    .iter()
                    .map(|typ| self.lower_type(typ))
                    .collect::<ThinVec<_>>();

                let return_type = self.lower_type(&return_type);

                let kind_and_idx = nazmc_nir::TypeKind::new(
                    nazmc_nir::TypeKind::LAMBDA,
                    self.nir.types.lambdas.len(),
                );

                self.nir.types.lambdas.push(nazmc_nir::LambdaType {
                    params,
                    return_type,
                });

                nazmc_nir::Type {
                    kind_and_idx,
                    span: *span,
                }
            }
        }
    }

    fn lower_fn(&mut self, _fn: &nazmc_ast::Fn) {}

    fn lower_expr(&mut self, expr: &nazmc_ast::Expr) -> nazmc_nir::Expr {
        todo!()
    }
}
