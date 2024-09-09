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
    pub(crate) id: ParseResult<Id>,
    pub(crate) let_type: Optional<LetType>,
    pub(crate) let_assign: Optional<LetAssign>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct LetType {
    pub(crate) colon: SyntaxNode<ColonSymbol>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse)]
pub(crate) struct LetAssign {
    pub(crate) equal: SyntaxNode<EqualSymbol>,
    pub(crate) expr: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) enum ExprStm {
    WithBlock(ExprWithBlockStm),
    WithoutBlock(ExprWithBlockStm),
}

#[derive(NazmcParse)]
pub(crate) struct ExprWithBlockStm {
    pub(crate) expr: SyntaxNode<ExprWithBlock>,
    pub(crate) semicolon: Optional<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct ExprWithoutBlockStm {
    pub(crate) expr: SyntaxNode<ExprWithoutBlock>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}
