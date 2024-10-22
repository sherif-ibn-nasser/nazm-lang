use crate::*;
use nazmc_ast;
use nazmc_data_pool::PoolIdx;
use thin_vec::ThinVec;

pub(crate) fn lower_file(file: File) -> nazmc_ast::File {
    let (imports, star_imports) = lower_imports(file.imports);

    let items = lower_file_items(file.content.items);

    nazmc_ast::File {
        imports,
        star_imports,
        items,
    }
}

#[inline]
fn lower_imports(
    imports_stms: Vec<ImportStm>,
) -> (
    ThinVec<(nazmc_ast::PkgPathWithItem, nazmc_ast::ASTId)>,
    ThinVec<nazmc_ast::PkgPath>,
) {
    let mut imports = ThinVec::new();
    let mut star_imports = ThinVec::new();

    for import_stm in imports_stms {
        let mut mod_path = nazmc_ast::PkgPath {
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

            let import = nazmc_ast::PkgPathWithItem {
                pkg_path: mod_path,
                item: nazmc_ast::ASTId {
                    span: item_span,
                    id: item_id,
                },
            };

            let alias = nazmc_ast::ASTId {
                span: item_span,
                id: item_id,
            };

            imports.push((import, alias));
        }
    }
    (imports, star_imports)
}

#[inline]
fn lower_file_items(file_items: Vec<ParseResult<FileItem>>) -> ThinVec<nazmc_ast::Item> {
    let mut items = ThinVec::new();

    for file_item in file_items {
        let (item, vis) = match file_item.unwrap() {
            syntax::FileItem::WithVisModifier(item_with_vis) => {
                let Ok(item) = item_with_vis.item else {
                    unreachable!()
                };

                (
                    item,
                    match item_with_vis.visibility.data {
                        syntax::VisModifierToken::Public => nazmc_ast::VisModifier::Public,
                        syntax::VisModifierToken::Private => nazmc_ast::VisModifier::Private,
                    },
                )
            }
            syntax::FileItem::WithoutModifier(item) => (item, nazmc_ast::VisModifier::Default),
        };

        match item {
            Item::Struct(s) => {
                let name = s.name.unwrap();
                let name = nazmc_ast::ASTId {
                    span: name.span,
                    id: name.data.val,
                };

                match s.kind.unwrap() {
                    StructKind::Unit(_) => {
                        items.push(nazmc_ast::Item {
                            name,
                            vis,
                            kind: nazmc_ast::ItemKind::UnitStruct,
                        });
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
                        items.push(nazmc_ast::Item {
                            name,
                            vis,
                            kind: nazmc_ast::ItemKind::TupleStruct(nazmc_ast::TupleStruct {
                                types,
                            }),
                        });
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
                        items.push(nazmc_ast::Item {
                            name,
                            vis,
                            kind: nazmc_ast::ItemKind::FieldsStruct(nazmc_ast::FieldsStruct {
                                fields,
                            }),
                        });
                    }
                }
            }
            Item::Fn(f) => {
                let name = f.name.unwrap();
                let name = nazmc_ast::ASTId {
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
                    nazmc_ast::Type::Unit(None)
                };

                let body = lower_lambda_as_body(f.body.unwrap());

                items.push(nazmc_ast::Item {
                    name,
                    vis,
                    kind: nazmc_ast::ItemKind::Fn(nazmc_ast::Fn {
                        params,
                        return_type,
                        body,
                    }),
                });
            }
        }
    }
    items
}

fn lower_tuple_struct_field(field: TupleStructField) -> (nazmc_ast::VisModifier, nazmc_ast::Type) {
    let vis = match field.visibility {
        Some(Terminal {
            data: syntax::VisModifierToken::Public,
            ..
        }) => nazmc_ast::VisModifier::Public,
        Some(Terminal {
            data: syntax::VisModifierToken::Private,
            ..
        }) => nazmc_ast::VisModifier::Private,
        None => nazmc_ast::VisModifier::Default,
    };

    let typ = lower_type(field.typ.unwrap());

    (vis, typ)
}

