use super::*;

#[derive(NazmcParse, Debug)]
pub enum Stm {
    Semicolon(SemicolonSymbol),
    Let(LetStm),
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
pub enum ExprStm {
    WithBlock(ExprWithBlockStm),
    Any(AnyExprStm),
}

#[derive(NazmcParse, Debug)]
pub struct ExprWithBlockStm {
    pub expr: ExprWithBlock,
    pub semicolon: Option<SemicolonSymbol>,
}

#[derive(NazmcParse, Debug)]
pub struct AnyExprStm {
    pub expr: Expr,
    pub semicolon: ParseResult<SemicolonSymbol>,
}
