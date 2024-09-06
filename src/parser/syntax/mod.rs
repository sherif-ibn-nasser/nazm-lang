mod terminal;

mod expr;

mod punctuated;

pub(crate) use terminal::*;

use super::*;

use paste::paste;

use punctuated::*;

use expr::*;

generateTrailingCommaWithCloseDelimiter!(CloseParenthesisSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseAngleBracketOrGreaterSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseCurlyBracesSymbol);

generatePunctuatedItem!(FnParam);

generatePunctuatedItem!(Expr);

// Could be used for tuples, function calls and and nodrma paren expressions
generateDelimitedPunctuated!(
    ParenExpr,
    OpenParenthesisSymbol,
    Expr,
    CloseParenthesisSymbol
);

generateDelimitedPunctuated!(
    FnParamsDecl,
    OpenParenthesisSymbol,
    FnParam,
    CloseParenthesisSymbol
);

#[derive(NazmcParse)]
pub(crate) struct FnParam {
    pub(crate) name: SyntaxNode<Id>,
    pub(crate) colon: ParseResult<ColonSymbol>,
    pub(crate) typ: ParseResult<Id>, // TODO: Change to TypeExpr
}