fn lower_struct_field(
    field: StructField,
) -> (nazmc_ast::VisModifier, nazmc_ast::ASTId, nazmc_ast::Type) {
    let vis = match field.visibility {
        Some(Terminal {
            data: syntax::VisModifierToken::Public,
            ..
        }) => nazmc_ast::VisModifier::Public,
        Some(Terminal {
            data: syntax::VisModifierToken::Private,
            ..
        }) => nazmc_ast::VisModifier::Private,
        None => nazmc_ast::VisModifier::Default,
    };

    let name = nazmc_ast::ASTId {
        span: field.name.span,
        id: field.name.data.val,
    };

    let typ = lower_type(field.typ.unwrap().typ.unwrap());

    (vis, name, typ)
}

fn lower_fn_param(param: FnParam) -> (nazmc_ast::ASTId, nazmc_ast::Type) {
    let name = nazmc_ast::ASTId {
        span: param.name.span,
        id: param.name.data.val,
    };

    let typ = lower_type(param.typ.unwrap().typ.unwrap());

    (name, typ)
}

fn lower_type(typ: Type) -> nazmc_ast::Type {
    match typ {
        Type::Path(simple_path) => nazmc_ast::Type::Path(lower_simple_path(simple_path)),
        Type::Ptr(ptr_type) => {
            let underlying_typ = Box::new(lower_type(ptr_type.typ.unwrap()));
            let star_span = ptr_type.star.span;
            if let Some(mut_) = ptr_type.mut_keyword {
                nazmc_ast::Type::PtrMut(underlying_typ, star_span.merged_with(&mut_.span))
            } else {
                nazmc_ast::Type::Ptr(underlying_typ, star_span)
            }
        }
        Type::Ref(ref_type) => {
            let underlying_typ = Box::new(lower_type(ref_type.typ.unwrap()));
            let hash_span = ref_type.hash.span;
            if let Some(mut_) = ref_type.mut_keyword {
                nazmc_ast::Type::RefMut(underlying_typ, hash_span.merged_with(&mut_.span))
            } else {
                nazmc_ast::Type::Ref(underlying_typ, hash_span)
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
                nazmc_ast::Type::Array(underlying_typ, size_expr, brackets_span)
            } else {
                nazmc_ast::Type::Slice(underlying_typ, brackets_span)
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

                nazmc_ast::Type::Lambda(types, return_type)
            } else {
                let parens_span = paren_type
                    .tuple
                    .open_delim
                    .span
                    .merged_with(&paren_type.tuple.close_delim.unwrap().span);

                if types.is_empty() {
                    nazmc_ast::Type::Unit(Some(parens_span))
                } else if !trailing_comma_in_types && types.len() == 1 {
                    nazmc_ast::Type::Paren(Box::new(types.pop().unwrap()), parens_span)
                } else {
                    nazmc_ast::Type::Tuple(types, parens_span)
                }
            }
        }
    }
}

fn lower_simple_path(mut simple_path: SimplePath) -> nazmc_ast::PkgPathWithItem {
    let mut mod_path = nazmc_ast::PkgPath {
        ids: ThinVec::new(),
        spans: ThinVec::new(),
    };

    if simple_path.inners.is_empty() {
        let item = nazmc_ast::ASTId {
            span: simple_path.top.span,
            id: simple_path.top.data.val,
        };
        nazmc_ast::PkgPathWithItem {
            pkg_path: mod_path,
            item,
        }
    } else {
        let item = simple_path.inners.pop().unwrap().inner.unwrap();

        let item = nazmc_ast::ASTId {
            span: item.span,
            id: item.data.val,
        };

        for inner in simple_path.inners {
            let inner = inner.inner.unwrap();
            mod_path.ids.push(inner.data.val);
            mod_path.spans.push(inner.span);
        }

        nazmc_ast::PkgPathWithItem {
            pkg_path: mod_path,
            item,
        }
    }
}

#[inline]
fn lower_lambda_as_body(lambda: LambdaExpr) -> nazmc_ast::Scope {
    lower_lambda_stms_and_return_expr(lambda.stms, lambda.last_expr)
}

