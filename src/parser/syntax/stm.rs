use super::*;

#[derive(NazmcParse)]
pub(crate) enum Stm {
    Semicolon(SemicolonSymbol),
    Let(LetStm),
    If(IfStm),
    Switch(WhenStm),
    Loop(LoopStm),
    While(WhileStm),
    DoWhile(DoWhileStm),
    BLock(BlockStm),
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
pub(crate) struct IfStm {
    pub(crate) if_keyword: SyntaxNode<IfKeyword>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct ElseIfClause {
    pub(crate) else_keyword: SyntaxNode<ElseKeyword>,
    pub(crate) if_keyword: SyntaxNode<IfKeyword>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct ElseClause {
    pub(crate) else_keyword: SyntaxNode<ElseKeyword>,
}

#[derive(NazmcParse)]
pub(crate) struct WhenStm {
    pub(crate) when_keyword: SyntaxNode<WhenKeyword>,
    pub(crate) expr: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct LoopStm {
    pub(crate) loop_keyword: SyntaxNode<LoopKeyword>,
}

#[derive(NazmcParse)]
pub(crate) struct WhileStm {
    pub(crate) while_keyword: SyntaxNode<WhileKeyword>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct DoWhileStm {
    pub(crate) do_keyword: SyntaxNode<DoKeyword>,
    pub(crate) while_keyword: ParseResult<WhileKeyword>,
    pub(crate) condition: ParseResult<Expr>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct BlockStm {}

#[derive(NazmcParse)]
pub(crate) struct BreakStm {
    pub(crate) break_keyowrd: SyntaxNode<BreakKeyword>,
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
pub(crate) struct ExprStm {
    pub(crate) expr: SyntaxNode<Expr>,
    pub(crate) assign: Optional<Assign>,
    pub(crate) semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct Assign {
    pub(crate) op: SyntaxNode<AssignOp>,
    pub(crate) right: ParseResult<Expr>,
}
