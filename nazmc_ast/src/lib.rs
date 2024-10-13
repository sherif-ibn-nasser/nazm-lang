use bumpalo::collections::Vec as BumpVec;
use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::span::Span;

pub struct ASTId {
    pub span: Span,
    pub id: PoolIdx,
}

pub struct AST<'a> {
    pub mods: BumpVec<'a, Mod<'a>>,
    pub types: Types<'a>,
    pub unit_structs: BumpVec<'a, UnitStruct>,
    pub tuple_structs: BumpVec<'a, TupleStruct<'a>>,
    pub fields_structs: BumpVec<'a, FieldsStruct<'a>>,
    pub fns: BumpVec<'a, Fn<'a>>,
    pub scopes: BumpVec<'a, ScopeBody<'a>>,
    pub stms: Stms<'a>,
    pub exprs: Exprs<'a>,
}

pub struct Mod<'a> {
    pub path: BumpVec<'a, PoolIdx>,
}

pub struct UnitStruct {
    pub mod_index: usize,
    pub name: ASTId,
}

pub struct TupleStruct<'a> {
    pub mod_index: usize,
    pub name: ASTId,
    pub parmas: BumpVec<'a, Ty>,
}

pub struct FieldsStruct<'a> {
    pub mod_index: usize,
    pub name: ASTId,
    pub fields: BumpVec<'a, (ASTId, Ty)>,
}

pub struct Fn<'a> {
    pub mod_index: usize,
    pub name: ASTId,
    pub params: BumpVec<'a, (ASTId, Ty)>,
    pub return_ty: Ty,
    pub scope: Scope,
}

pub struct ScopeBody<'a> {
    pub stms: BumpVec<'a, Stm>,
    pub return_expr: Option<Expr>,
}

pub struct Scope {
    pub index: u64,
}

pub struct Ty {
    pub kind_and_index: u64,
    pub span: Span,
}

pub struct Stm {
    pub kind_and_index: u64,
}

pub struct Expr {
    pub kind_and_index: u64,
    pub span: Span,
}

pub struct Types<'a> {
    pub paths: BumpVec<'a, PathType<'a>>,
    pub ptrs: BumpVec<'a, Ty>,
    pub refs: BumpVec<'a, Ty>,
    pub slices: BumpVec<'a, Ty>,
    pub arrays: BumpVec<'a, ArrayType>,
}

pub struct Stms<'a> {
    pub lets: BumpVec<'a, LetStm<'a>>,
    pub exprs: BumpVec<'a, Expr>,
    // pub whiles: BumpVec<'a, ()>,
}

pub struct Exprs<'a> {
    pub str_lits: BumpVec<'a, PoolIdx>,
    pub char_lits: BumpVec<'a, char>,
    pub bool_lits: BumpVec<'a, bool>,
    pub f32_lits: BumpVec<'a, f32>,
    pub f64_lits: BumpVec<'a, f64>,
    pub i_lits: BumpVec<'a, isize>,
    pub i1_lits: BumpVec<'a, i8>,
    pub i2_lits: BumpVec<'a, i16>,
    pub i4_lits: BumpVec<'a, i32>,
    pub i8_lits: BumpVec<'a, i64>,
    pub usize_lits: BumpVec<'a, usize>,
    pub u1_lits: BumpVec<'a, u8>,
    pub u2_lits: BumpVec<'a, u16>,
    pub u4_lits: BumpVec<'a, u32>,
    pub u8_lits: BumpVec<'a, u64>,
    pub unspecified_u_lits: BumpVec<'a, u64>,
    pub unspecified_f_lits: BumpVec<'a, f64>,
    pub paths: BumpVec<'a, PathExpr<'a>>,
    pub calls: BumpVec<'a, CallExpr<'a>>,
    pub unit_structs: BumpVec<'a, UnitStructExpr<'a>>,
    pub tuple_structs: BumpVec<'a, TupleStructExpr<'a>>,
    pub fields_structs: BumpVec<'a, FieldsStructExpr<'a>>,
    pub fields: BumpVec<'a, FieldExpr>,
    pub indecies: BumpVec<'a, IndexExpr>,
    pub array_elements: BumpVec<'a, BumpVec<'a, Expr>>,
    pub array_elements_sized: BumpVec<'a, ArrayElementsSized>,
    pub tuples: BumpVec<'a, BumpVec<'a, Expr>>,
    pub parens: BumpVec<'a, Expr>,
    pub returns_w_exprs: BumpVec<'a, Expr>,
    pub ifs: BumpVec<'a, IfExpr<'a>>,
    pub lambdas: BumpVec<'a, LambdaExpr<'a>>,
}

pub struct LetStm<'a> {
    binding: Binding<'a>,
    typ: Option<Ty>,
    init: Option<Expr>,
}

pub enum Binding<'a> {
    Name(ASTId),
    TupleDestruction(BumpVec<'a, Binding<'a>>),
}

pub struct PathType<'a> {
    dist: BumpVec<'a, ASTId>,
    name: ASTId,
}

pub struct ArrayType {
    pub ty: Ty,
    pub size: Expr,
}

pub struct PathExpr<'a> {
    dist: BumpVec<'a, ASTId>,
    name: ASTId,
}

pub struct CallExpr<'a> {
    path: PathExpr<'a>,
    args: BumpVec<'a, Expr>,
}

pub struct UnitStructExpr<'a> {
    path: PathExpr<'a>,
}

pub struct TupleStructExpr<'a> {
    path: PathExpr<'a>,
    args: BumpVec<'a, Expr>,
}

pub struct FieldsStructExpr<'a> {
    path: PathExpr<'a>,
    fields: BumpVec<'a, FieldInStructExpr>,
}

pub struct FieldInStructExpr {
    name: ASTId,
    val: Expr,
}

pub struct FieldExpr {
    on: Expr,
    name: ASTId,
}

pub struct IndexExpr {
    on: Expr,
    idx: Expr,
}

pub struct ArrayElementsSized {
    repeat: Expr,
    size: Expr,
}

pub struct IfExpr<'a> {
    if_: (Expr, Scope),
    else_ifs: BumpVec<'a, (Expr, Scope)>,
    else_: Option<Scope>,
}

pub struct LambdaExpr<'a> {
    param: BumpVec<'a, LambdaParam<'a>>,
    body: Scope,
}

pub struct LambdaParam<'a> {
    binding: Binding<'a>,
    ty: Option<Ty>,
}