fn lower_lambda_stms_and_return_expr(
    stms: Vec<ParseResult<Stm>>,
    return_expr: Option<Expr>,
) -> nazmc_ast::Scope {
    let mut ast_stms = ThinVec::new();

    for stm in stms {
        let stm = match stm.unwrap() {
            Stm::Semicolon(_) => continue,
            Stm::Let(let_stm) => {
                let binding = lower_binding(let_stm.binding.unwrap());

                let assign = let_stm
                    .let_assign
                    .map(|a| Box::new(lower_expr(a.expr.unwrap())));

                let let_stm_ = Box::new(nazmc_ast::LetStm { binding, assign });

                if let_stm.mut_keyword.is_some() {
                    nazmc_ast::Stm::LetMut(let_stm_)
                } else {
                    nazmc_ast::Stm::Let(let_stm_)
                }
            }
            Stm::While(while_stm) => nazmc_ast::Stm::While(Box::new((
                lower_expr(while_stm.conditional_block.condition.unwrap()),
                lower_lambda_as_body(while_stm.conditional_block.block.unwrap()),
            ))),
            Stm::If(if_expr) => nazmc_ast::Stm::If(Box::new(lower_if_expr(if_expr))),
            Stm::When(_when_expr) => todo!(),
            Stm::Expr(stm) => nazmc_ast::Stm::Expr(Box::new(lower_expr(stm.expr))),
        };
        ast_stms.push(stm);
    }

    let return_expr = return_expr.map(|expr| lower_expr(expr));

    nazmc_ast::Scope {
        stms: ast_stms,
        return_expr,
    }
}

fn lower_binding(binding: Binding) -> nazmc_ast::Binding {
    let kind = lower_binding_kind(binding.kind);

    let typ = binding.typ.map(|t| lower_type(t.typ.unwrap()));

    nazmc_ast::Binding { kind, typ }
}

