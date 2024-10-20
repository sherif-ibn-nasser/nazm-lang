use crate::{stms::Binding, ConditionalScope, ModPathWithItem, NIRId, Scope};
use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::span::{Span, SpanCursor};
use thin_vec::ThinVec;

pub struct Expr {
    pub kind_and_index: ExprKindAndIndex,
    pub span: Span,
}

pub struct ExprKindAndIndex(u64);

impl ExprKindAndIndex {
    pub const KIND_BITS: u64 = 5;
    pub const KIND_SHIFT: u64 = 64 - Self::KIND_BITS;
    pub const KIND_MASK: u64 = 0b11111 << Self::KIND_SHIFT;
    pub const INDEX_MASK: u64 = !Self::KIND_MASK;

    // Possible kinds
    pub const UNIT: u64 = 0 << Self::KIND_SHIFT;
    pub const LITERAL: u64 = 1 << Self::KIND_SHIFT;
    pub const PARENS: u64 = 2 << Self::KIND_SHIFT;
    pub const PATH_CALL: u64 = 3 << Self::KIND_SHIFT;
    pub const METHOD_CALL: u64 = 4 << Self::KIND_SHIFT;
    pub const UNIT_STRUCT: u64 = 5 << Self::KIND_SHIFT;
    pub const TUPLE_STRUCT: u64 = 6 << Self::KIND_SHIFT;
    pub const FIELDS_STRUCT: u64 = 7 << Self::KIND_SHIFT;
    pub const FIELD: u64 = 8 << Self::KIND_SHIFT;
    pub const INDEX: u64 = 9 << Self::KIND_SHIFT;
    pub const ARRAY_ELEMENTS: u64 = 10 << Self::KIND_SHIFT;
    pub const ARRAY_ELEMENTS_SIZED: u64 = 11 << Self::KIND_SHIFT;
    pub const TUPLE_EXPR: u64 = 12 << Self::KIND_SHIFT;
    pub const RETURN_WITH_VALUE: u64 = 13 << Self::KIND_SHIFT;
    pub const IF_EXPR: u64 = 14 << Self::KIND_SHIFT;
    pub const LAMBDA_EXPR: u64 = 15 << Self::KIND_SHIFT;
    pub const UNARY_EXPR: u64 = 16 << Self::KIND_SHIFT;
    pub const BIN_EXPR: u64 = 17 << Self::KIND_SHIFT;

    // Create a new encoded value for a given kind and index
    pub fn new(kind: u64, index: usize) -> Self {
        Self(kind | index as u64)
    }

    // Decode the kind of the expression
    pub fn kind(self) -> u64 {
        self.0 & Self::KIND_MASK
    }

    // Decode the index of the expression
    pub fn index(self) -> u64 {
        self.0 & Self::INDEX_MASK
    }
}

#[derive(Default)]
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
    pub indexes: ThinVec<IndexExpr>,
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
    pub name: NIRId,
    pub expr: Expr,
}

pub struct FieldExpr {
    pub on: Expr,
    pub name: NIRId,
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
