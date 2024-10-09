use nazmc_data_pool::PoolIdx;
use thin_vec::ThinVec;

pub(crate) enum Type {
    Path(Box<PathType>),
    Ptr(Box<Type>),
    Ref(Box<Type>),
    Slice(Box<Type>),
    Array(Box<ArrayType>),
}

pub(crate) struct PathType {
    path: ThinVec<PoolIdx>,
    name: PoolIdx,
}

pub(crate) struct ArrayType {
    typ: Box<Type>,
    size: usize,
}

pub(crate) struct Scope {
    stms: ThinVec<Stm>,
    return_expr: Option<Expr>,
}

pub(crate) enum Stm {
    Let(Box<LetStm>),
    Expr(Box<Expr>),
}

pub(crate) struct LetStm {
    binding: Binding,
    typ: Option<Type>,
    init: Option<Expr>,
}

pub(crate) enum Binding {
    Name(PoolIdx),
    TupleDestruction(ThinVec<Binding>),
}

pub(crate) enum Expr {
    StrLit(PoolIdx),
    CharLit(char),
    BoolLit(bool),
    F4Lit(f32),
    F8Lit(f64),
    ILit(isize),
    I1Lit(i8),
    I2Lit(i16),
    I4Lit(i32),
    I8Lit(i64),
    ULit(usize),
    U1Lit(u8),
    U2Lit(u16),
    U4Lit(u32),
    U8Lit(u64),
    UnspecifiedIntLit(u64),
    UnspecifiedFloatLit(f64),
    Path(Box<PathExpr>),
    Call(Box<CallExpr>),
    UnitStruct(Box<UnitStructExpr>),
    TupleStruct(Box<TupleStructExpr>),
    FieldsStruct(Box<FieldsStructExpr>),
    Field(Box<FieldExpr>),
    Index(Box<IndexExpr>),
    ArrayElements(ThinVec<Expr>),
    ArrayElementsSized(Box<ArrayElementsSized>),
    Tuple(ThinVec<Expr>),
    Paren(Box<Expr>),
    Break,
    Continue,
    Return(Box<Option<Expr>>),
    If(Box<IfExpr>),
    Lambda(Box<LambdaExpr>),
}

pub(crate) struct PathExpr {
    dist: ThinVec<PoolIdx>,
    name: PoolIdx,
}

pub(crate) struct CallExpr {
    dist: ThinVec<PoolIdx>,
    name: PoolIdx,
    args: ThinVec<Expr>,
}

pub(crate) struct UnitStructExpr {
    dist: ThinVec<PoolIdx>,
    name: PoolIdx,
}

pub(crate) struct TupleStructExpr {
    dist: ThinVec<PoolIdx>,
    name: PoolIdx,
    args: ThinVec<Expr>,
}

pub(crate) struct FieldsStructExpr {
    dist: ThinVec<PoolIdx>,
    name: PoolIdx,
    fields: ThinVec<Expr>,
}

pub(crate) struct FieldExpr {
    on: Expr,
    name: PoolIdx,
}

pub(crate) struct IndexExpr {
    on: Expr,
    idx: Expr,
}

pub(crate) struct ArrayElementsSized {
    repeat: Expr,
    size: Expr,
}

pub(crate) struct IfExpr {
    if_: (Expr, Scope),
    else_ifs: ThinVec<(Expr, Scope)>,
    else_: Option<Scope>,
}

pub(crate) struct LambdaExpr {
    param: ThinVec<LambdaParam>,
    body: Scope,
}

pub(crate) struct LambdaParam {
    binding: Binding,
    typ: Option<Type>,
}