fn lower_binding_kind(kind: BindingKind) -> nazmc_ast::BindingKind {
    match kind {
        BindingKind::Id(id) => nazmc_ast::BindingKind::Id(nazmc_ast::ASTId {
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
            nazmc_ast::BindingKind::Tuple(destructed_bindings, span)
        }
    }
}

fn lower_expr(expr: Expr) -> nazmc_ast::Expr {
    let left = lower_primary_expr(*expr.left);
    let mut ops_stack = ThinVec::new();
    let mut expr_stack = vec![left]; // Stack to keep track of expressions

    // Shunting-yard algorithm
    for b in expr.bin {
        let right = lower_primary_expr(b.right.unwrap());
        let op = lower_bin_op(b.op.data);
        let op_span_cursor = b.op.span.start;

        // Pop operators from the stack while they have higher or equal precedence
        while let Some((last_op, _)) = ops_stack.last() {
            if get_precendence(&op) > get_precendence(last_op) {
                break;
            }

            let (last_op, last_op_span_cursor) = ops_stack.pop().unwrap();
            let right_expr = expr_stack.pop().unwrap();
            let left_expr = expr_stack.pop().unwrap();

            // Combine left and right expressions using the last operator
            let combined_expr = nazmc_ast::Expr {
                span: left_expr.span.merged_with(&right_expr.span),
                kind: nazmc_ast::ExprKind::BinaryOp(Box::new(nazmc_ast::BinaryOpExpr {
                    op: last_op,
                    op_span_cursor: last_op_span_cursor,
                    left: left_expr,
                    right: right_expr,
                })),
            };

            expr_stack.push(combined_expr);
        }

        // Push the current operator and the right-hand expression onto the stacks
        ops_stack.push((op, op_span_cursor));
        expr_stack.push(right);
    }

    // Apply remaining operators in the stack
    while let Some((last_op, last_op_span_cursor)) = ops_stack.pop() {
        let right_expr = expr_stack.pop().unwrap();
        let left_expr = expr_stack.pop().unwrap();

        // Combine left and right expressions using the remaining operators
        let combined_expr = nazmc_ast::Expr {
            span: left_expr.span.merged_with(&right_expr.span),
            kind: nazmc_ast::ExprKind::BinaryOp(Box::new(nazmc_ast::BinaryOpExpr {
                op: last_op,
                op_span_cursor: last_op_span_cursor,
                left: left_expr,
                right: right_expr,
            })),
        };

        expr_stack.push(combined_expr);
    }

    // Return the final expression
    expr_stack.pop().unwrap()
}

#[inline]
fn get_precendence(op: &nazmc_ast::BinOp) -> u8 {
    match op {
        nazmc_ast::BinOp::Assign
        | nazmc_ast::BinOp::PlusAssign
        | nazmc_ast::BinOp::MinusAssign
        | nazmc_ast::BinOp::TimesAssign
        | nazmc_ast::BinOp::DivAssign
        | nazmc_ast::BinOp::ModAssign
        | nazmc_ast::BinOp::BAndAssign
        | nazmc_ast::BinOp::BOrAssign
        | nazmc_ast::BinOp::XorAssign
        | nazmc_ast::BinOp::ShlAssign
        | nazmc_ast::BinOp::ShrAssign => 0, // Assignments have the lowest precedence
        nazmc_ast::BinOp::LOr => 1,
        nazmc_ast::BinOp::LAnd => 2,
        nazmc_ast::BinOp::EqualEqual | nazmc_ast::BinOp::NotEqual => 3,
        nazmc_ast::BinOp::GE
        | nazmc_ast::BinOp::GT
        | nazmc_ast::BinOp::LE
        | nazmc_ast::BinOp::LT => 4,
        nazmc_ast::BinOp::OpenOpenRange
        | nazmc_ast::BinOp::CloseOpenRange
        | nazmc_ast::BinOp::OpenCloseRange
        | nazmc_ast::BinOp::CloseCloseRange => 5,
        nazmc_ast::BinOp::BOr => 6,
        nazmc_ast::BinOp::Xor => 7,
        nazmc_ast::BinOp::BAnd => 8,
        nazmc_ast::BinOp::Shl | nazmc_ast::BinOp::Shr => 9,
        nazmc_ast::BinOp::Plus | nazmc_ast::BinOp::Minus => 10,
        nazmc_ast::BinOp::Times | nazmc_ast::BinOp::Div | nazmc_ast::BinOp::Mod => 11,
    }
}

#[inline]
fn lower_bin_op(op: BinOpToken) -> nazmc_ast::BinOp {
    match op {
        BinOpToken::LOr => nazmc_ast::BinOp::LOr,
        BinOpToken::LAnd => nazmc_ast::BinOp::LAnd,
        BinOpToken::EqualEqual => nazmc_ast::BinOp::EqualEqual,
        BinOpToken::NotEqual => nazmc_ast::BinOp::NotEqual,
        BinOpToken::GE => nazmc_ast::BinOp::GE,
        BinOpToken::GT => nazmc_ast::BinOp::GT,
        BinOpToken::LE => nazmc_ast::BinOp::LE,
        BinOpToken::LT => nazmc_ast::BinOp::LT,
        BinOpToken::OpenOpenRange => nazmc_ast::BinOp::OpenOpenRange,
        BinOpToken::CloseOpenRange => nazmc_ast::BinOp::CloseOpenRange,
        BinOpToken::OpenCloseRange => nazmc_ast::BinOp::OpenCloseRange,
        BinOpToken::CloseCloseRange => nazmc_ast::BinOp::CloseCloseRange,
        BinOpToken::BOr => nazmc_ast::BinOp::BOr,
        BinOpToken::Xor => nazmc_ast::BinOp::Xor,
        BinOpToken::BAnd => nazmc_ast::BinOp::BAnd,
        BinOpToken::Shr => nazmc_ast::BinOp::Shr,
        BinOpToken::Shl => nazmc_ast::BinOp::Shl,
        BinOpToken::Plus => nazmc_ast::BinOp::Plus,
        BinOpToken::Minus => nazmc_ast::BinOp::Minus,
        BinOpToken::Times => nazmc_ast::BinOp::Times,
        BinOpToken::Div => nazmc_ast::BinOp::Div,
        BinOpToken::Mod => nazmc_ast::BinOp::Mod,
        BinOpToken::Assign => nazmc_ast::BinOp::Assign,
        BinOpToken::PlusAssign => nazmc_ast::BinOp::PlusAssign,
        BinOpToken::MinusAssign => nazmc_ast::BinOp::MinusAssign,
        BinOpToken::TimesAssign => nazmc_ast::BinOp::TimesAssign,
        BinOpToken::DivAssign => nazmc_ast::BinOp::DivAssign,
        BinOpToken::ModAssign => nazmc_ast::BinOp::ModAssign,
        BinOpToken::BAndAssign => nazmc_ast::BinOp::BAndAssign,
        BinOpToken::BOrAssign => nazmc_ast::BinOp::BOrAssign,
        BinOpToken::XorAssign => nazmc_ast::BinOp::XorAssign,
        BinOpToken::ShlAssign => nazmc_ast::BinOp::ShlAssign,
        BinOpToken::ShrAssign => nazmc_ast::BinOp::ShrAssign,
    }
}

fn lower_primary_expr(primary_expr: PrimaryExpr) -> nazmc_ast::Expr {
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
    mut on: nazmc_ast::Expr,
    inner_access_exprs: Vec<InnerAccessExpr>,
) -> nazmc_ast::Expr {
    for inner_access_expr in inner_access_exprs {
        let name = inner_access_expr.inner.unwrap();

        let name = nazmc_ast::ASTId {
            span: name.span,
            id: name.data.val,
        };

        let field_expr = nazmc_ast::Expr {
            span: on.span.merged_with(&name.span),
            kind: nazmc_ast::ExprKind::Field(Box::new(nazmc_ast::FieldExpr { on, name })),
        };

        on = lower_post_ops_exprs(field_expr, inner_access_expr.post_ops);
    }
    on
}

fn lower_post_ops_exprs(mut on: nazmc_ast::Expr, ops: Vec<PostOpExpr>) -> nazmc_ast::Expr {
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

                let call = nazmc_ast::CallExpr {
                    on,
                    args,
                    parens_span,
                };

                on = nazmc_ast::Expr {
                    span,
                    kind: nazmc_ast::ExprKind::Call(Box::new(call)),
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

                let call = nazmc_ast::CallExpr {
                    on,
                    args,
                    parens_span,
                };

                on = nazmc_ast::Expr {
                    span,
                    kind: nazmc_ast::ExprKind::Call(Box::new(call)),
                };
            }
            PostOpExpr::Index(idx_expr) => {
                let brackets_span = idx_expr
                    .open_bracket
                    .span
                    .merged_with(&idx_expr.close_bracket.unwrap().span);

                let span = on.span.merged_with(&brackets_span);

                let index = lower_expr(idx_expr.expr.unwrap());

                let index = nazmc_ast::IndexExpr {
                    on,
                    index,
                    brackets_span,
                };

                on = nazmc_ast::Expr {
                    span,
                    kind: nazmc_ast::ExprKind::Index(Box::new(index)),
                };
            }
        }
    }
    on
}

