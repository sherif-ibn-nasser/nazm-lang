use nazmc_diagnostics::span;

use crate::{KeywordKind, SymbolKind};

use super::*;

/// The parse method is written by hand to avoid backtracking
pub(crate) enum BinOp {
    LOr,
    LAnd,
    EqualEqual,
    NotEqual,
    GE,
    GT,
    LE,
    LT,
    OpenOpenRange,
    CloseOpenRange,
    OpenCloseRange,
    CloseCloseRange,
    BOr,
    Xor,
    BAnd,
    Shr,
    Shl,
    Plus,
    Minus,
    Times,
    Div,
    Mod,
}

/// The parse method is written by hand to avoid backtracking
///
/// Note that there is no unary plus operator
pub(crate) enum UnaryOp {
    Minus,
    LNot,
    BNot,
    Deref,
    Borrow,
    BorrowMut,
}

/// The parse method is written by hand to avoid backtracking
pub(crate) enum AssignOp {
    Assign,
    PlusAssign,
    MinusAssign,
    TimesAssign,
    DivAssign,
    ModAssign,
    BitAndAssign,
    BitOrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
}

macro_rules! match_peek_symbols {
    ($iter:ident, $symbol0:ident, $symbol1:ident, $symbol2:ident) => {
        match_peek_symbols!($iter, 0, $symbol0)
            && match_peek_symbols!($iter, 1, $symbol1)
            && match_peek_symbols!($iter, 2, $symbol2)
    };
    ($iter:ident, $symbol0:ident, $symbol1:ident) => {
        match_peek_symbols!($iter, 0, $symbol0) && match_peek_symbols!($iter, 1, $symbol1)
    };
    ($iter:ident, $symbol:ident) => {
        match_peek_symbols!($iter, 0, $symbol)
    };
    ($iter:ident, $nth: literal, $symbol:ident) => {
        matches!(
            $iter.peek_nth($nth),
            Some(Token {
                kind: TokenKind::Symbol(SymbolKind::$symbol),
                ..
            })
        )
    };
}

