use thin_vec::ThinVec;

use crate::*;

pub(crate) fn lower_file(file: File) -> ast::File {
    let (imports, star_imports) = lower_imports(file.imports);

    let (unit_structs, tuple_structs, fields_structs, fns) = lower_file_items(file.content.items);

    ast::File {
        imports,
        star_imports,
        unit_structs,
        tuple_structs,
        fields_structs,
        fns,
    }
}

#[inline]
fn lower_imports(
    imports_stms: Vec<ImportStm>,
) -> (ThinVec<ast::ModPathWithItem>, ThinVec<ast::ModPath>) {
    let mut imports = ThinVec::new();
    let mut star_imports = ThinVec::new();

    for import_stm in imports_stms {
        let mut mod_path = ast::ModPath {
            ids: ThinVec::new(),
            spans: ThinVec::new(),
        };

        let mut import_all = false;

        if let Ok(id) = import_stm.top {
            mod_path.ids.push(id.data.val);
            mod_path.spans.push(id.span);
        } else {
            unreachable!()
        };

        if let Ok(s) = import_stm.sec {
            match s.seg.unwrap() {
                syntax::PathSegInImportStm::Id(id) => {
                    mod_path.ids.push(id.data.val);
                    mod_path.spans.push(id.span);
                }
                syntax::PathSegInImportStm::Star(_) => import_all = true,
            }
        } else {
            unreachable!()
        };

        for s in import_stm.segs {
            match s.seg.unwrap() {
                syntax::PathSegInImportStm::Id(id) => {
                    mod_path.ids.push(id.data.val);
                    mod_path.spans.push(id.span);
                }
                syntax::PathSegInImportStm::Star(_) => import_all = true,
            }
        }

        if import_all {
            star_imports.push(mod_path);
        } else {
            let item_id = mod_path.ids.pop().unwrap();
            let item_span = mod_path.spans.pop().unwrap();

            imports.push(ast::ModPathWithItem {
                mod_path,
                item: ast::ASTId {
                    span: item_span,
                    id: item_id,
                },
            });
        }
    }
    (imports, star_imports)
}

#[inline]
fn lower_file_items(
    file_items: Vec<ParseResult<FileItem>>,
) -> (
    ThinVec<ast::UnitStruct>,
    ThinVec<ast::TupleStruct>,
    ThinVec<ast::FieldsStruct>,
    ThinVec<ast::Fn>,
) {
    let mut unit_structs = ThinVec::new();
    let mut tuple_structs = ThinVec::new();
    let mut fields_structs = ThinVec::new();
    let mut fns = ThinVec::new();

    for file_item in file_items {
        let (item, vis) = match file_item.unwrap() {
            syntax::FileItem::WithVisModifier(item_with_vis) => {
                let Ok(item) = item_with_vis.item else {
                    unreachable!()
                };

                (
                    item,
                    match item_with_vis.visibility.data {
                        syntax::VisModifierToken::Public => ast::VisModifier::Public,
                        syntax::VisModifierToken::Private => ast::VisModifier::Private,
                    },
                )
            }
            syntax::FileItem::WithoutModifier(item) => (item, ast::VisModifier::Default),
        };

        match item {
            Item::Struct(s) => {
                let name = s.name.unwrap();
                let name = ast::ASTId {
                    span: name.span,
                    id: name.data.val,
                };

                match s.kind.unwrap() {
                    StructKind::Unit(_) => {
                        unit_structs.push(ast::UnitStruct { vis, name });
                    }
                    StructKind::Tuple(tuple_struct_fields) => {
                        let mut types = ThinVec::new();

                        if let Some(PunctuatedTupleStructField {
                            first_item,
                            rest_items,
                            trailing_comma: _,
                        }) = tuple_struct_fields.items
                        {
                            let first = lower_tuple_struct_field(first_item.unwrap());
                            types.push(first);

                            for r in rest_items {
                                let typ = lower_tuple_struct_field(r.unwrap().item);
                                types.push(typ);
                            }
                        }

                        tuple_structs.push(ast::TupleStruct { vis, name, types });
                    }
                    StructKind::Fields(struct_fields) => {
                        let mut fields = ThinVec::new();

                        if let Some(PunctuatedStructField {
                            first_item,
                            rest_items,
                            trailing_comma: _,
                        }) = struct_fields.items
                        {
                            let first = lower_struct_field(first_item.unwrap());
                            fields.push(first);

                            for r in rest_items {
                                let field = lower_struct_field(r.unwrap().item);
                                fields.push(field);
                            }
                        }
                        fields_structs.push(ast::FieldsStruct { vis, name, fields });
                    }
                }
            }
            Item::Fn(f) => {
                let name = f.name.unwrap();
                let name = ast::ASTId {
                    span: name.span,
                    id: name.data.val,
                };

                let mut params = ThinVec::new();

                if let Some(PunctuatedFnParam {
                    first_item,
                    rest_items,
                    trailing_comma: _,
                }) = f.params_decl.unwrap().items
                {
                    let first = lower_fn_param(first_item.unwrap());
                    params.push(first);

                    for r in rest_items {
                        let param = lower_fn_param(r.unwrap().item);
                        params.push(param);
                    }
                }

                let return_type = if let Some(ColonWithType { colon: _, typ }) = f.return_type {
                    lower_type(typ.unwrap())
                } else {
                    ast::Type::Unit(None)
                };

                let body = lower_lambda_as_body(f.body.unwrap());

                fns.push(ast::Fn {
                    vis,
                    name,
                    params,
                    return_type,
                    body,
                });
            }
        }
    }

    (unit_structs, tuple_structs, fields_structs, fns)
}

