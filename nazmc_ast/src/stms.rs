use nazmc_diagnostics::span::Span;
use thin_vec::ThinVec;

use crate::{
    exprs::{Expr, IfExpr},
    types::Type,
    ASTId, ConditionalScope,
};

pub struct Stm {
    pub kind_and_index: u64,
}

pub struct Stms {
    pub lets: ThinVec<LetStm>,
    pub let_muts: ThinVec<LetStm>,
    pub ifs: ThinVec<IfExpr>,
    pub whiles: ThinVec<ConditionalScope>,
    pub exprs: ThinVec<Expr>,
}

pub struct LetStm {
    pub binding: Binding,
    pub assign: Option<Expr>,
}

pub struct Binding {
    pub kind: BindingKind,
    pub typ: Type,
}

pub enum BindingKind {
    Name(ASTId),
    TupleDestruction(ThinVec<Binding>, Span),
}
