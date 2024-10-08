use super::*;

#[derive(Debug)]
pub(crate) struct ArrayExpr {
    pub(crate) open_bracket: OpenSquareBracketSymbol,
    pub(crate) expr_kind: Option<ArrayExprKind>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(Debug)]
pub(crate) enum ArrayExprKind {
    ExplicitSize(ExplicitSizeArrayExpr),
    Elements(ElementsArrayExpr),
}

#[derive(Debug)]
pub(crate) struct ExplicitSizeArrayExpr {
    pub(crate) repeated_expr: ParseResult<Expr>,
    pub(crate) semicolon: SemicolonSymbol,
    pub(crate) size_expr: ParseResult<Expr>,
}

#[derive(Debug)]
pub(crate) struct ElementsArrayExpr {
    pub(crate) first: ParseResult<Expr>,
    pub(crate) rest: Vec<ParseResult<CommaWithExpr>>,
    pub(crate) trailing_comma: Option<CommaSymbol>,
}

impl NazmcParse for ParseResult<ArrayExpr> {
    fn parse(iter: &mut TokensIter) -> Self {
        let decl_impl_node = ParseResult::<ArrayExprParseImpl>::parse(iter)?;

        let open_bracket = decl_impl_node.open_bracket;

        // It is safe to unwrap as it has zero or many in it that will only return if the terminator is found
        // Apply `cargo expand` on this file to see the generated NazmcParse
        let close_array_expr = decl_impl_node.close_array.unwrap();

        let close_array_expr_with_items = match close_array_expr {
            CloseArrayExpr::WithItems(close_array_expr_with_items) => close_array_expr_with_items,
            CloseArrayExpr::NoItems(close_bracket) => {
                return Ok(ArrayExpr {
                    open_bracket,
                    expr_kind: None,
                    close_bracket: Ok(close_bracket),
                });
            }
        };

        let first = close_array_expr_with_items.first;

        // It is safe here also to unwrap
        let kind = close_array_expr_with_items.kind.unwrap();

        let (expr_kind_tree, close_bracket) = match kind {
            CloseArrayExprWithItemsKind::ExplicitSize(explicit_size) => {
                let close_bracket = explicit_size.close_bracket;
                let expr_kind_tree = ArrayExprKind::ExplicitSize(ExplicitSizeArrayExpr {
                    repeated_expr: first,
                    semicolon: explicit_size.semicolon,
                    size_expr: explicit_size.size_expr,
                });
                (expr_kind_tree, close_bracket)
            }
            CloseArrayExprWithItemsKind::ImplicitSize(imlicit_size) => {
                let (trailing_comma, close_bracket) =
                    if let Ok(tree) = imlicit_size.items.terminator {
                        (tree.comma, Ok(tree.close_delim))
                    } else {
                        (None, ParseErr::eof())
                    };

                let expr_kind_tree = ArrayExprKind::Elements(ElementsArrayExpr {
                    first,
                    rest: imlicit_size.items.items,
                    trailing_comma,
                });
                (expr_kind_tree, close_bracket)
            }
        };

        Ok(ArrayExpr {
            open_bracket,
            expr_kind: Some(expr_kind_tree),
            close_bracket,
        })
    }
}

impl NazmcParse for ParseResult<ArrayExprKind> {
    fn parse(_iter: &mut TokensIter) -> Self {
        unreachable!()
    }
}

impl NazmcParse for ParseResult<ExplicitSizeArrayExpr> {
    fn parse(_iter: &mut TokensIter) -> Self {
        unreachable!()
    }
}

impl NazmcParse for ParseResult<ElementsArrayExpr> {
    fn parse(_iter: &mut TokensIter) -> Self {
        unreachable!()
    }
}

// Here is the real implentation for parsing array expressions
// it is created this way to not reparse first expression if the size expression parsing failed
#[derive(NazmcParse)]
struct ArrayExprParseImpl {
    open_bracket: OpenSquareBracketSymbol,
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
    semicolon: SemicolonSymbol,
    size_expr: ParseResult<Expr>,
    close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse)]
struct CloseArrayExprWithImplicitSize {
    items: ZeroOrMany<CommaWithExpr, CommaWithCloseSquareBracketSymbol>,
}
