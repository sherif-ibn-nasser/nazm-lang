use std::sync::Arc;

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

#[inline]
fn lower_lambda_as_body(lambda: LambdaExpr) -> ast::Scope {
    lower_lambda_stms_and_return_expr(lambda.stms, lambda.last_expr)
}

fn lower_lambda_stms_and_return_expr(
    stms: Vec<ParseResult<Stm>>,
    return_expr: Option<Expr>,
) -> ast::Scope {
    let mut ast_stms = ThinVec::new();

    for stm in stms {
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
        ast_stms.push(stm);
    }

    let return_expr = return_expr.map(|expr| lower_expr(expr));

    ast::Scope {
        stms: ast_stms,
        return_expr,
    }
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
    let mut left = lower_primary_expr(*expr.left);
    // TODO: Shunting-yard algorithm
    left
}

fn lower_primary_expr(primary_expr: PrimaryExpr) -> ast::Expr {
    let expr = match primary_expr.kind {
        PrimaryExprKind::Unary(unary_expr) => lower_unary_expr(unary_expr),
        PrimaryExprKind::Atomic(atomic_expr) => lower_atomic_expr(atomic_expr),
    };

    let expr = lower_post_ops_exprs(expr, primary_expr.post_ops);

    let expr = lower_inner_access_expr(expr, primary_expr.inner_access);

    expr
}

#[inline]
fn lower_inner_access_expr(
    mut on: ast::Expr,
    inner_access_exprs: Vec<InnerAccessExpr>,
) -> ast::Expr {
    for inner_access_expr in inner_access_exprs {
        let name = inner_access_expr.inner.unwrap();

        let name = ast::ASTId {
            span: name.span,
            id: name.data.val,
        };

        let field_expr = ast::Expr {
            span: on.span.merged_with(&name.span),
            kind: ast::ExprKind::Field(Box::new(ast::FieldExpr { on, name })),
        };

        on = lower_post_ops_exprs(field_expr, inner_access_expr.post_ops);
    }
    on
}

fn lower_post_ops_exprs(mut on: ast::Expr, ops: Vec<PostOpExpr>) -> ast::Expr {
    for op in ops {
        match op {
            PostOpExpr::Invoke(paren_expr) => {
                let parens_span = paren_expr
                    .open_delim
                    .span
                    .merged_with(&paren_expr.close_delim.unwrap().span);

                let span = on.span.merged_with(&parens_span);

                let mut args = ThinVec::new();

                if let Some(PunctuatedExpr {
                    first_item,
                    rest_items,
                    trailing_comma: _,
                }) = paren_expr.items
                {
                    let first = lower_expr(first_item.unwrap());
                    args.push(first);
                    for r in rest_items {
                        args.push(lower_expr(r.unwrap().item));
                    }
                }

                let call = ast::CallExpr {
                    on,
                    args,
                    parens_span,
                };

                on = ast::Expr {
                    span,
                    kind: ast::ExprKind::Call(Box::new(call)),
                };
            }
            PostOpExpr::Lambda(lambda_expr) => {
                let parens_span = lambda_expr
                    .open_curly
                    .span
                    .merged_with(&lambda_expr.close_curly.as_ref().unwrap().span);

                let span = on.span.merged_with(&parens_span);

                let mut args = ThinVec::new();

                args.push(lower_lambda_expr(lambda_expr));

                let call = ast::CallExpr {
                    on,
                    args,
                    parens_span,
                };

                on = ast::Expr {
                    span,
                    kind: ast::ExprKind::Call(Box::new(call)),
                };
            }
            PostOpExpr::Index(idx_expr) => {
                let brackets_span = idx_expr
                    .open_bracket
                    .span
                    .merged_with(&idx_expr.close_bracket.unwrap().span);

                let span = on.span.merged_with(&brackets_span);

                let index = lower_expr(idx_expr.expr.unwrap());

                let index = ast::IndexExpr {
                    on,
                    index,
                    brackets_span,
                };

                on = ast::Expr {
                    span,
                    kind: ast::ExprKind::Index(Box::new(index)),
                };
            }
        }
    }
    on
}

fn lower_unary_expr(unary_expr: UnaryExpr) -> ast::Expr {
    let mut expr = lower_atomic_expr(unary_expr.expr.unwrap());

    for op in unary_expr.rest_ops.into_iter().rev() {
        let op_span = op.span;
        let op = lower_unary_op(op.data);

        expr = ast::Expr {
            span: op_span.merged_with(&expr.span),
            kind: ast::ExprKind::UnaryOp(Box::new(ast::UnaryOpExpr { op, op_span, expr })),
        }
    }

    let op_span = unary_expr.first_op.span;
    let op = lower_unary_op(unary_expr.first_op.data);
    ast::Expr {
        span: op_span.merged_with(&expr.span),
        kind: ast::ExprKind::UnaryOp(Box::new(ast::UnaryOpExpr { op, op_span, expr })),
    }
}

