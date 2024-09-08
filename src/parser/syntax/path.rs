use crate::SymbolKind;

use super::*;

#[derive(NazmcParse)]
pub(crate) struct SimplePath {
    pub(crate) top: SyntaxNode<Id>,
    pub(crate) inners: Vec<SyntaxNode<SimpleInnerPath>>,
}

#[derive(NazmcParse)]
pub(crate) struct SimpleInnerPath {
    pub(crate) double_colons: SyntaxNode<DoubleColonsSymbol>,
    pub(crate) inner: ParseResult<Id>,
}

pub(crate) struct DoubleColonsSymbol;

impl NazmcParse for ParseResult<DoubleColonsSymbol> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    kind: TokenKind::Symbol(SymbolKind::Colon),
                    ..
                },
            ) if matches!(
                iter.peek(),
                Some(Token {
                    kind: TokenKind::Symbol(SymbolKind::Colon),
                    ..
                },)
            ) =>
            {
                let mut span = token.span;
                span.end.col += 1;
                iter.peek_idx += 1; // Eat next colon
                iter.next_non_space_or_comment();
                ParseResult::Parsed(SyntaxNode {
                    span,
                    is_broken: false,
                    tree: DoubleColonsSymbol,
                })
            }
            Some(token) => ParseResult::Unexpected {
                span: token.span,
                found: token.kind.clone(),
                is_start_failure: true,
            },
            None => ParseResult::unexpected_eof(iter.peek_start_span()),
        }
    }
}
