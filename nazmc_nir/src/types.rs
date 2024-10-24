use crate::{Expr, ItemInPkg};
use nazmc_diagnostics::span::Span;
use thin_vec::ThinVec;

pub struct Types {
    pub paths: ThinVec<ItemInPkg>,
    pub ptrs: ThinVec<Type>,
    pub refs: ThinVec<Type>,
    pub ptrs_mut: ThinVec<Type>,
    pub refs_mut: ThinVec<Type>,
    pub slices: ThinVec<Type>,
    pub tuples: ThinVec<TupleType>,
    pub arrays: ThinVec<ArrayType>,
    pub lambdas: ThinVec<LambdaType>,
}

impl Default for Types {
    fn default() -> Self {
        let mut paths = ThinVec::default();

        // paths.push(ItemInPkg {
        //     pkg_idx: 0,
        //     id: PoolIdx::UNIT,
        // });

        Self {
            paths,
            ptrs: Default::default(),
            refs: Default::default(),
            ptrs_mut: Default::default(),
            refs_mut: Default::default(),
            slices: Default::default(),
            tuples: Default::default(),
            arrays: Default::default(),
            lambdas: Default::default(),
        }
    }
}

pub struct Type {
    pub kind_and_idx: TypeKindAndIndex,
    pub span: Span,
}

pub struct TypeKindAndIndex(u64);

impl TypeKindAndIndex {
    const KIND_BITS: u64 = 4;
    const KIND_SHIFT: u64 = 64 - Self::KIND_BITS;
    const KIND_MASK: u64 = 0b11 << Self::KIND_SHIFT;
    const INDEX_MASK: u64 = !Self::KIND_MASK;

    // Possible kinds
    pub const PATH: u64 = 0 << Self::KIND_SHIFT;
    pub const PTR: u64 = 1 << Self::KIND_SHIFT;
    pub const REF: u64 = 2 << Self::KIND_SHIFT;
    pub const PTR_MUT: u64 = 3 << Self::KIND_SHIFT;
    pub const REF_MUT: u64 = 4 << Self::KIND_SHIFT;
    pub const SLICE: u64 = 5 << Self::KIND_SHIFT;
    pub const TUPLE: u64 = 6 << Self::KIND_SHIFT;
    pub const ARRAY: u64 = 7 << Self::KIND_SHIFT;
    pub const LAMBDA: u64 = 8 << Self::KIND_SHIFT;

    // Create a new encoded value for a given kind and index
    pub fn new(kind: u64, index: usize) -> Self {
        Self(kind | index as u64)
    }

    // Decode the kind of the expression
    pub fn kind(self) -> u64 {
        self.0 & Self::KIND_MASK
    }

    // Decode the index of the expression
    pub fn index(self) -> usize {
        (self.0 & Self::INDEX_MASK) as usize
    }
}

pub struct TupleType {
    pub types: ThinVec<Type>,
    pub parens_span: Span,
}

pub struct ArrayType {
    pub typ: Type,
    pub size: Expr,
}

pub struct LambdaType {
    pub params: ThinVec<Type>,
    pub return_type: Type,
}
