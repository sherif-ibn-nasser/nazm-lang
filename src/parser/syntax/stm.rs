use super::*;

#[derive(NazmcParse)]
pub(crate) enum Stm {
    Semicolon(SemicolonSymbol),
    Let(LetStm),
    Expr(ExprStm),
}

#[derive(NazmcParse)]
pub(crate) struct LetStm {
    pub(crate) let_keyword: SyntaxNode<LetKeyword>,
    pub(crate) mut_keyword: Optional<MutKeyword>,
    pub(crate) decl: ParseResult<BindingDecl>,
    pub(crate) let_assign: Optional<LetAssign>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct BindingDecl {
    pub(crate) kind: SyntaxNode<BindingDeclKind>,
    pub(crate) typ: Optional<ColonWithType>,
}

#[derive(NazmcParse)]
pub(crate) struct LetAssign {
    pub(crate) equal: SyntaxNode<EqualSymbol>,
    pub(crate) expr: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) enum ExprStm {
    WithBlock(ExprWithBlockStm),
    Any(AnyExprStm),
}

#[derive(NazmcParse)]
pub(crate) struct ExprWithBlockStm {
    pub(crate) expr: SyntaxNode<ExprWithBlock>,
    pub(crate) semicolon: Optional<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct AnyExprStm {
    pub(crate) expr: SyntaxNode<Expr>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}
