use super::*;

#[derive(NazmcParse, Debug)]
pub(crate) enum Stm {
    Semicolon(SemicolonSymbol),
    Let(LetStm),
    Expr(ExprStm),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct LetStm {
    pub(crate) let_keyword: LetKeyword,
    pub(crate) mut_keyword: Option<MutKeyword>,
    pub(crate) binding: ParseResult<Binding>,
    pub(crate) let_assign: Option<LetAssign>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct Binding {
    pub(crate) kind: BindingKind,
    pub(crate) typ: Option<ColonWithType>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum BindingKind {
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
pub(crate) struct ColonWithType {
    pub(crate) colon: ColonSymbol,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct LetAssign {
    pub(crate) equal: EqualSymbol,
    pub(crate) expr: ParseResult<Expr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum ExprStm {
    WithBlock(ExprWithBlockStm),
    Any(AnyExprStm),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct ExprWithBlockStm {
    pub(crate) expr: ExprWithBlock,
    pub(crate) semicolon: Option<SemicolonSymbol>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct AnyExprStm {
    pub(crate) expr: Expr,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}
