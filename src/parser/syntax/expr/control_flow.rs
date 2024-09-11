use super::*;

#[derive(NazmcParse)]
pub(crate) struct IfExpr {
    pub(crate) if_keyword: SyntaxNode<IfKeyword>,
    pub(crate) condition: ParseResult<Expr>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
    pub(crate) else_ifs: Vec<SyntaxNode<ElseIfClause>>,
    pub(crate) else_cluase: Optional<ElseClause>,
}

#[derive(NazmcParse)]
pub(crate) struct ElseIfClause {
    pub(crate) else_keyword: SyntaxNode<ElseKeyword>,
    pub(crate) if_keyword: SyntaxNode<IfKeyword>,
    pub(crate) condition: ParseResult<Expr>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}

#[derive(NazmcParse)]
pub(crate) struct ElseClause {
    pub(crate) else_keyword: SyntaxNode<ElseKeyword>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}
#[derive(NazmcParse)]
pub(crate) struct WhenExpr {
    pub(crate) when_keyword: SyntaxNode<WhenKeyword>,
    pub(crate) expr: ParseResult<Expr>,
    // TODO
}

#[derive(NazmcParse)]
pub(crate) struct LoopExpr {
    pub(crate) loop_keyword: SyntaxNode<LoopKeyword>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}

#[derive(NazmcParse)]
pub(crate) struct WhileExpr {
    pub(crate) while_keyword: SyntaxNode<WhileKeyword>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct DoWhileExpr {
    pub(crate) do_keyword: SyntaxNode<DoKeyword>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
    pub(crate) while_keyword: ParseResult<WhileKeyword>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse)]
pub(crate) struct RunExpr {
    pub(crate) run: SyntaxNode<RunKeyword>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
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

struct ConditionalBlock {
    pub(crate) condition: SyntaxNode<Expr>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}

impl NazmcParse for ParseResult<ConditionalBlock> {
    fn parse(iter: &mut TokensIter) -> Self {
        let peek_idx = iter.peek_idx;
        let condition = match ParseResult::<Expr>::parse(iter) {
            ParseResult::Parsed(node) => node,
            ParseResult::Unexpected { span, found, .. } => {
                iter.peek_idx = peek_idx;
                return ParseResult::Unexpected {
                    span,
                    found,
                    is_start_failure: true,
                };
            }
        };

        let last_unary_ex = match condition.tree.bin.last() {
            Some(SyntaxNode {
                tree:
                    BinExpr {
                        right: ParseResult::Parsed(node),
                        ..
                    },
                ..
            }) => node,
            _ => &condition.tree.left,
        };

        //let last_lambda_expr =
        match last_unary_ex.tree.post_ops.last() {
            Some(SyntaxNode {
                tree:
                    PostOpExpr::Lambda(LambdaExpr {
                        lambda_arrow: Optional::None, // No '->' in the lambda expression
                        ..
                    }),
                ..
            }) => todo!(), // Block is found
            Some(_) => todo!(), // Block expected after the condition
            None => match &last_unary_ex.tree.expr {
                SyntaxNode {
                    tree:
                        AtomicExpr::Lambda(LambdaExpr {
                            lambda_arrow: Optional::None, // No '->' in the lambda expression
                            ..
                        }),
                    ..
                } if last_unary_ex.tree.ops.is_empty() => todo!(), // Condition expected before the block
                _ => todo!(), // Block expected after the condition
            },
        };

        todo!()
    }
}
