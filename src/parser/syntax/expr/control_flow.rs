use super::*;

#[derive(NazmcParse, Debug)]
pub(crate) struct IfExpr {
    pub(crate) if_keyword: IfKeyword,
    pub(crate) conditional_block: ParseResult<ConditionalBlock>,
    pub(crate) else_ifs: Vec<ElseIfClause>,
    pub(crate) else_cluase: Option<ElseClause>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct ElseIfClause {
    pub(crate) else_keyword: ElseKeyword,
    pub(crate) if_keyword: IfKeyword,
    pub(crate) conditional_block: ParseResult<ConditionalBlock>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct ElseClause {
    pub(crate) else_keyword: ElseKeyword,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct WhenExpr {
    pub(crate) when_keyword: WhenKeyword,
    pub(crate) expr: ParseResult<Expr>,
    // TODO
}

#[derive(NazmcParse, Debug)]
pub(crate) struct LoopExpr {
    pub(crate) loop_keyword: LoopKeyword,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct WhileExpr {
    pub(crate) while_keyword: WhileKeyword,
    pub(crate) conditional_block: ParseResult<ConditionalBlock>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct DoWhileExpr {
    pub(crate) do_keyword: DoKeyword,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
    pub(crate) while_keyword: ParseResult<WhileKeyword>,
    pub(crate) condition: ParseResult<Expr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct RunExpr {
    pub(crate) run: RunKeyword,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct BreakExpr {
    pub(crate) break_keyowrd: BreakKeyword,
    pub(crate) expr: Option<Expr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct ContinueExpr {
    pub(crate) continue_keyowrd: ContinueKeyword,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct ReturnExpr {
    pub(crate) return_keyowrd: ReturnKeyword,
    pub(crate) expr: Option<Expr>,
}

#[derive(SpannedAndCheck, Debug)]
pub(crate) struct ConditionalBlock {
    pub(crate) condition: ParseResult<Expr>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) block: ParseResult<LambdaExpr>,
}

impl NazmcParse for ParseResult<ConditionalBlock> {
    fn parse(iter: &mut TokensIter) -> Self {
        let mut condition = ParseResult::<Expr>::parse(iter)?;

        let len = condition.bin.len();

        let last_primary_ex = if len == 0 {
            &mut condition.left
        } else {
            match &mut condition.bin[len - 1] {
                BinExpr {
                    right: Ok(ref mut node),
                    ..
                } => node,
                BinExpr {
                    right: Err(err), ..
                } => {
                    return Ok(ConditionalBlock {
                        block: Err(err.clone()), // No expressions found after the bin op (so no lambda block is found after the op) so clone the error
                        condition: Ok(condition),
                    });
                }
            }
        };

        let len = last_primary_ex.inner_access.len();

        let last_post_ops = if len == 0 {
            &mut last_primary_ex.post_ops
        } else {
            &mut last_primary_ex.inner_access[len - 1].post_ops
        };

        match last_post_ops.last() {
            Some(PostOpExpr::Lambda(LambdaExpr {
                lambda_arrow: Option::None, // No '->' in the lambda expression
                ..
            })) => {} // Block is found
            _ => {
                // No block is found (or found a lambda block with '->')
                let parse_err = match iter.recent() {
                    Some(token) => Err(ParseErr {
                        span: token.span,
                        found_token: token.kind.clone(),
                    }),
                    None => ParseErr::eof(iter.peek_start_span()),
                };
                return Ok(ConditionalBlock {
                    condition: Ok(condition),
                    block: parse_err,
                });
            }
        };

        let Some(PostOpExpr::Lambda(lambda)) = last_post_ops.pop() else {
            unreachable!()
        };

        Ok(ConditionalBlock {
            condition: Ok(condition),
            block: Ok(lambda),
        })
    }
}