fn lower_unary_expr(unary_expr: UnaryExpr) -> nazmc_ast::Expr {
    let mut expr = lower_atomic_expr(unary_expr.expr.unwrap());

    for op in unary_expr.rest_ops.into_iter().rev() {
        let op_span = op.span;
        let op = lower_unary_op(op.data);

        expr = nazmc_ast::Expr {
            span: op_span.merged_with(&expr.span),
            kind: nazmc_ast::ExprKind::UnaryOp(Box::new(nazmc_ast::UnaryOpExpr {
                op,
                op_span,
                expr,
            })),
        }
    }

    let op_span = unary_expr.first_op.span;
    let op = lower_unary_op(unary_expr.first_op.data);
    nazmc_ast::Expr {
        span: op_span.merged_with(&expr.span),
        kind: nazmc_ast::ExprKind::UnaryOp(Box::new(nazmc_ast::UnaryOpExpr { op, op_span, expr })),
    }
}

fn lower_unary_op(op: UnaryOpToken) -> nazmc_ast::UnaryOp {
    match op {
        UnaryOpToken::Minus => nazmc_ast::UnaryOp::Minus,
        UnaryOpToken::LNot => nazmc_ast::UnaryOp::LNot,
        UnaryOpToken::BNot => nazmc_ast::UnaryOp::BNot,
        UnaryOpToken::Deref => nazmc_ast::UnaryOp::Deref,
        UnaryOpToken::Borrow => nazmc_ast::UnaryOp::Borrow,
        UnaryOpToken::BorrowMut => nazmc_ast::UnaryOp::BorrowMut,
    }
}

