use super::*;

#[derive(NazmcParse)]
pub(crate) enum Stm {
    Semicolon(SemicolonSymbol),
    Let(LetStm),
    Break(BreakStm),
    Continue(ContinueStm),
    Return(ReturnStm),
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
pub(crate) struct BreakStm {
    pub(crate) break_keyowrd: SyntaxNode<BreakKeyword>,
    pub(crate) expr: Optional<Expr>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct ContinueStm {
    pub(crate) continue_keyowrd: SyntaxNode<ContinueKeyword>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct ReturnStm {
    pub(crate) return_keyowrd: SyntaxNode<ReturnKeyword>,
    pub(crate) expr: Optional<Expr>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
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
    pub(crate) assign: Optional<Assign>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct Assign {
    pub(crate) op: SyntaxNode<AssignOp>,
    pub(crate) right: ParseResult<Expr>,
}
