use crate::SymbolKind;

use super::*;

pub(crate) struct DoubleColonsSymbol;

pub(crate) struct RArrow;

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

impl NazmcParse for ParseResult<RArrow> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    kind: TokenKind::Symbol(SymbolKind::Minus),
                    ..
                },
            ) if matches!(
                iter.peek(),
                Some(Token {
                    kind: TokenKind::Symbol(SymbolKind::CloseAngleBracketOrGreater),
                    ..
                },)
            ) =>
            {
                let mut span = token.span;
                span.end.col += 1;
                iter.peek_idx += 1; // Eat next '>'
                iter.next_non_space_or_comment();
                ParseResult::Parsed(SyntaxNode {
                    span,
                    is_broken: false,
                    tree: RArrow,
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
