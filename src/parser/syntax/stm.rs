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
    pub(crate) decl: ParseResult<BindingDecl>,
    pub(crate) let_assign: Option<LetAssign>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct BindingDecl {
    pub(crate) kind: BindingDeclKind,
    pub(crate) typ: Option<ColonWithType>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum BindingDeclKind {
    Id(Id),
    Destructed(Box<DestructedTuple>), // Box for the large size
}

generatePunctuatedItem!(BindingDecl);

generatePunctuatedItem!(BindingDeclKind);

generateDelimitedPunctuated!(
    DestructedTuple,
    OpenParenthesisSymbol,
    BindingDeclKind,
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
