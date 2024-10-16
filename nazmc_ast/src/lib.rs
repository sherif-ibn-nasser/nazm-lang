use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::span::Span;

pub struct ASTId {
    pub span: Span,
    pub id: PoolIdx,
}

pub struct AST {
    pub mods: Vec<Mod>,
    pub types: Types,
    pub unit_structs: Vec<UnitStruct>,
    pub tuple_structs: Vec<TupleStruct>,
    pub fields_structs: Vec<FieldsStruct>,
    pub fns: Vec<Fn>,
    pub scopes: Vec<ScopeBody>,
    pub stms: Stms,
    pub exprs: Exprs,
}

pub struct Mod {
    pub path: Vec<PoolIdx>,
}

pub struct UnitStruct {
    pub mod_index: usize,
    pub name: ASTId,
}

pub struct TupleStruct {
    pub mod_index: usize,
    pub name: ASTId,
    pub parmas: Vec<Ty>,
}

pub struct FieldsStruct {
    pub mod_index: usize,
    pub name: ASTId,
    pub fields: Vec<(ASTId, Ty)>,
}

pub struct Fn {
    pub mod_index: usize,
    pub name: ASTId,
    pub params: Vec<(ASTId, Ty)>,
    pub return_ty: Ty,
    pub scope: Scope,
}

pub struct ScopeBody {
    pub stms: Vec<Stm>,
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

pub struct Types {
    pub paths: Vec<PathType>,
    pub ptrs: Vec<Ty>,
    pub refs: Vec<Ty>,
    pub slices: Vec<Ty>,
    pub arrays: Vec<ArrayType>,
}

pub struct Stms {
    pub lets: Vec<LetStm>,
    pub exprs: Vec<Expr>,
    // pub whiles: Vec< ()>,
}

pub struct Exprs {
    pub str_lits: Vec<PoolIdx>,
    pub char_lits: Vec<char>,
    pub bool_lits: Vec<bool>,
    pub f32_lits: Vec<f32>,
    pub f64_lits: Vec<f64>,
    pub i_lits: Vec<isize>,
    pub i1_lits: Vec<i8>,
    pub i2_lits: Vec<i16>,
    pub i4_lits: Vec<i32>,
    pub i8_lits: Vec<i64>,
    pub usize_lits: Vec<usize>,
    pub u1_lits: Vec<u8>,
    pub u2_lits: Vec<u16>,
    pub u4_lits: Vec<u32>,
    pub u8_lits: Vec<u64>,
    pub unspecified_u_lits: Vec<u64>,
    pub unspecified_f_lits: Vec<f64>,
    pub paths: Vec<PathExpr>,
    pub calls: Vec<CallExpr>,
    pub unit_structs: Vec<UnitStructExpr>,
    pub tuple_structs: Vec<TupleStructExpr>,
    pub fields_structs: Vec<FieldsStructExpr>,
    pub fields: Vec<FieldExpr>,
    pub indecies: Vec<IndexExpr>,
    pub array_elements: Vec<Vec<Expr>>,
    pub array_elements_sized: Vec<ArrayElementsSized>,
    pub tuples: Vec<Vec<Expr>>,
    pub parens: Vec<Expr>,
    pub returns_w_exprs: Vec<Expr>,
    pub ifs: Vec<IfExpr>,
    pub lambdas: Vec<LambdaExpr>,
}

pub struct LetStm {
    binding: Binding,
    typ: Option<Ty>,
    init: Option<Expr>,
}

pub enum Binding {
    Name(ASTId),
    TupleDestruction(Vec<Binding>),
}

pub struct PathType {
    dist: Vec<ASTId>,
    name: ASTId,
}

pub struct ArrayType {
    pub ty: Ty,
    pub size: Expr,
}

pub struct PathExpr {
    dist: Vec<ASTId>,
    name: ASTId,
}

pub struct CallExpr {
    path: PathExpr,
    args: Vec<Expr>,
}

pub struct UnitStructExpr {
    path: PathExpr,
}

pub struct TupleStructExpr {
    path: PathExpr,
    args: Vec<Expr>,
}

pub struct FieldsStructExpr {
    path: PathExpr,
    fields: Vec<FieldInStructExpr>,
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

pub struct IfExpr {
    if_: (Expr, Scope),
    else_ifs: Vec<(Expr, Scope)>,
    else_: Option<Scope>,
}

pub struct LambdaExpr {
    param: Vec<LambdaParam>,
    body: Scope,
}

pub struct LambdaParam {
    binding: Binding,
    ty: Option<Ty>,
}
