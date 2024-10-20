use nazmc_data_pool::PoolIdx;
use nazmc_diagnostics::span::{Span, SpanCursor};
use thin_vec::ThinVec;

#[derive(Clone)]
pub struct File {
    pub imports: ThinVec<ModPathWithItem>,
    pub star_imports: ThinVec<ModPath>,
    pub unit_structs: ThinVec<UnitStruct>,
    pub tuple_structs: ThinVec<TupleStruct>,
    pub fields_structs: ThinVec<FieldsStruct>,
    pub fns: ThinVec<Fn>,
}

#[derive(Clone)]
pub struct ModPath {
    pub ids: ThinVec<PoolIdx>,
    pub spans: ThinVec<Span>,
}

#[derive(Clone)]
pub struct ModPathWithItem {
    pub mod_path: ModPath,
    pub item: ASTId,
}

#[derive(Clone)]
pub struct ASTId {
    pub span: Span,
    pub id: PoolIdx,
}

#[derive(Clone)]
pub struct Binding {
    pub kind: BindingKind,
    pub typ: Option<Type>,
}

#[derive(Clone)]
pub enum BindingKind {
    Id(ASTId),
    Tuple(ThinVec<BindingKind>, Span),
}

#[derive(Clone)]
pub enum Type {
    Path(ModPathWithItem),
    Unit(Option<Span>),
    Tuple(ThinVec<Type>, Span),
    Paren(Box<Type>, Span),
    Slice(Box<Type>, Span),
    Array(Box<Type>, Box<Expr>, Span),
    Ptr(Box<Type>, Span),
    Ref(Box<Type>, Span),
    PtrMut(Box<Type>, Span),
    RefMut(Box<Type>, Span),
    Lambda(ThinVec<Type>, Box<Type>),
}

#[derive(Clone)]
pub enum VisModifier {
    Default,
    Public,
    Private,
}

#[derive(Clone)]
pub struct UnitStruct {
    pub vis: VisModifier,
    pub name: ASTId,
}

#[derive(Clone)]
pub struct TupleStruct {
    pub vis: VisModifier,
    pub name: ASTId,
    pub types: ThinVec<(VisModifier, Type)>,
}

#[derive(Clone)]
pub struct FieldsStruct {
    pub vis: VisModifier,
    pub name: ASTId,
    pub fields: ThinVec<(VisModifier, ASTId, Type)>,
}

#[derive(Clone)]
pub struct Fn {
    pub vis: VisModifier,
    pub name: ASTId,
    pub params: ThinVec<(ASTId, Type)>,
    pub return_type: Type,
    pub body: Scope,
}

#[derive(Clone)]
pub struct Scope {
    pub stms: ThinVec<Stm>,
    pub return_expr: Option<Expr>,
}

#[derive(Clone)]
pub enum Stm {
    Let(Box<LetStm>),
    LetMut(Box<LetStm>),
    While(Box<(Expr, Scope)>),
    If(Box<IfExpr>),
    Expr(Box<Expr>),
}

#[derive(Clone)]
pub struct LetStm {
    pub binding: Binding,
    pub assign: Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Clone)]
pub enum ExprKind {
    Literal(LiteralExpr),
    Parens(Box<Expr>),
    Path(Box<ModPathWithItem>),
    Call(Box<CallExpr>),
    UnitStruct(Box<ModPathWithItem>),
    TupleStruct(Box<TupleStructExpr>),
    FieldsStruct(Box<FieldsStructExpr>),
    Field(Box<FieldExpr>),
    Index(Box<IndexExpr>),
    Tuple(ThinVec<Expr>),
    ArrayElemnts(ThinVec<Expr>),
    ArrayElemntsSized(Box<ArrayElementsSizedExpr>),
    If(Box<IfExpr>),
    Lambda(Box<LambdaExpr>),
    UnaryOp(Box<UnaryOpExpr>),
    BinaryOp(Box<BinaryOpExpr>),
    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,
    On,
}

#[derive(Clone)]
pub enum LiteralExpr {
    Str(PoolIdx),
    Char(char),
    Bool(bool),
    Num(NumKind),
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct CallExpr {
    pub on: Expr,
    pub args: ThinVec<Expr>,
    pub parens_span: Span,
}

#[derive(Clone)]
pub struct TupleStructExpr {
    pub path: ModPathWithItem,
    pub args: ThinVec<Expr>,
}

#[derive(Clone)]
pub struct FieldsStructExpr {
    pub path: ModPathWithItem,
    pub fields: ThinVec<(ASTId, Expr)>,
}

#[derive(Clone)]
pub struct FieldExpr {
    pub on: Expr,
    pub name: ASTId,
}

#[derive(Clone)]
pub struct IndexExpr {
    pub on: Expr,
    pub index: Expr,
    pub brackets_span: Span,
}

#[derive(Clone)]
pub struct ArrayElementsSizedExpr {
    pub repeat: Expr,
    pub size: Expr,
}

#[derive(Clone)]
pub struct IfExpr {
    pub if_: (Expr, Scope),
    pub else_ifs: ThinVec<(Expr, Scope)>,
    pub else_: Option<Box<Scope>>,
}

#[derive(Clone)]
pub struct LambdaExpr {
    pub params: ThinVec<Binding>,
    pub body: Scope,
}

#[derive(Clone)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub op_span: Span,
    pub expr: Expr,
}

#[derive(Clone)]
pub enum UnaryOp {
    Minus,
    LNot,
    BNot,
    Deref,
    Borrow,
    BorrowMut,
}

#[derive(Clone)]
pub struct BinaryOpExpr {
    pub op: BinOp,
    pub op_span_cursor: SpanCursor,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Clone)]
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
