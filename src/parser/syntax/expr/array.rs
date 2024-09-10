use super::*;

pub(crate) struct ArrayExpr {
    pub(crate) open_bracket: SyntaxNode<OpenSquareBracketSymbol>,
    pub(crate) expr_kind: Optional<ArrayExprKind>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

pub(crate) enum ArrayExprKind {
    ExplicitSize(ExplicitSizeArrayExpr),
    Elements(ElementsArrayExpr),
}

pub(crate) struct ExplicitSizeArrayExpr {
    pub(crate) repeated_expr: ParseResult<Expr>,
    pub(crate) semicolon: SyntaxNode<SemicolonSymbol>,
    pub(crate) size_expr: ParseResult<Expr>,
}

pub(crate) struct ElementsArrayExpr {
    pub(crate) first: ParseResult<Expr>,
    pub(crate) rest: Vec<ParseResult<CommaWithExpr>>,
    pub(crate) trailing_comma: Optional<CommaSymbol>,
}

impl NazmcParse for ParseResult<ArrayExpr> {
    fn parse(iter: &mut TokensIter) -> Self {
        let parse_result = ParseResult::<ArrayExprParseImpl>::parse(iter);

        let decl_impl_node = match parse_result {
            ParseResult::Parsed(decl_impl) => decl_impl,
            ParseResult::Unexpected {
                span,
                found,
                is_start_failure,
            } => {
                return ParseResult::Unexpected {
                    span,
                    found,
                    is_start_failure,
                }
            }
        };

        let is_broken = decl_impl_node.is_broken;
        let span = decl_impl_node.span;
        let open_bracket = decl_impl_node.tree.open_bracket;

        // It is safe to unwrap as it has zero or many in it that will only return if the terminator is found
        // Apply `cargo expand` on this file to see the generated NazmcParse
        let close_array_expr = decl_impl_node.tree.close_array.unwrap();

        let close_array_expr_with_items = match close_array_expr {
            SyntaxNode {
                tree: CloseArrayExpr::WithItems(close_array_expr_with_items),
                ..
            } => close_array_expr_with_items,
            SyntaxNode {
                tree: CloseArrayExpr::NoItems(close_bracket),
                span: bracket_span,
                ..
            } => {
                return ParseResult::Parsed(SyntaxNode {
                    span,
                    is_broken,
                    tree: ArrayExpr {
                        open_bracket,
                        expr_kind: Optional::None,
                        close_bracket: ParseResult::Parsed(SyntaxNode {
                            span: bracket_span,
                            is_broken: false,
                            tree: close_bracket,
                        }),
                    },
                });
            }
        };

        let first = close_array_expr_with_items.first;
        let first_span = first.span().unwrap();
        // It is safe here also to unwrap
        let kind = close_array_expr_with_items.kind.unwrap();

        let expr_kind_is_broken = !first.is_parsed_and_valid() || kind.is_broken;

        let (expr_kind_tree, end_span, close_bracket) = match kind.tree {
            CloseArrayExprWithItemsKind::ExplicitSize(explicit_size) => {
                let close_bracket = explicit_size.close_bracket;
                let end_span = explicit_size.size_expr.span().unwrap();
                let expr_kind_tree = ArrayExprKind::ExplicitSize(ExplicitSizeArrayExpr {
                    repeated_expr: first,
                    semicolon: explicit_size.semicolon,
                    size_expr: explicit_size.size_expr,
                });
                (expr_kind_tree, end_span, close_bracket)
            }
            CloseArrayExprWithItemsKind::ImplicitSize(imlicit_size) => {
                let (trailing_comma, close_bracket) = if imlicit_size.items.terminator.is_parsed() {
                    let tree = imlicit_size.items.terminator.unwrap().tree;
                    (tree._comma, ParseResult::Parsed(tree.close_delim))
                } else {
                    (
                        Optional::None,
                        ParseResult::unexpected_eof(imlicit_size.items.terminator.span().unwrap()),
                    )
                };

                let end_span = if let Some(span) = trailing_comma.span() {
                    span
                } else if let Some(item) = imlicit_size.items.items.last() {
                    item.span().unwrap()
                } else {
                    first_span
                };

                let expr_kind_tree = ArrayExprKind::Elements(ElementsArrayExpr {
                    first,
                    rest: imlicit_size.items.items,
                    trailing_comma,
                });
                (expr_kind_tree, end_span, close_bracket)
            }
        };

        let expr_kind_span = first_span.merged_with(&end_span);

        ParseResult::Parsed(SyntaxNode {
            span,
            is_broken,
            tree: ArrayExpr {
                open_bracket,
                expr_kind: Optional::Some(SyntaxNode {
                    span: expr_kind_span,
                    is_broken: expr_kind_is_broken,
                    tree: expr_kind_tree,
                }),
                close_bracket,
            },
        })
    }
}

impl NazmcParse for ParseResult<ArrayExprKind> {
    fn parse(iter: &mut TokensIter) -> Self {
        unreachable!()
    }
}

impl NazmcParse for ParseResult<ExplicitSizeArrayExpr> {
    fn parse(iter: &mut TokensIter) -> Self {
        unreachable!()
    }
}

impl NazmcParse for ParseResult<ElementsArrayExpr> {
    fn parse(iter: &mut TokensIter) -> Self {
        unreachable!()
    }
}

// Here is the real implentation for parsing array expressions
// it is created this way to not reparse first expression if the size expression parsing failed
#[derive(NazmcParse)]
struct ArrayExprParseImpl {
    open_bracket: SyntaxNode<OpenSquareBracketSymbol>,
    close_array: ParseResult<CloseArrayExpr>,
}

#[derive(NazmcParse)]
enum CloseArrayExpr {
    NoItems(CloseSquareBracketSymbol),
    WithItems(CloseArrayExprWithItems),
}

#[derive(NazmcParse)]
struct CloseArrayExprWithItems {
    first: ParseResult<Expr>,
    kind: ParseResult<CloseArrayExprWithItemsKind>,
}

#[derive(NazmcParse)]
enum CloseArrayExprWithItemsKind {
    ExplicitSize(CloseArrayExprWithExplicitSize),
    ImplicitSize(CloseArrayExprWithImplicitSize),
}

#[derive(NazmcParse)]
struct CloseArrayExprWithExplicitSize {
    semicolon: SyntaxNode<SemicolonSymbol>,
    size_expr: ParseResult<Expr>,
    close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse)]
struct CloseArrayExprWithImplicitSize {
    items: ZeroOrMany<CommaWithExpr, CommaWithCloseSquareBracketSymbol>,
}