fn lower_unary_op(op: UnaryOpToken) -> ast::UnaryOp {
    match op {
        UnaryOpToken::Minus => ast::UnaryOp::Minus,
        UnaryOpToken::LNot => ast::UnaryOp::LNot,
        UnaryOpToken::BNot => ast::UnaryOp::BNot,
        UnaryOpToken::Deref => ast::UnaryOp::Deref,
        UnaryOpToken::Borrow => ast::UnaryOp::Borrow,
        UnaryOpToken::BorrowMut => ast::UnaryOp::BorrowMut,
    }
}

fn lower_atomic_expr(atomic_expr: AtomicExpr) -> ast::Expr {
    match atomic_expr {
        AtomicExpr::Array(array_expr) => lower_array_expr(array_expr),
        AtomicExpr::Paren(paren_expr) => lower_paren_expr(paren_expr),
        AtomicExpr::Struct(struct_expr) => lower_struct_expr(struct_expr),
        AtomicExpr::Lambda(lambda_expr) => lower_lambda_expr(lambda_expr),
        AtomicExpr::When(when_expr) => todo!(),
        AtomicExpr::If(if_expr) => {
            let span_end = if let Some(ref else_) = if_expr.else_cluase {
                &else_
                    .block
                    .as_ref()
                    .unwrap()
                    .close_curly
                    .as_ref()
                    .unwrap()
                    .span
            } else if !if_expr.else_ifs.is_empty() {
                &if_expr
                    .else_ifs
                    .last()
                    .unwrap()
                    .conditional_block
                    .block
                    .as_ref()
                    .unwrap()
                    .close_curly
                    .as_ref()
                    .unwrap()
                    .span
            } else {
                &if_expr
                    .conditional_block
                    .block
                    .as_ref()
                    .unwrap()
                    .close_curly
                    .as_ref()
                    .unwrap()
                    .span
            };

            ast::Expr {
                span: if_expr.if_keyword.span.merged_with(span_end),
                kind: ast::ExprKind::If(Box::new(lower_if_expr(if_expr))),
            }
        }
        AtomicExpr::Path(simple_path) => {
            let path = lower_simple_path(simple_path);

            let span = if path.mod_path.spans.is_empty() {
                path.item.span
            } else {
                path.mod_path
                    .spans
                    .first()
                    .unwrap()
                    .merged_with(&path.item.span)
            };

            ast::Expr {
                span,
                kind: ast::ExprKind::Path(Box::new(path)),
            }
        }
        AtomicExpr::Literal(lit) => ast::Expr {
            span: lit.span,
            kind: ast::ExprKind::Literal(lit.data),
        },
        AtomicExpr::Return(return_expr) => {
            let expr = return_expr.expr.map(|e| Box::new(lower_expr(e)));

            let span = if let Some(e) = expr.as_ref() {
                return_expr.return_keyword.span.merged_with(&e.span)
            } else {
                return_expr.return_keyword.span
            };

            ast::Expr {
                span,
                kind: ast::ExprKind::Return(expr),
            }
        }
        AtomicExpr::Break(break_expr) => {
            let expr = break_expr.expr.map(|e| Box::new(lower_expr(e)));

            let span = if let Some(e) = expr.as_ref() {
                break_expr.break_keyword.span.merged_with(&e.span)
            } else {
                break_expr.break_keyword.span
            };

            ast::Expr {
                span: break_expr.break_keyword.span,
                kind: ast::ExprKind::Break(expr),
            }
        }
        AtomicExpr::Continue(continue_expr) => ast::Expr {
            span: continue_expr.continue_keyword.span,
            kind: ast::ExprKind::Continue,
        },
        AtomicExpr::On(on) => ast::Expr {
            span: on.span,
            kind: ast::ExprKind::On,
        },
    }
}

#[inline]
fn lower_array_expr(array_expr: ArrayExpr) -> ast::Expr {
    let span = array_expr
        .open_bracket
        .span
        .merged_with(&array_expr.close_bracket.unwrap().span);

    if let Some(ArrayExprKind::Elements(ElementsArrayExpr {
        first,
        rest,
        trailing_comma: _,
    })) = array_expr.expr_kind
    {
        let mut elements = ThinVec::new();
        let first = lower_expr(first.unwrap());
        elements.push(first);
        for r in rest {
            elements.push(lower_expr(r.unwrap().item));
        }

        ast::Expr {
            span,
            kind: ast::ExprKind::ArrayElemnts(elements),
        }
    } else if let Some(ArrayExprKind::ExplicitSize(ExplicitSizeArrayExpr {
        repeated_expr,
        semicolon: _,
        size_expr,
    })) = array_expr.expr_kind
    {
        let repeat = lower_expr(repeated_expr.unwrap());
        let size = lower_expr(size_expr.unwrap());
        let array_elements_sized_expr = Box::new(ast::ArrayElementsSizedExpr { repeat, size });
        ast::Expr {
            span,
            kind: ast::ExprKind::ArrayElemntsSized(array_elements_sized_expr),
        }
    } else {
        let elements = ThinVec::new();
        ast::Expr {
            span,
            kind: ast::ExprKind::ArrayElemnts(elements),
        }
    }
}

