use crate::SymbolKind;

use super::*;

#[derive(NazmcParse)]
pub(crate) struct LambdaExpr {
    pub(crate) open_curly: SyntaxNode<OpenCurlyBracesSymbol>,
    pub(crate) arrow: Optional<LambdaArrow>,
    // TODO: stms
    pub(crate) close_curly: SyntaxNode<CloseCurlyBracesSymbol>,
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

impl NazmcParse for ParseResult<LambdaParams> {
    fn parse(iter: &mut TokensIter) -> Self {
        let peek_idx = iter.peek_idx;

        let first = match <ParseResult<BindingDecl>>::parse(iter) {
            ParseResult::Parsed(tree) => tree,
            ParseResult::Unexpected { span, found, .. } => {
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
