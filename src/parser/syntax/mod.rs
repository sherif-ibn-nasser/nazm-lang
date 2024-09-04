mod terminal;

mod expr;

mod punctuated;

pub(crate) use terminal::*;

use super::*;

use paste::paste;

use punctuated::*;

use expr::*;

generateTrailingCommaWithCloseDelimiter!(CloseParenthesisSymbol);

generateTrailingCommaWithCloseDelimiter!(BitOrSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseAngleBracketOrGreaterSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseCurlyBracesSymbol);

generatePunctuatedItem!(FnParam);

generatePunctuatedItem!(Expr);

generateDelimitedPunctuated!(TupleExpr, BitOrSymbol, Expr, BitOrSymbol);

generateDelimitedPunctuated!(
    FnCallExpr,
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
