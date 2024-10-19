use crate::{Expr, ModPathWithItem};
use nazmc_diagnostics::span::Span;
use thin_vec::ThinVec;

pub struct Types {
    pub paths: ThinVec<ModPathWithItem>,
    pub ptrs: ThinVec<Type>,
    pub refs: ThinVec<Type>,
    pub ptrs_mut: ThinVec<PtrMutType>,
    pub refs_mut: ThinVec<RefMutType>,
    pub parens: ThinVec<Type>,
    pub slices: ThinVec<Type>,
    pub tuples: ThinVec<TupleType>,
    pub arrays: ThinVec<ArrayType>,
}

pub struct Type {
    pub kind_and_index: u64,
    pub span: Span,
}

pub struct PtrMutType {
    pub typ: Type,
    pub star_mut_span: Span,
}

pub struct RefMutType {
    pub typ: Type,
    pub hash_mut_span: Span,
}

pub struct TupleType {
    pub types: ThinVec<Type>,
    pub parens_span: Span,
}

pub struct ArrayType {
    pub typ: Type,
    pub size: Expr,
}