fn lower_atomic_expr(atomic_expr: AtomicExpr) -> nazmc_ast::Expr {
    match atomic_expr {
        AtomicExpr::Array(array_expr) => lower_array_expr(array_expr),
        AtomicExpr::Paren(paren_expr) => lower_paren_expr(paren_expr),
        AtomicExpr::Struct(struct_expr) => lower_struct_expr(struct_expr),
        AtomicExpr::Lambda(lambda_expr) => lower_lambda_expr(lambda_expr),
        AtomicExpr::When(when_expr) => lower_when_expr(when_expr),
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

            nazmc_ast::Expr {
                span: if_expr.if_keyword.span.merged_with(span_end),
                kind: nazmc_ast::ExprKind::If(Box::new(lower_if_expr(if_expr))),
            }
        }
        AtomicExpr::Path(simple_path) => {
            let path = lower_simple_path(simple_path);

            let span = if path.pkg_path.spans.is_empty() {
                path.item.span
            } else {
                path.pkg_path
                    .spans
                    .first()
                    .unwrap()
                    .merged_with(&path.item.span)
            };

            nazmc_ast::Expr {
                span,
                kind: nazmc_ast::ExprKind::Path(Box::new(path)),
            }
        }
        AtomicExpr::Literal(lit) => {
            let literal_expr = match lit.data {
                LiteralKind::Str(pool_idx) => nazmc_ast::LiteralExpr::Str(pool_idx),
                LiteralKind::Char(ch) => nazmc_ast::LiteralExpr::Char(ch),
                LiteralKind::Bool(b) => nazmc_ast::LiteralExpr::Bool(b),
                LiteralKind::Num(num_kind) => {
                    let num_kind = match num_kind {
                        NumKind::F4(f4) => nazmc_ast::NumKind::F4(f4),
                        NumKind::F8(f8) => nazmc_ast::NumKind::F8(f8),
                        NumKind::I(i) => nazmc_ast::NumKind::I(i),
                        NumKind::I1(i1) => nazmc_ast::NumKind::I1(i1),
                        NumKind::I2(i2) => nazmc_ast::NumKind::I2(i2),
                        NumKind::I4(i4) => nazmc_ast::NumKind::I4(i4),
                        NumKind::I8(i8) => nazmc_ast::NumKind::I8(i8),
                        NumKind::U(u) => nazmc_ast::NumKind::U(u),
                        NumKind::U1(u1) => nazmc_ast::NumKind::U1(u1),
                        NumKind::U2(u2) => nazmc_ast::NumKind::U2(u2),
                        NumKind::U4(u4) => nazmc_ast::NumKind::U4(u4),
                        NumKind::U8(u8) => nazmc_ast::NumKind::U8(u8),
                        NumKind::UnspecifiedInt(i) => nazmc_ast::NumKind::UnspecifiedInt(i),
                        NumKind::UnspecifiedFloat(f) => nazmc_ast::NumKind::UnspecifiedFloat(f),
                    };
                    nazmc_ast::LiteralExpr::Num(num_kind)
                }
            };
            nazmc_ast::Expr {
                span: lit.span,
                kind: nazmc_ast::ExprKind::Literal(literal_expr),
            }
        }
        AtomicExpr::Return(return_expr) => {
            let expr = return_expr.expr.map(|e| Box::new(lower_expr(e)));

            let span = if let Some(e) = expr.as_ref() {
                return_expr.return_keyword.span.merged_with(&e.span)
            } else {
                return_expr.return_keyword.span
            };

            nazmc_ast::Expr {
                span,
                kind: nazmc_ast::ExprKind::Return(expr),
            }
        }
        AtomicExpr::Break(break_expr) => {
            let expr = break_expr.expr.map(|e| Box::new(lower_expr(e)));

            let span = if let Some(e) = expr.as_ref() {
                break_expr.break_keyword.span.merged_with(&e.span)
            } else {
                break_expr.break_keyword.span
            };

            nazmc_ast::Expr {
                span,
                kind: nazmc_ast::ExprKind::Break(expr),
            }
        }
        AtomicExpr::Continue(continue_expr) => nazmc_ast::Expr {
            span: continue_expr.continue_keyword.span,
            kind: nazmc_ast::ExprKind::Continue,
        },
        AtomicExpr::On(on) => nazmc_ast::Expr {
            span: on.span,
            kind: nazmc_ast::ExprKind::On,
        },
    }
}

