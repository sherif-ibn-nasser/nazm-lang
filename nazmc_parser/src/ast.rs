use std::sync::Arc;

use nazmc_diagnostics::span::{Span, SpanCursor};
use nazmc_lexer::LiteralKind;
use thin_vec::ThinVec;

pub struct File {
    pub imports: ThinVec<ModPathWithItem>,
    pub star_imports: ThinVec<ModPath>,
    pub unit_structs: ThinVec<UnitStruct>,
    pub tuple_structs: ThinVec<TupleStruct>,
    pub fields_structs: ThinVec<FieldsStruct>,
    pub fns: ThinVec<Fn>,
}

pub struct ModPath {
    pub ids: ThinVec<Arc<String>>,
    pub spans: ThinVec<Span>,
}

pub struct ModPathWithItem {
    pub mod_path: ModPath,
    pub item: ASTId,
}

pub struct ASTId {
    pub span: Span,
    pub id: Arc<String>,
}

pub struct Binding {
    pub kind: BindingKind,
    pub typ: Type,
}

pub enum BindingKind {
    Id(ASTId),
    Tuple(ThinVec<BindingKind>, Span),
}

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

pub enum VisModifier {
    Default,
    Public,
    Private,
}

pub struct UnitStruct {
    pub vis: VisModifier,
    pub name: ASTId,
}

pub struct TupleStruct {
    pub vis: VisModifier,
    pub name: ASTId,
    pub types: ThinVec<(VisModifier, Type)>,
}

pub struct FieldsStruct {
    pub vis: VisModifier,
    pub name: ASTId,
    pub fields: ThinVec<(VisModifier, ASTId, Type)>,
}

pub struct Fn {
    pub vis: VisModifier,
    pub name: ASTId,
    pub params: ThinVec<(ASTId, Type)>,
    pub return_type: Type,
    pub body: Scope,
}

pub struct Scope {
    pub stms: ThinVec<Stm>,
    pub return_expr: Option<Expr>,
}

pub enum Stm {
    Let(Box<LetStm>),
    LetMut(Box<LetStm>),
    While(Box<(Expr, Scope)>),
    If(Box<IfExpr>),
    Expr(Box<Expr>),
}

pub struct LetStm {
    pub binding: Binding,
    pub assign: Option<Box<Expr>>,
}

pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

pub enum ExprKind {
    Literal(LiteralKind),
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

pub struct CallExpr {
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
    pub fields: ThinVec<(ASTId, Expr)>,
}

pub struct FieldExpr {
    pub on: Expr,
    pub name: ASTId,
}

pub struct IndexExpr {
    pub on: Expr,
    pub index: Expr,
    pub brackets_span: Span,
}

pub struct ArrayElementsSizedExpr {
    pub repeat: Expr,
    pub size: Expr,
}

pub struct IfExpr {
    pub if_: (Expr, Scope),
    pub else_ifs: ThinVec<(Expr, Scope)>,
    pub else_: Option<Box<Scope>>,
}

pub struct LambdaExpr {
    pub params: LambdaParams,
    pub body: Scope,
}

pub enum LambdaParams {
    Implicit,
    Explicit(ThinVec<Binding>),
}

pub struct UnaryOpExpr {
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

pub struct BinaryOpExpr {
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