#[inline]
fn lower_paren_expr(paren_expr: ParenExpr) -> ast::Expr {
    let span = paren_expr
        .open_delim
        .span
        .merged_with(&paren_expr.close_delim.unwrap().span);

    if let Some(PunctuatedExpr {
        first_item,
        rest_items,
        trailing_comma,
    }) = paren_expr.items
    {
        let first = lower_expr(first_item.unwrap());
        if rest_items.is_empty() && trailing_comma.is_none() {
            ast::Expr {
                span,
                kind: ast::ExprKind::Parens(Box::new(first)),
            }
        } else {
            let mut exprs = ThinVec::new();
            exprs.push(first);
            for r in rest_items {
                exprs.push(lower_expr(r.unwrap().item));
            }
            ast::Expr {
                span,
                kind: ast::ExprKind::Tuple(exprs),
            }
        }
    } else {
        ast::Expr {
            span,
            kind: ast::ExprKind::Tuple(ThinVec::new()),
        }
    }
}

#[inline]
fn lower_struct_expr(struct_expr: StructExpr) -> ast::Expr {
    let path = lower_simple_path(struct_expr.path.unwrap());
    if let Some(StructInit::Tuple(tuple_struct)) = struct_expr.init {
        let span = struct_expr
            .dot
            .span
            .merged_with(&tuple_struct.close_delim.unwrap().span);

        let mut args = ThinVec::new();

        if let Some(PunctuatedExpr {
            first_item,
            rest_items,
            trailing_comma: _,
        }) = tuple_struct.items
        {
            let first = lower_expr(first_item.unwrap());
            args.push(first);
            for r in rest_items {
                args.push(lower_expr(r.unwrap().item));
            }
        }

        let tuple_struct = Box::new(ast::TupleStructExpr { path, args });

        ast::Expr {
            span,
            kind: ast::ExprKind::TupleStruct(tuple_struct),
        }
    } else if let Some(StructInit::Fields(fields_struct)) = struct_expr.init {
        let span = struct_expr
            .dot
            .span
            .merged_with(&fields_struct.close_delim.unwrap().span);

        let mut fields = ThinVec::new();

        if let Some(PunctuatedStructFieldInitExpr {
            first_item,
            rest_items,
            trailing_comma: _,
        }) = fields_struct.items
        {
            fn lower_struct_field_expr(e: StructFieldInitExpr) -> (ast::ASTId, ast::Expr) {
                let name = ast::ASTId {
                    span: e.name.span,
                    id: e.name.data.val,
                };

                let expr = if let Some(e) = e.expr {
                    lower_expr(e.expr.unwrap())
                } else {
                    ast::Expr {
                        span: name.span,
                        kind: ast::ExprKind::Path(Box::new(ast::ModPathWithItem {
                            mod_path: ast::ModPath {
                                ids: ThinVec::new(),
                                spans: ThinVec::new(),
                            },
                            item: ast::ASTId {
                                span: name.span,
                                id: Arc::clone(&name.id),
                            },
                        })),
                    }
                };

                (name, expr)
            }

            let first = lower_struct_field_expr(first_item.unwrap());
            fields.push(first);
            for r in rest_items {
                fields.push(lower_struct_field_expr(r.unwrap().item));
            }
        }

        let fields_struct = Box::new(ast::FieldsStructExpr { path, fields });

        ast::Expr {
            span,
            kind: ast::ExprKind::FieldsStruct(fields_struct),
        }
    } else {
        let span = struct_expr.dot.span.merged_with(&path.item.span);

        ast::Expr {
            span,
            kind: ast::ExprKind::UnitStruct(Box::new(path)),
        }
    }
}

#[inline]
fn lower_lambda_expr(lambda_expr: LambdaExpr) -> ast::Expr {
    let span = lambda_expr
        .open_curly
        .span
        .merged_with(&lambda_expr.close_curly.unwrap().span);

    let body = lower_lambda_stms_and_return_expr(lambda_expr.stms, lambda_expr.last_expr);

    let lambda = if let Some(arrow) = lambda_expr.lambda_arrow {
        let mut params = ThinVec::new();

        if let LambdaArrow::WithParams(LambdaParams {
            first,
            rest,
            trailing_comma: _,
            r_arrow: _,
        }) = arrow
        {
            let first = lower_binding(first);
            params.push(first);

            for r in rest {
                params.push(lower_binding(r.item));
            }
        }

        ast::LambdaExpr {
            params: ast::LambdaParams::Explicit(params),
            body,
        }
    } else {
        ast::LambdaExpr {
            params: ast::LambdaParams::Implicit,
            body,
        }
    };

    ast::Expr {
        span,
        kind: ast::ExprKind::Lambda(Box::new(lambda)),
    }
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

fn lower_when_expr(when_expr: WhenExpr) -> ast::Expr {
    todo!()
}
