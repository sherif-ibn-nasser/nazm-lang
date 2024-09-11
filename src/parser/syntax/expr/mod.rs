use crate::{parser::*, LiteralKind};

use super::*;

mod array;
mod control_flow;
mod lambda;
mod op;

pub(crate) use array::*;
pub(crate) use control_flow::*;
pub(crate) use lambda::*;
pub(crate) use op::*;

#[derive(NazmcParse)]
pub(crate) enum ExprWithBlock {
    If(IfExpr),
    When(WhenExpr),
    Loop(LoopExpr),
    While(WhileExpr),
    DoWhile(DoWhileExpr),
    Run(RunExpr),
}

#[derive(NazmcParse)]
/// The wrapper for all valid expressions syntax in the language
pub(crate) struct Expr {
    pub(crate) left: Box<SyntaxNode<UnaryExpr>>,
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
    pub(crate) expr: SyntaxNode<AtomicExpr>,
    pub(crate) post_ops: Vec<SyntaxNode<PostOpExpr>>,
    pub(crate) inner_access: Vec<SyntaxNode<InnerAccessExpr>>,
}

#[derive(NazmcParse)]
pub(crate) struct InnerAccessExpr {
    pub(crate) dot: SyntaxNode<DotSymbol>,
    pub(crate) inner: ParseResult<IdExpr>,
    pub(crate) post_ops: Vec<SyntaxNode<PostOpExpr>>,
}

#[derive(NazmcParse)]
pub(crate) enum PostOpExpr {
    Invoke(ParenExpr),
    Lambda(LambdaExpr),
    Index(IdxExpr),
}

#[derive(NazmcParse)]
pub(crate) struct IdxExpr {
    pub(crate) open_bracket: SyntaxNode<OpenSquareBracketSymbol>,
    pub(crate) expr: ParseResult<Expr>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse)]
/// It's the atom in constructing an expression
pub(crate) enum AtomicExpr {
    Array(ArrayExpr),
    Paren(ParenExpr),
    Struct(StructExpr),
    Id(IdExpr),
    Literal(LiteralExpr),
    On(OnKeyword),
    Lambda(LambdaExpr),
    Break(BreakExpr),
    Continue(ContinueExpr),
    Return(ReturnExpr),
    WithBlock(ExprWithBlock),
}

#[derive(NazmcParse)]
pub(crate) struct IdExpr {
    pub(crate) path: SyntaxNode<SimplePath>,
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