impl NazmcParse for ParseResult<BinOp> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    kind: TokenKind::Symbol(symbol_kind),
                    ..
                },
            ) => {
                let mut span = token.span;
                let (tree_kind, peek_inc) = match symbol_kind {
                    SymbolKind::Equal if match_peek_symbols!(iter, Equal) => (BinOp::EqualEqual, 1),
                    SymbolKind::ExclamationMark if match_peek_symbols!(iter, Equal) => {
                        (BinOp::NotEqual, 1)
                    }
                    SymbolKind::Dot if match_peek_symbols!(iter, Dot) => {
                        (BinOp::CloseCloseRange, 1)
                    }
                    SymbolKind::Dot if match_peek_symbols!(iter, Dot, OpenAngleBracketOrLess) => {
                        (BinOp::CloseOpenRange, 2)
                    }
                    SymbolKind::OpenAngleBracketOrLess
                        if match_peek_symbols!(iter, Dot, Dot, OpenAngleBracketOrLess) =>
                    {
                        (BinOp::OpenOpenRange, 3)
                    }

                    SymbolKind::OpenAngleBracketOrLess if match_peek_symbols!(iter, Dot, Dot) => {
                        (BinOp::OpenCloseRange, 2)
                    }

                    SymbolKind::OpenAngleBracketOrLess
                        if match_peek_symbols!(iter, OpenAngleBracketOrLess)
                            && !match_peek_symbols!(iter, 1, Equal) =>
                    {
                        (BinOp::Shr, 1)
                    }
                    SymbolKind::OpenAngleBracketOrLess if match_peek_symbols!(iter, Equal) => {
                        (BinOp::LE, 1)
                    }
                    SymbolKind::OpenAngleBracketOrLess => (BinOp::LT, 0),

                    SymbolKind::CloseAngleBracketOrGreater
                        if match_peek_symbols!(iter, CloseAngleBracketOrGreater)
                            && !match_peek_symbols!(iter, 1, Equal) =>
                    {
                        (BinOp::Shl, 1)
                    }
                    SymbolKind::CloseAngleBracketOrGreater if match_peek_symbols!(iter, Equal) => {
                        (BinOp::GE, 1)
                    }

                    SymbolKind::CloseAngleBracketOrGreater => (BinOp::GT, 0),

                    SymbolKind::BitOr if match_peek_symbols!(iter, BitOr) => (BinOp::LOr, 1),
                    SymbolKind::BitOr if !match_peek_symbols!(iter, Equal) => (BinOp::BOr, 0),
                    SymbolKind::Xor if !match_peek_symbols!(iter, Equal) => (BinOp::Xor, 0),
                    SymbolKind::BitAnd if match_peek_symbols!(iter, BitAnd) => (BinOp::LAnd, 1),
                    SymbolKind::BitAnd if !match_peek_symbols!(iter, Equal) => (BinOp::BOr, 0),
                    SymbolKind::Plus if !match_peek_symbols!(iter, Equal) => (BinOp::Plus, 0),
                    SymbolKind::Minus if !match_peek_symbols!(iter, Equal) => (BinOp::Minus, 0),
                    SymbolKind::Star if !match_peek_symbols!(iter, Equal) => (BinOp::Times, 0),
                    SymbolKind::Slash if !match_peek_symbols!(iter, Equal) => (BinOp::Div, 0),
                    SymbolKind::Modulo if !match_peek_symbols!(iter, Equal) => (BinOp::Mod, 0),
                    _ => {
                        return ParseResult::Unexpected {
                            span,
                            found: token.kind.clone(),
                            is_start_failure: true,
                        };
                    }
                };

                iter.peek_idx += peek_inc;
                span.end.col += peek_inc;

                let ok = ParseResult::Parsed(SyntaxNode {
                    span,
                    is_broken: false,
                    tree: tree_kind,
                });
                iter.next_non_space_or_comment();
                ok
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

impl NazmcParse for ParseResult<UnaryOp> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    val: _,
                    span,
                    kind: TokenKind::Symbol(symbol_kind),
                },
            ) => {
                let mut span = *span;

                let tree_kind = match symbol_kind {
                    SymbolKind::Minus if !match_peek_symbols!(iter, Equal) => UnaryOp::Minus,
                    SymbolKind::ExclamationMark if !match_peek_symbols!(iter, Equal) => {
                        UnaryOp::LNot
                    }
                    SymbolKind::BitNot if !match_peek_symbols!(iter, Equal) => UnaryOp::BNot,
                    SymbolKind::Star if !match_peek_symbols!(iter, Equal) => UnaryOp::Deref,
                    SymbolKind::Hash => {
                        let peek_idx = iter.peek_idx;
                        if let Some(Token {
                            span: mut_keyword_span,
                            kind: TokenKind::Keyword(KeywordKind::Mut),
                            ..
                        }) = iter.next_non_space_or_comment()
                        {
                            span = span.merged_with(mut_keyword_span);
                            UnaryOp::Borrow
                        } else {
                            iter.peek_idx = peek_idx;
                            UnaryOp::BorrowMut
                        }
                    }
                    _ => {
                        return ParseResult::Unexpected {
                            span,
                            found: token.kind.clone(),
                            is_start_failure: true,
                        };
                    }
                };

                let ok = ParseResult::Parsed(SyntaxNode {
                    span,
                    is_broken: false,
                    tree: tree_kind,
                });
                iter.next_non_space_or_comment();
                ok
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

impl NazmcParse for ParseResult<AssignOp> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    kind: TokenKind::Symbol(symbol_kind),
                    ..
                },
            ) => {
                let mut span = token.span;
                let (tree_kind, peek_inc) = match symbol_kind {
                    SymbolKind::Equal => (AssignOp::Assign, 0),
                    SymbolKind::Plus if match_peek_symbols!(iter, Equal) => {
                        (AssignOp::PlusAssign, 1)
                    }
                    SymbolKind::Minus if match_peek_symbols!(iter, Equal) => {
                        (AssignOp::MinusAssign, 1)
                    }
                    SymbolKind::Star if match_peek_symbols!(iter, Equal) => {
                        (AssignOp::TimesAssign, 1)
                    }
                    SymbolKind::Slash if match_peek_symbols!(iter, Equal) => {
                        (AssignOp::DivAssign, 1)
                    }
                    SymbolKind::Modulo if match_peek_symbols!(iter, Equal) => {
                        (AssignOp::ModAssign, 1)
                    }
                    SymbolKind::BitAnd if match_peek_symbols!(iter, Equal) => {
                        (AssignOp::BitAndAssign, 1)
                    }
                    SymbolKind::BitOr if match_peek_symbols!(iter, Equal) => {
                        (AssignOp::BitOrAssign, 1)
                    }
                    SymbolKind::Xor if match_peek_symbols!(iter, Equal) => (AssignOp::XorAssign, 1),
                    SymbolKind::OpenAngleBracketOrLess
                        if match_peek_symbols!(iter, OpenAngleBracketOrLess, Equal) =>
                    {
                        (AssignOp::ShrAssign, 2)
                    }
                    SymbolKind::CloseAngleBracketOrGreater
                        if match_peek_symbols!(iter, CloseAngleBracketOrGreater, Equal) =>
                    {
                        (AssignOp::ShlAssign, 2)
                    }
                    _ => {
                        return ParseResult::Unexpected {
                            span,
                            found: token.kind.clone(),
                            is_start_failure: true,
                        };
                    }
                };

                iter.peek_idx += peek_inc;
                span.end.col += peek_inc;

                let ok = ParseResult::Parsed(SyntaxNode {
                    span,
                    is_broken: false,
                    tree: tree_kind,
                });
                iter.next_non_space_or_comment();
                ok
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
