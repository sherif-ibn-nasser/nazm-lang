use crate::SymbolKind;

use super::*;

pub(crate) struct LambdaExpr {
    pub(crate) open_curly: SyntaxNode<OpenCurlyBraceSymbol>,
    pub(crate) lambda_arrow: Optional<LambdaArrow>,
    pub(crate) stms: Vec<ParseResult<Stm>>,
    /// The last expression that has no semicolons
    pub(crate) last_expr: Optional<Expr>,
    pub(crate) close_curly: ParseResult<CloseCurlyBraceSymbol>,
}

#[derive(NazmcParse)]
struct LambdaExprImpl {
    pub(crate) open_curly: SyntaxNode<OpenCurlyBraceSymbol>,
    pub(crate) lambda_arrow: Optional<LambdaArrow>,
    pub(crate) stms: ZeroOrMany<Stm, CloseCurlyBraceSymbol>,
}

#[derive(NazmcParse)]
pub(crate) enum LambdaArrow {
    NoParams(RArrow),
    WithParams(LambdaParams),
}

pub(crate) struct LambdaParams {
    pub(crate) first: SyntaxNode<BindingDecl>,
    pub(crate) rest: Vec<ParseResult<CommaWithBindingDecl>>,
    pub(crate) trailing_comma: Optional<CommaSymbol>,
    pub(crate) r_arrow: ParseResult<RArrow>,
}

impl NazmcParse for ParseResult<LambdaExpr> {
    fn parse(iter: &mut super::TokensIter) -> Self {
        let peek_idx = iter.peek_idx;
        let node = match ParseResult::<LambdaExprImpl>::parse(iter) {
            ParseResult::Parsed(node) => node,
            ParseResult::Unexpected { span, found, .. } => {
                iter.peek_idx = peek_idx;
                return ParseResult::Unexpected {
                    span,
                    found,
                    is_start_failure: true,
                };
            }
        };

        let span = node.span;
        let is_broken = node.is_broken;
        let open_curly = node.tree.open_curly;
        let lambda_arrow = node.tree.lambda_arrow;
        let mut stms = node.tree.stms.items;

        let pop = matches!(
            stms.last(),
            Some(ParseResult::Parsed(SyntaxNode {
                tree: Stm::Expr(ExprStm::Any(AnyExprStm {
                    semicolon: ParseResult::Unexpected { .. },
                    ..
                })),
                ..
            }))
        );

        let last_expr = if pop {
            let Some(ParseResult::Parsed(SyntaxNode {
                tree:
                    Stm::Expr(ExprStm::Any(AnyExprStm {
                        semicolon: ParseResult::Unexpected { .. },
                        expr,
                    })),
                ..
            })) = stms.pop()
            else {
                unreachable!()
            };

            Optional::Some(expr)
        } else {
            Optional::None
        };

        let close_curly = node.tree.stms.terminator;
        ParseResult::Parsed(SyntaxNode {
            span,
            is_broken,
            tree: LambdaExpr {
                open_curly,
                lambda_arrow,
                stms,
                last_expr,
                close_curly,
            },
        })
    }
}

impl NazmcParse for ParseResult<LambdaParams> {
    fn parse(iter: &mut TokensIter) -> Self {
        let peek_idx = iter.peek_idx;

        let first = match <ParseResult<BindingDecl>>::parse(iter) {
            ParseResult::Parsed(tree) => tree,
            ParseResult::Unexpected { span, found, .. } => {
                iter.peek_idx = peek_idx;
                return ParseResult::Unexpected {
                    span,
                    found,
                    is_start_failure: true,
                };
            }
        };

        let unexpected = match iter.recent() {
            Some(Token {
                kind: TokenKind::Symbol(SymbolKind::Comma),
                ..
            }) => None,
            Some(Token {
                kind: TokenKind::Symbol(SymbolKind::Minus),
                ..
            }) if matches!(
                iter.peek(),
                Some(Token {
                    kind: TokenKind::Symbol(SymbolKind::CloseAngleBracketOrGreater),
                    ..
                },)
            ) =>
            {
                None
            }
            Some(token) => Some(ParseResult::Unexpected {
                span: token.span,
                found: token.kind.clone(),
                is_start_failure: true,
            }),
            None => Some(ParseResult::unexpected_eof(iter.peek_start_span())),
        };

        if let Some(unexpected) = unexpected {
            iter.peek_idx = peek_idx;
            return unexpected;
        }

        let result = ZeroOrMany::<CommaWithBindingDecl, CommaWithRArrow>::parse(iter);

        let span = first.span.merged_with(&result.span().unwrap());

        let is_broken = first.is_broken || !result.is_parsed_and_valid();

        let (trailing_comma, r_arrow) = match result.terminator {
            ParseResult::Parsed(node) => {
                (node.tree._comma, ParseResult::Parsed(node.tree.close_delim))
            }
            ParseResult::Unexpected {
                span,
                found,
                is_start_failure,
            } => (
                Optional::None,
                ParseResult::Unexpected {
                    span,
                    found,
                    is_start_failure,
                },
            ),
        };

        ParseResult::Parsed(SyntaxNode {
            span,
            is_broken,
            tree: LambdaParams {
                first,
                rest: result.items,
                trailing_comma,
                r_arrow,
            },
        })
    }
}
