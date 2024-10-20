use crate::{SymbolKind, Token, TokenKind};

use super::*;

#[derive(Debug)]
pub(crate) struct LambdaExpr {
    pub(crate) open_curly: OpenCurlyBraceSymbol,
    pub(crate) lambda_arrow: Option<LambdaArrow>,
    pub(crate) stms: Vec<ParseResult<Stm>>,
    /// The last expression that has no semicolons
    pub(crate) last_expr: Option<Expr>,
    pub(crate) close_curly: ParseResult<CloseCurlyBraceSymbol>,
}

#[derive(NazmcParse)]
struct LambdaExprImpl {
    pub(crate) open_curly: OpenCurlyBraceSymbol,
    pub(crate) lambda_arrow: Option<LambdaArrow>,
    pub(crate) stms: ZeroOrMany<Stm, CloseCurlyBraceSymbol>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum LambdaArrow {
    NoParams(RArrowSymbol),
    WithParams(LambdaParams),
}

#[derive(Debug)]
pub(crate) struct LambdaParams {
    pub(crate) first: Binding,
    pub(crate) rest: Vec<CommaWithBinding>,
    pub(crate) trailing_comma: Option<CommaSymbol>,
    pub(crate) r_arrow: ParseResult<RArrowSymbol>,
}

impl NazmcParse for ParseResult<LambdaExpr> {
    fn parse(iter: &mut super::TokensIter) -> Self {
        let decl_impl_node = ParseResult::<LambdaExprImpl>::parse(iter)?;

        let open_curly = decl_impl_node.open_curly;

        let lambda_arrow = decl_impl_node.lambda_arrow;

        let mut stms = decl_impl_node.stms.items;

        let pop = matches!(
            stms.last(),
            Some(Ok(Stm::Expr(ExprStm {
                semicolon: Err(_),
                ..
            })))
        );

        let last_expr = if pop {
            let Some(Ok(Stm::Expr(ExprStm { expr, .. }))) = stms.pop() else {
                unreachable!()
            };
            Some(expr)
        } else {
            None
        };

        let close_curly = decl_impl_node.stms.terminator;

        Ok(LambdaExpr {
            open_curly,
            lambda_arrow,
            stms,
            last_expr,
            close_curly,
        })
    }
}

impl NazmcParse for ParseResult<LambdaParams> {
    fn parse(iter: &mut TokensIter) -> Self {
        let peek_idx = iter.peek_idx;

        let first = ParseResult::<Binding>::parse(iter)?;

        match iter.recent() {
            Some(Token {
                kind: TokenKind::Symbol(SymbolKind::Comma),
                ..
            }) => {}
            Some(Token {
                kind: TokenKind::Symbol(SymbolKind::Minus),
                ..
            }) if matches!(
                iter.peek(),
                Some(Token {
                    kind: TokenKind::Symbol(SymbolKind::CloseAngleBracketOrGreater),
                    ..
                },)
            ) => {}
            Some(_) => {
                iter.peek_idx = peek_idx;
                return Err(ParseErr {
                    found_token_index: iter.peek_idx - 1,
                });
            }
            None => {
                iter.peek_idx = peek_idx;
                return ParseErr::eof();
            }
        };

        let rest = Vec::<CommaWithBinding>::parse(iter);

        let comma_with_arrow_symbol = ParseResult::<CommaWithRArrowSymbol>::parse(iter);

        let (trailing_comma, r_arrow) = match comma_with_arrow_symbol {
            Ok(node) => (node.comma, Ok(node.close_delim)),
            Err(err) => (Option::None, Err(err)),
        };

        Ok(LambdaParams {
            first,
            rest,
            trailing_comma,
            r_arrow,
        })
    }
}