fn lower_tuple_struct_field(field: TupleStructField) -> (ast::VisModifier, ast::Type) {
    let vis = match field.visibility {
        Some(Terminal {
            data: syntax::VisModifierToken::Public,
            ..
        }) => ast::VisModifier::Public,
        Some(Terminal {
            data: syntax::VisModifierToken::Private,
            ..
        }) => ast::VisModifier::Private,
        None => ast::VisModifier::Default,
    };

    let typ = lower_type(field.typ.unwrap());

    (vis, typ)
}

fn lower_struct_field(field: StructField) -> (ast::VisModifier, ast::ASTId, ast::Type) {
    let vis = match field.visibility {
        Some(Terminal {
            data: syntax::VisModifierToken::Public,
            ..
        }) => ast::VisModifier::Public,
        Some(Terminal {
            data: syntax::VisModifierToken::Private,
            ..
        }) => ast::VisModifier::Private,
        None => ast::VisModifier::Default,
    };

    let name = ast::ASTId {
        span: field.name.span,
        id: field.name.data.val,
    };

    let typ = lower_type(field.typ.unwrap().typ.unwrap());

    (vis, name, typ)
}

fn lower_fn_param(param: FnParam) -> (ast::ASTId, ast::Type) {
    let name = ast::ASTId {
        span: param.name.span,
        id: param.name.data.val,
    };

    let typ = lower_type(param.typ.unwrap().typ.unwrap());

    (name, typ)
}

fn lower_type(typ: Type) -> ast::Type {
    match typ {
        Type::Path(simple_path) => ast::Type::Path(lower_simple_path(simple_path)),
        Type::Ptr(ptr_type) => {
            let underlying_typ = Box::new(lower_type(ptr_type.typ.unwrap()));
            let star_span = ptr_type.star.span;
            if let Some(mut_) = ptr_type.mut_keyword {
                ast::Type::PtrMut(underlying_typ, star_span.merged_with(&mut_.span))
            } else {
                ast::Type::Ptr(underlying_typ, star_span)
            }
        }
        Type::Ref(ref_type) => {
            let underlying_typ = Box::new(lower_type(ref_type.typ.unwrap()));
            let hash_span = ref_type.hash.span;
            if let Some(mut_) = ref_type.mut_keyword {
                ast::Type::RefMut(underlying_typ, hash_span.merged_with(&mut_.span))
            } else {
                ast::Type::Ref(underlying_typ, hash_span)
            }
        }
        Type::Slice(slice_type) => {
            let underlying_typ = Box::new(lower_type(slice_type.typ.unwrap()));
            let brackets_span = slice_type
                .open_bracket
                .span
                .merged_with(&slice_type.close_bracket.unwrap().span);
            if let Some(array_size) = slice_type.array_size {
                let size_expr = Box::new(lower_expr(array_size.expr.unwrap()));
                ast::Type::Array(underlying_typ, size_expr, brackets_span)
            } else {
                ast::Type::Slice(underlying_typ, brackets_span)
            }
        }
        Type::Paren(paren_type) => {
            let mut types = ThinVec::new();

            let mut trailing_comma_in_types = false;

            if let Some(PunctuatedType {
                first_item,
                rest_items,
                trailing_comma,
            }) = paren_type.tuple.items
            {
                let first = lower_type(first_item.unwrap());
                types.push(first);
                for r in rest_items {
                    let r = lower_type(r.unwrap().item);
                    types.push(r);
                }

                trailing_comma_in_types = trailing_comma.is_some();
            }

            if let Some(lambda_type) = paren_type.lambda {
                let return_type = Box::new(lower_type(lambda_type.typ.unwrap()));

                ast::Type::Lambda(types, return_type)
            } else {
                let parens_span = paren_type
                    .tuple
                    .open_delim
                    .span
                    .merged_with(&paren_type.tuple.close_delim.unwrap().span);

                if types.is_empty() {
                    ast::Type::Unit(Some(parens_span))
                } else if !trailing_comma_in_types && types.len() == 1 {
                    ast::Type::Paren(Box::new(types.pop().unwrap()), parens_span)
                } else {
                    ast::Type::Tuple(types, parens_span)
                }
            }
        }
    }
}

