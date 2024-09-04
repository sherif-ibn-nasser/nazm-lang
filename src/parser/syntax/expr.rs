use crate::{parser::*, LiteralKind, SymbolKind};

use super::*;

#[derive(NazmcParse)]
/// The wrapper for all valid expressions syntax in the language
pub(crate) struct Expr {
    pub(crate) left: SyntaxNode<UnaryExpr>,
    pub(crate) bin: Vec<SyntaxNode<BinExpr>>,
}

#[derive(NazmcParse)]
/// This will parse the valid syntax of binary operators and will not parse their precedences
///
/// The precedence parsing will be when constructiong the HIR by the shunting-yard algorithm
/// as we want it here to be simple
pub(crate) struct BinExpr {
    pub(crate) op: SyntaxNode<BinOp>,
    pub(crate) right: ParseResult<UnaryExpr>,
}

#[derive(NazmcParse)]
pub(crate) struct UnaryExpr {
    pub(crate) ops: Vec<SyntaxNode<UnaryOp>>,
    pub(crate) expr: ParseResult<AtomicExpr>,
}

#[derive(NazmcParse)]
/// It's the atom in constructing an expression
pub(crate) enum AtomicExpr {
    Paren(Box<ParenExpr>),
    Tuple(Box<TupleExpr>),
    Id(Box<IdExpr>),
    Literal(LiteralExpr),
}

#[derive(NazmcParse)]
pub(crate) struct ParenExpr {
    pub(crate) open_paren: SyntaxNode<OpenParenthesisSymbol>,
    pub(crate) expr: ParseResult<Expr>,
    pub(crate) close_paren: ParseResult<CloseParenthesisSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct IdExpr {
    pub(crate) id: SyntaxNode<Id>,
    pub(crate) fn_call: Optional<FnCallExpr>,
}

/// This has a hand-written parse method and it is like the other treminal tokens
pub(crate) struct LiteralExpr {
    kind: LiteralKind,
}

/// The parse method is written by hand to avoid backtracking
pub(crate) enum BinOp {
    LOr(LogicalOrSymbol),
    LAnd(LogicalAndSymbol),
    BOr(BitOrSymbol),
    Xor(XorSymbol),
    BAnd(BitAndSymbol),
    Shr(ShrSymbol),
    Shl(ShlSymbol),
    EqualEqual(EqualEqualSymbol),
    NotEqual(NotEqualSymbol),
    GE(GreaterEqualSymbol),
    GT(CloseAngleBracketOrGreaterSymbol),
    LE(LessEqualSymbol),
    LT(OpenAngleBracketOrLessSymbol),
    Plus(PlusSymbol),
    Minus(MinusSymbol),
    Times(StarSymbol),
    Div(SlashSymbol),
    Mod(ModuloSymbol),
    Pow(PowerSymbol),
}

/// The parse method is written by hand to avoid backtracking
pub(crate) enum UnaryOp {
    Plus(PlusSymbol),
    Minus(MinusSymbol),
    LNot(ExclamationMarkSymbol),
    BNot(BitNotSymbol),
}

impl NazmcParse for ParseResult<LiteralExpr> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(Token {
                val,
                span,
                kind: TokenKind::Literal(literal_kind),
            }) => {
                let ok = ParseResult::Parsed(SyntaxNode {
                    span: *span,
                    is_broken: false,
                    tree: LiteralExpr {
                        kind: literal_kind.clone(),
                    },
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

impl NazmcParse for ParseResult<BinOp> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    val,
                    span,
                    kind: TokenKind::Symbol(symbol_kind),
                },
            ) => {
                let tree_kind = match symbol_kind {
                    SymbolKind::LogicalOr => BinOp::LOr(LogicalOrSymbol),
                    SymbolKind::LogicalAnd => BinOp::LAnd(LogicalAndSymbol),
                    SymbolKind::BitOr => BinOp::BOr(BitOrSymbol),
                    SymbolKind::Xor => BinOp::Xor(XorSymbol),
                    SymbolKind::BitAnd => BinOp::BAnd(BitAndSymbol),
                    SymbolKind::Shr => BinOp::Shr(ShrSymbol),
                    SymbolKind::Shl => BinOp::Shl(ShlSymbol),
                    SymbolKind::EqualEqual => BinOp::EqualEqual(EqualEqualSymbol),
                    SymbolKind::NotEqual => BinOp::NotEqual(NotEqualSymbol),
                    SymbolKind::GreaterEqual => BinOp::GE(GreaterEqualSymbol),
                    SymbolKind::CloseAngleBracketOrGreater => {
                        BinOp::GT(CloseAngleBracketOrGreaterSymbol)
                    }
                    SymbolKind::LessEqual => BinOp::LE(LessEqualSymbol),
                    SymbolKind::OpenAngleBracketOrLess => BinOp::LT(OpenAngleBracketOrLessSymbol),
                    SymbolKind::Plus => BinOp::Plus(PlusSymbol),
                    SymbolKind::Minus => BinOp::Minus(MinusSymbol),
                    SymbolKind::Star => BinOp::Times(StarSymbol),
                    SymbolKind::Slash => BinOp::Div(SlashSymbol),
                    SymbolKind::Modulo => BinOp::Mod(ModuloSymbol),
                    SymbolKind::Power => BinOp::Pow(PowerSymbol),
                    _ => {
                        return ParseResult::Unexpected {
                            span: *span,
                            found: token.kind.clone(),
                            is_start_failure: true,
                        };
                    }
                };

                let ok = ParseResult::Parsed(SyntaxNode {
                    span: *span,
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
                    val,
                    span,
                    kind: TokenKind::Symbol(symbol_kind),
                },
            ) => {
                let tree_kind = match symbol_kind {
                    SymbolKind::Plus => UnaryOp::Plus(PlusSymbol),
                    SymbolKind::Minus => UnaryOp::Minus(MinusSymbol),
                    SymbolKind::ExclamationMark => UnaryOp::LNot(ExclamationMarkSymbol),
                    SymbolKind::BitNot => UnaryOp::BNot(BitNotSymbol),
                    _ => {
                        return ParseResult::Unexpected {
                            span: *span,
                            found: token.kind.clone(),
                            is_start_failure: true,
                        };
                    }
                };

                let ok = ParseResult::Parsed(SyntaxNode {
                    span: *span,
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
