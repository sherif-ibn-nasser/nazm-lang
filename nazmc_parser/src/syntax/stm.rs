use super::*;

#[derive(NazmcParse, Debug)]
pub enum Stm {
    Semicolon(SemicolonSymbol),
    Let(LetStm),
    While(WhileStm),
    If(IfExpr),
    When(WhenExpr),
    Expr(ExprStm),
}

#[derive(NazmcParse, Debug)]
pub struct LetStm {
    pub let_keyword: LetKeyword,
    pub mut_keyword: Option<MutKeyword>,
    pub binding: ParseResult<Binding>,
    pub let_assign: Option<LetAssign>,
    pub semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse, Debug)]
pub struct Binding {
    pub kind: BindingKind,
    pub typ: Option<ColonWithType>,
}

#[derive(NazmcParse, Debug)]
pub enum BindingKind {
    Id(Id),
    Destructed(Box<DestructedTuple>), // Box for the large size
}

generatePunctuatedItem!(Binding);

generatePunctuatedItem!(BindingKind);

generateDelimitedPunctuated!(
    DestructedTuple,
    OpenParenthesisSymbol,
    BindingKind,
    CloseParenthesisSymbol
);

#[derive(NazmcParse, Debug)]
pub struct ColonWithType {
    pub colon: ColonSymbol,
    pub typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub struct LetAssign {
    pub equal: EqualSymbol,
    pub expr: ParseResult<Expr>,
}

#[derive(NazmcParse, Debug)]
pub struct WhileStm {
    pub(crate) while_keyword: WhileKeyword,
    pub(crate) conditional_block: ConditionalBlock,
}

#[derive(NazmcParse, Debug)]
pub struct DoWhileStm {
    // TODO
    pub(crate) do_keyword: DoKeyword,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
    pub(crate) while_keyword: ParseResult<WhileKeyword>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse, Debug)]
pub struct ExprStm {
    pub expr: Expr,
    pub semicolon: ParseResult<SemicolonSymbol>,
}