#[inline]
fn lower_array_expr(array_expr: ArrayExpr) -> nazmc_ast::Expr {
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

        nazmc_ast::Expr {
            span,
            kind: nazmc_ast::ExprKind::ArrayElemnts(elements),
        }
    } else if let Some(ArrayExprKind::ExplicitSize(ExplicitSizeArrayExpr {
        repeated_expr,
        semicolon: _,
        size_expr,
    })) = array_expr.expr_kind
    {
        let repeat = lower_expr(repeated_expr.unwrap());
        let size = lower_expr(size_expr.unwrap());
        let array_elements_sized_expr =
            Box::new(nazmc_ast::ArrayElementsSizedExpr { repeat, size });
        nazmc_ast::Expr {
            span,
            kind: nazmc_ast::ExprKind::ArrayElemntsSized(array_elements_sized_expr),
        }
    } else {
        let elements = ThinVec::new();
        nazmc_ast::Expr {
            span,
            kind: nazmc_ast::ExprKind::ArrayElemnts(elements),
        }
    }
}

#[inline]
fn lower_paren_expr(paren_expr: ParenExpr) -> nazmc_ast::Expr {
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
            nazmc_ast::Expr {
                span,
                kind: nazmc_ast::ExprKind::Parens(Box::new(first)),
            }
        } else {
            let mut exprs = ThinVec::new();
            exprs.push(first);
            for r in rest_items {
                exprs.push(lower_expr(r.unwrap().item));
            }
            nazmc_ast::Expr {
                span,
                kind: nazmc_ast::ExprKind::Tuple(exprs),
            }
        }
    } else {
        nazmc_ast::Expr {
            span,
            kind: nazmc_ast::ExprKind::Tuple(ThinVec::new()),
        }
    }
}

#[inline]
fn lower_struct_expr(struct_expr: StructExpr) -> nazmc_ast::Expr {
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

        let tuple_struct = Box::new(nazmc_ast::TupleStructExpr { path, args });

        nazmc_ast::Expr {
            span,
            kind: nazmc_ast::ExprKind::TupleStruct(tuple_struct),
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
            fn lower_struct_field_expr(
                e: StructFieldInitExpr,
            ) -> (nazmc_ast::ASTId, nazmc_ast::Expr) {
                let name = nazmc_ast::ASTId {
                    span: e.name.span,
                    id: e.name.data.val,
                };

                let expr = if let Some(e) = e.expr {
                    lower_expr(e.expr.unwrap())
                } else {
                    nazmc_ast::Expr {
                        span: name.span,
                        kind: nazmc_ast::ExprKind::Path(Box::new(nazmc_ast::PkgPathWithItem {
                            pkg_path: nazmc_ast::PkgPath {
                                ids: ThinVec::new(),
                                spans: ThinVec::new(),
                            },
                            item: nazmc_ast::ASTId {
                                span: name.span,
                                id: name.id,
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

        let fields_struct = Box::new(nazmc_ast::FieldsStructExpr { path, fields });

        nazmc_ast::Expr {
            span,
            kind: nazmc_ast::ExprKind::FieldsStruct(fields_struct),
        }
    } else {
        let span = struct_expr.dot.span.merged_with(&path.item.span);

        nazmc_ast::Expr {
            span,
            kind: nazmc_ast::ExprKind::UnitStruct(Box::new(path)),
        }
    }
}

#[inline]
fn lower_lambda_expr(lambda_expr: LambdaExpr) -> nazmc_ast::Expr {
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

        nazmc_ast::LambdaExpr { params, body }
    } else {
        let mut params = ThinVec::new();
        params.push(nazmc_ast::Binding {
            kind: nazmc_ast::BindingKind::Id(nazmc_ast::ASTId {
                span: lambda_expr.open_curly.span,
                id: PoolIdx::LAMBDA_IMPLICIT_PARAM,
            }),
            typ: None,
        });
        nazmc_ast::LambdaExpr { params, body }
    };

    nazmc_ast::Expr {
        span,
        kind: nazmc_ast::ExprKind::Lambda(Box::new(lambda)),
    }
}

fn lower_if_expr(if_expr: IfExpr) -> nazmc_ast::IfExpr {
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

    nazmc_ast::IfExpr {
        if_,
        else_ifs,
        else_,
    }
}

fn lower_when_expr(_when_expr: WhenExpr) -> nazmc_ast::Expr {
    todo!()
}
