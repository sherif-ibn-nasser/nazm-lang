use crate::{stms::Binding, ASTId, ConditionalScope, ModPathWithItem, Scope};
use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::span::{Span, SpanCursor};
use thin_vec::ThinVec;

pub struct Expr {
    pub kind_and_index: u64,
    pub span: Span,
}

pub struct Exprs {
    pub literals: ThinVec<LiteralExpr>,
    pub parens: ThinVec<ParensExpr>,
    pub paths: ThinVec<ModPathWithItem>,
    pub path_calls: ThinVec<PathCallExpr>,
    pub method_calls: ThinVec<MethodCallExpr>,
    pub unit_structs: ThinVec<ModPathWithItem>,
    pub tuple_structs: ThinVec<TupleStructExpr>,
    pub fields_structs: ThinVec<FieldsStructExpr>,
    pub fields: ThinVec<FieldExpr>,
    pub indecies: ThinVec<IndexExpr>,
    pub array_elements: ThinVec<ArrayElementsExpr>,
    pub array_elements_sized: ThinVec<ArrayElementsSizedExpr>,
    pub tuples: ThinVec<TupleExpr>,
    pub returns: ThinVec<ReturWithValueExpr>,
    pub ifs: ThinVec<IfExpr>,
    pub lambdas: ThinVec<LambdaExpr>,
    pub unary_exprs: ThinVec<UnaryExpr>,
    pub bin_exprs: ThinVec<BinExpr>,
}

pub enum LiteralExpr {
    Str(PoolIdx),
    Char(char),
    Bool(bool),
    Num(NumKind),
}

pub enum NumKind {
    F4(f32),
    F8(f64),
    I(isize),
    I1(i8),
    I2(i16),
    I4(i32),
    I8(i64),
    U(usize),
    U1(u8),
    U2(u16),
    U4(u32),
    U8(u64),
    UnspecifiedInt(u64),
    UnspecifiedFloat(f64),
}

pub struct ParensExpr {
    pub expr: Expr,
}

pub struct PathCallExpr {
    pub path: ModPathWithItem,
    pub args: ThinVec<Expr>,
    pub parens_span: Span,
}

pub struct MethodCallExpr {
    pub on: Expr,
    pub args: ThinVec<Expr>,
    pub parens_span: Span,
}

pub struct TupleStructExpr {
    pub path: ModPathWithItem,
    pub args: ThinVec<Expr>,
}

pub struct FieldsStructExpr {
    pub path: ModPathWithItem,
    pub fields: ThinVec<FieldInStructExpr>,
}

pub struct FieldInStructExpr {
    pub name: ASTId,
    pub expr: Expr,
}

pub struct FieldExpr {
    pub on: Expr,
    pub name: ASTId,
}

pub struct IndexExpr {
    pub on: Expr,
    pub idx: Expr,
    pub brackets_span: Span,
}

pub struct ArrayElementsExpr {
    pub elements: ThinVec<Expr>,
}

pub struct ArrayElementsSizedExpr {
    pub repeat: Expr,
    pub size: Expr,
}

pub struct TupleExpr {
    pub elements: ThinVec<Expr>,
}

pub struct ReturWithValueExpr {
    pub expr_to_return: Expr,
}

pub struct IfExpr {
    pub if_: ConditionalScope,
    pub else_ifs: ThinVec<ConditionalScope>,
    pub else_: Option<Scope>,
}

pub struct LambdaExpr {
    pub param: ThinVec<Binding>,
    pub body: Scope,
}

pub struct UnaryExpr {
    pub op: UnaryOp,
    pub op_span: Span,
    pub expr: Expr,
}

pub enum UnaryOp {
    Minus,
    LNot,
    BNot,
    Deref,
    Borrow,
    BorrowMut,
}

pub struct BinExpr {
    pub op: BinOp,
    pub op_span_cursor: SpanCursor,
    pub left: Expr,
    pub right: Expr,
}

pub enum BinOp {
    LOr,
    LAnd,
    EqualEqual,
    NotEqual,
    GE,
    GT,
    LE,
    LT,
    OpenOpenRange,
    CloseOpenRange,
    OpenCloseRange,
    CloseCloseRange,
    BOr,
    Xor,
    BAnd,
    Shr,
    Shl,
    Plus,
    Minus,
    Times,
    Div,
    Mod,
    Assign,
    PlusAssign,
    MinusAssign,
    TimesAssign,
    DivAssign,
    ModAssign,
    BAndAssign,
    BOrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
}
