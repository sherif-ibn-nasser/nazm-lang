use thin_vec::ThinVec;

use crate::{
    exprs::{Expr, IfExpr},
    types::Type,
    ConditionalScope, NIRId,
};

pub struct Stm {
    pub kind_and_index: u64,
}

#[derive(Default)]
pub struct Stms {
    pub lets: ThinVec<LetStm>,
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
    pub typ: Option<Type>,
}

pub enum BindingKind {
    Id(NIRId),
    MutId(NIRId),
    CompilerId,
}
