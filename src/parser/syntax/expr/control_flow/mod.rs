use super::*;

mod block;
mod conditional;
use block::*;
use conditional::*;

#[derive(NazmcParse)]
pub(crate) struct IfExpr {
    pub(crate) if_keyword: SyntaxNode<IfKeyword>,
    pub(crate) condition: ParseResult<Expr>,
    pub(crate) block: ParseResult<BlockExpr>,
    pub(crate) else_ifs: Vec<SyntaxNode<ElseIfClause>>,
    pub(crate) else_cluase: Optional<ElseClause>,
}

#[derive(NazmcParse)]
pub(crate) struct ElseIfClause {
    pub(crate) else_keyword: SyntaxNode<ElseKeyword>,
    pub(crate) if_keyword: SyntaxNode<IfKeyword>,
    pub(crate) condition: ParseResult<Expr>,
    pub(crate) block: ParseResult<BlockExpr>,
}

#[derive(NazmcParse)]
pub(crate) struct ElseClause {
    pub(crate) else_keyword: SyntaxNode<ElseKeyword>,
    pub(crate) block: ParseResult<BlockExpr>,
}
#[derive(NazmcParse)]
pub(crate) struct WhenExpr {
    pub(crate) when_keyword: SyntaxNode<WhenKeyword>,
    pub(crate) expr: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct LoopExpr {
    pub(crate) loop_keyword: SyntaxNode<LoopKeyword>,
    pub(crate) block: ParseResult<BlockExpr>,
}

#[derive(NazmcParse)]
pub(crate) struct WhileExpr {
    pub(crate) while_keyword: SyntaxNode<WhileKeyword>,
    pub(crate) block: ParseResult<BlockExpr>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct DoWhileExpr {
    pub(crate) do_keyword: SyntaxNode<DoKeyword>,
    pub(crate) block: ParseResult<BlockExpr>,
    pub(crate) while_keyword: ParseResult<WhileKeyword>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct BlockExpr {
    pub(crate) open_delimiter: SyntaxNode<OpenCurlyBracesSymbol>,
    pub(crate) stms: ZeroOrMany<Stm, CloseCurlyBracesSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct BreakExpr {
    pub(crate) break_keyowrd: SyntaxNode<BreakKeyword>,
    pub(crate) expr: Optional<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct ContinueExpr {
    pub(crate) continue_keyowrd: SyntaxNode<ContinueKeyword>,
}

#[derive(NazmcParse)]
pub(crate) struct ReturnExpr {
    pub(crate) return_keyowrd: SyntaxNode<ReturnKeyword>,
    pub(crate) expr: Optional<Expr>,
}