fn lower_simple_path(mut simple_path: SimplePath) -> ast::ModPathWithItem {
    let mut mod_path = ast::ModPath {
        ids: ThinVec::new(),
        spans: ThinVec::new(),
    };

    if simple_path.inners.is_empty() {
        let item = ast::ASTId {
            span: simple_path.top.span,
            id: simple_path.top.data.val,
        };
        ast::ModPathWithItem { mod_path, item }
    } else {
        let item = simple_path.inners.pop().unwrap().inner.unwrap();

        let item = ast::ASTId {
            span: item.span,
            id: item.data.val,
        };

        for inner in simple_path.inners {
            let inner = inner.inner.unwrap();
            mod_path.ids.push(inner.data.val);
            mod_path.spans.push(inner.span);
        }

        ast::ModPathWithItem { mod_path, item }
    }
}

fn lower_lambda_as_body(lambda: LambdaExpr) -> ast::Scope {
    let mut stms = ThinVec::new();

    for stm in lambda.stms {
        let stm = match stm.unwrap() {
            Stm::Semicolon(_) => continue,
            Stm::Let(let_stm) => {
                let binding = lower_binding(let_stm.binding.unwrap());

                let assign = let_stm
                    .let_assign
                    .map(|a| Box::new(lower_expr(a.expr.unwrap())));

                let let_stm_ = Box::new(ast::LetStm { binding, assign });

                if let_stm.mut_keyword.is_some() {
                    ast::Stm::LetMut(let_stm_)
                } else {
                    ast::Stm::Let(let_stm_)
                }
            }
            Stm::While(while_stm) => ast::Stm::While(Box::new((
                lower_expr(while_stm.conditional_block.condition.unwrap()),
                lower_lambda_as_body(while_stm.conditional_block.block.unwrap()),
            ))),
            Stm::If(if_expr) => ast::Stm::If(Box::new(lower_if_expr(if_expr))),
            Stm::When(when_expr) => todo!(),
            Stm::Expr(stm) => ast::Stm::Expr(Box::new(lower_expr(stm.expr))),
        };
        stms.push(stm);
    }

    let return_expr = lambda.last_expr.map(|expr| lower_expr(expr));

    ast::Scope { stms, return_expr }
}

fn lower_binding(binding: Binding) -> ast::Binding {
    let kind = lower_binding_kind(binding.kind);

    let typ = if let Some(ColonWithType { colon: _, typ }) = binding.typ {
        lower_type(typ.unwrap())
    } else {
        ast::Type::Unit(None)
    };

    ast::Binding { kind, typ }
}

fn lower_binding_kind(kind: BindingKind) -> ast::BindingKind {
    match kind {
        BindingKind::Id(id) => ast::BindingKind::Id(ast::ASTId {
            span: id.span,
            id: id.data.val,
        }),
        BindingKind::Destructed(destructed_tuple) => {
            let span = destructed_tuple
                .open_delim
                .span
                .merged_with(&destructed_tuple.close_delim.unwrap().span);

            let mut destructed_bindings = ThinVec::new();

            if let Some(PunctuatedBindingKind {
                first_item,
                rest_items,
                trailing_comma,
            }) = destructed_tuple.items
            {
                let first = lower_binding_kind(first_item.unwrap());

                if trailing_comma.is_none() && rest_items.is_empty() {
                    return first;
                }

                destructed_bindings.push(first);

                for r in rest_items {
                    let r = lower_binding_kind(r.unwrap().item);
                    destructed_bindings.push(r);
                }
            }
            ast::BindingKind::Tuple(destructed_bindings, span)
        }
    }
}

fn lower_expr(expr: Expr) -> ast::Expr {
    todo!()
}

fn lower_if_expr(if_expr: IfExpr) -> ast::IfExpr {
    let if_condition = lower_expr(if_expr.conditional_block.condition.unwrap());
    let if_body = lower_lambda_as_body(if_expr.conditional_block.block.unwrap());
    let if_ = (if_condition, if_body);

    let mut else_ifs = ThinVec::new();

    for else_if in if_expr.else_ifs {
        let condition = lower_expr(else_if.conditional_block.condition.unwrap());
        let body = lower_lambda_as_body(else_if.conditional_block.block.unwrap());
        else_ifs.push((condition, body));
    }

    let else_ = if_expr
        .else_cluase
        .map(|e| Box::new(lower_lambda_as_body(e.block.unwrap())));

    ast::IfExpr {
        if_,
        else_ifs,
        else_,
    }
}

fn lower_when_expr(if_expr: WhenExpr) -> ast::Expr {
    todo!()
}
