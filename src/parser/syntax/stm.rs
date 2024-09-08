use super::*;

#[derive(NazmcParse)]
pub(crate) enum Stm {
    Let(LetStm),
    If(IfStm),
    Switch(SwitchStm),
    Loop(LoopStm),
    While(WhileStm),
    DoWhile(DoWhileStm),
    BLock(BlockStm),
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
pub(crate) struct IfStm {}

#[derive(NazmcParse)]
pub(crate) struct SwitchStm {}

#[derive(NazmcParse)]
pub(crate) struct LoopStm {}

#[derive(NazmcParse)]
pub(crate) struct WhileStm {}

#[derive(NazmcParse)]
pub(crate) struct DoWhileStm {}

#[derive(NazmcParse)]
pub(crate) struct BlockStm {}

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
