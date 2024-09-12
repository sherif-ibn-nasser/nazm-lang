use crate::parser::*;

use super::*;

mod array;
mod control_flow;
mod lambda;

pub(crate) use array::*;
pub(crate) use control_flow::*;
pub(crate) use lambda::*;

#[derive(NazmcParse, Debug)]
/// The wrapper for all valid expressions syntax in the language
pub(crate) struct Expr {
    pub(crate) left: Box<PrimaryExpr>,
    pub(crate) bin: Vec<BinExpr>,
}

/// This will parse the valid syntax of binary operators and will not parse their precedences
///
/// The precedence parsing will be when constructiong the HIR by the shunting-yard algorithm
/// as we want it here to be simple
///
#[derive(NazmcParse, Debug)]
pub(crate) struct BinExpr {
    pub(crate) op: BinOp,
    pub(crate) right: ParseResult<PrimaryExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct PrimaryExpr {
    pub(crate) expr: PrimaryExprKind,
    pub(crate) post_ops: Vec<PostOpExpr>,
    pub(crate) inner_access: Vec<InnerAccessExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum PrimaryExprKind {
    Unary(UnaryExpr),
    Atomic(AtomicExpr),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct UnaryExpr {
    pub(crate) first_op: UnaryOp,
    pub(crate) rest_ops: Vec<UnaryOp>,
    pub(crate) expr: ParseResult<AtomicExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum PostOpExpr {
    Invoke(ParenExpr),
    Lambda(LambdaExpr),
    Index(IdxExpr),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct InnerAccessExpr {
    pub(crate) dot: DotSymbol,
    pub(crate) inner: ParseResult<IdExpr>,
    pub(crate) post_ops: Vec<PostOpExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct IdxExpr {
    pub(crate) open_bracket: OpenSquareBracketSymbol,
    pub(crate) expr: ParseResult<Expr>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse, Debug)]
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

#[derive(NazmcParse, Debug)]
pub(crate) struct IdExpr {
    pub(crate) path: SimplePath,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct StructExpr {
    pub(crate) dot: DotSymbol,
    pub(crate) path: ParseResult<SimplePath>,
    pub(crate) init: Option<StructInit>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum StructInit {
    Tuple(ParenExpr),
    Fields(StructFieldsInitExpr),
}

#[derive(NazmcParse, Debug)]
pub(crate) enum ExprWithBlock {
    If(IfExpr),
    When(WhenExpr),
    Loop(LoopExpr),
    While(WhileExpr),
    DoWhile(DoWhileExpr),
    Run(RunExpr),
}
