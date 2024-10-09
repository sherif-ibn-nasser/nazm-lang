use nazmc_data_pool::PoolIdx;

pub(crate) enum Type {
    Path { path: Vec<PoolIdx>, name: PoolIdx },
    Ptr(Box<Type>),
    Ref(Box<Type>),
    Slice(Box<Type>),
    Array { typ: Box<Type>, size: Box<Expr> },
}

pub(crate) struct Scope {
    stms: Vec<Stm>,
    return_expr: Option<Box<Expr>>,
}

pub(crate) enum Stm {
    Let {
        name: PoolIdx,
        typ: Option<Type>,
        init: Option<Box<Expr>>,
    },
    Expr(Expr),
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
    Path {
        dist: Vec<PoolIdx>,
        name: PoolIdx,
    },
    Call {
        dist: Vec<PoolIdx>,
        name: PoolIdx,
        args: Vec<Expr>,
    },
    Struct {
        dist: Vec<PoolIdx>,
        name: PoolIdx,
        kind: StructExprKind,
    },
    Field {
        on: Box<Expr>,
        name: PoolIdx,
    },
    Index {
        on: Box<Expr>,
        idx: Box<Expr>,
    },
    ArrayElements(Vec<Expr>),
    ArrayElementsSized {
        repeat: Box<Expr>,
        size: Box<Expr>,
    },
    Tuple(Vec<Expr>),
    Paren(Box<Expr>),
    Break,
    Continue,
    Return(Option<Box<Expr>>),
    If {
        if_: (Box<Expr>, Scope),
        else_ifs: Vec<(Expr, Scope)>,
        else_: Scope,
    },
    Lambda {
        param: Vec<LambdaParam>,
        body: Scope,
    },
}

pub(crate) enum StructExprKind {
    Unit,
    Tuple { args: Vec<Expr> },
    Struct { fields: Vec<Expr> },
}

pub(crate) struct LambdaParam {
    name: PoolIdx,
    typ: Option<Type>,
}
