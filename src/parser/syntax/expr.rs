use crate::{parser::*, LiteralKind};

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
    pub(crate) indecies: Vec<SyntaxNode<IdxExpr>>,
    pub(crate) inner_access: Vec<SyntaxNode<InnerAccessExpr>>,
}

#[derive(NazmcParse)]
pub(crate) struct InnerAccessExpr {
    pub(crate) dot: SyntaxNode<DotSymbol>,
    pub(crate) inner: ParseResult<IdExpr>,
    pub(crate) indecies: Vec<SyntaxNode<IdxExpr>>,
}

#[derive(NazmcParse)]
/// It's the atom in constructing an expression
pub(crate) enum AtomicExpr {
    SizedArray(Box<SizedArrayExpr>),
    Array(Box<ArrayExpr>),
    Paren(Box<ParenExpr>),
    Struct(Box<StructExpr>),
    Id(Box<IdExpr>),
    Literal(LiteralExpr),
    On(OnKeyword),
    If(Box<IfExpr>),
    When(Box<WhenExpr>),
    Loop(Box<LoopExpr>),
    While(Box<WhileExpr>),
    DoWhile(Box<DoWhileExpr>),
    BLock(Box<BlockExpr>),
}

#[derive(NazmcParse)]
/// TODO: This might not have good error recovery if the bracket is not closed as we need to skip bad tokens between the two brackets
pub(crate) struct SizedArrayExpr {
    pub(crate) dot: SyntaxNode<DotSymbol>,
    pub(crate) open_bracket: SyntaxNode<OpenSquareBracketSymbol>, // This will backtrack as it maybe a struct expression after the dot symbol
    pub(crate) repeat: ParseResult<Expr>,
    pub(crate) semi_colon: ParseResult<SemicolonSymbol>,
    pub(crate) size: ParseResult<Expr>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct StructExpr {
    pub(crate) dot: SyntaxNode<DotSymbol>,
    pub(crate) path: ParseResult<SimplePath>,
    pub(crate) init: Optional<StructInit>,
}

#[derive(NazmcParse)]
pub(crate) enum StructInit {
    Tuple(ParenExpr),
    Fields(StructFieldsInitExpr),
}

#[derive(NazmcParse)]
pub(crate) struct IdExpr {
    pub(crate) path: SyntaxNode<SimplePath>,
    pub(crate) call: Optional<ParenExpr>,
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

#[derive(NazmcParse)]
pub(crate) struct IfExpr {}

#[derive(NazmcParse)]
pub(crate) struct WhenExpr {}

#[derive(NazmcParse)]
pub(crate) struct LoopExpr {}

#[derive(NazmcParse)]
pub(crate) struct WhileExpr {}

#[derive(NazmcParse)]
pub(crate) struct DoWhileExpr {}

#[derive(NazmcParse)]
pub(crate) struct BlockExpr {}
