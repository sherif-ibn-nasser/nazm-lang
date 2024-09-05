use crate::{parser::*, KeywordKind, LiteralKind, SymbolKind};

use super::*;

/// The wrapper for all valid expressions syntax in the language
#[derive(NazmcParse)]
pub(crate) struct Expr {
    pub(crate) left: SyntaxNode<UnaryExpr>,
    pub(crate) bin: Vec<SyntaxNode<BinExpr>>,
}

/// This will parse the valid syntax of binary operators and will not parse their precedences
///
/// The precedence parsing will be when constructiong the HIR by the shunting-yard algorithm
/// as we want it here to be simple
///
#[derive(NazmcParse)]
pub(crate) struct BinExpr {
    pub(crate) op: SyntaxNode<BinOp>,
    pub(crate) right: ParseResult<UnaryExpr>,
}

#[derive(NazmcParse)]
pub(crate) struct UnaryExpr {
    pub(crate) ops: Vec<SyntaxNode<UnaryOp>>,
    pub(crate) expr: ParseResult<AtomicExpr>,
    pub(crate) inner_access: Vec<SyntaxNode<InnerAccessExpr>>,
}

#[derive(NazmcParse)]
pub(crate) struct InnerAccessExpr {
    pub(crate) dot: SyntaxNode<DotSymbol>,
    pub(crate) inner: ParseResult<IdExpr>,
}

#[derive(NazmcParse)]
/// It's the atom in constructing an expression
pub(crate) enum AtomicExpr {
    Paren(Box<ParenExpr>),
    Tuple(Box<TupleExpr>),
    Id(Box<IdExpr>),
    Literal(LiteralExpr),
    On(OnKeyword),
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
    pub(crate) indecies: Vec<SyntaxNode<IdxExpr>>,
}

#[derive(NazmcParse)]
pub(crate) struct IdxExpr {
    pub(crate) open_bracket: SyntaxNode<OpenSquareBracketSymbol>,
    pub(crate) expr: ParseResult<Expr>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
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
    OpenOpenRange(LessDotDotLessSymbol),
    CloseOpenRange(DotDotLessSymbol),
    OpenCLoseRange(LessDotDotSymbol),
    CloseCLoseRange(DotDotSymbol),
}

/// The parse method is written by hand to avoid backtracking
///
/// Note that there is no unary plus operator
pub(crate) enum UnaryOp {
    Minus(MinusSymbol),
    LNot(ExclamationMarkSymbol),
    BNot(BitNotSymbol),
    Deref(StarSymbol),
    Borrow(HashSymbol),
    BorrowMut(HashSymbol, MutKeyword),
}

impl NazmcParse for ParseResult<LiteralExpr> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(Token {
                span,
                kind: TokenKind::Literal(literal_kind),
                ..
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
                    span,
                    kind: TokenKind::Symbol(symbol_kind),
                    ..
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
                    SymbolKind::LessDotDotLess => BinOp::OpenOpenRange(LessDotDotLessSymbol),
                    SymbolKind::DotDotLess => BinOp::CloseOpenRange(DotDotLessSymbol),
                    SymbolKind::LessDotDot => BinOp::OpenCLoseRange(LessDotDotSymbol),
                    SymbolKind::DotDot => BinOp::CloseCLoseRange(DotDotSymbol),
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
                    val: _,
                    span,
                    kind: TokenKind::Symbol(symbol_kind),
                },
            ) => {
                let mut span = *span;

                let tree_kind = match symbol_kind {
                    SymbolKind::Minus => UnaryOp::Minus(MinusSymbol),
                    SymbolKind::ExclamationMark => UnaryOp::LNot(ExclamationMarkSymbol),
                    SymbolKind::BitNot => UnaryOp::BNot(BitNotSymbol),
                    SymbolKind::Star => UnaryOp::Deref(StarSymbol),
                    SymbolKind::Hash => {
                        let peek_idx = iter.peek_idx;
                        if let Some(Token {
                            span: mut_keyword_span,
                            kind: TokenKind::Keyword(KeywordKind::Mut),
                            ..
                        }) = iter.next_non_space_or_comment()
                        {
                            span = span.merged_with(mut_keyword_span);
                            UnaryOp::Borrow(HashSymbol)
                        } else {
                            iter.peek_idx = peek_idx;
                            UnaryOp::BorrowMut(HashSymbol, MutKeyword)
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
