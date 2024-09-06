mod terminal;

mod path;

mod expr;

mod typ;

mod punctuated;

use super::*;

use paste::paste;

pub(crate) use terminal::*;

use path::*;

use punctuated::*;

use expr::*;

use typ::*;

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
pub(crate) enum VisModifier {
    Public(PublicKeyword),
    Private(PrivateKeyword),
}

#[derive(NazmcParse)]
pub(crate) struct FnParam {
    pub(crate) name: SyntaxNode<Id>,
    pub(crate) colon: ParseResult<ColonSymbol>,
    pub(crate) typ: ParseResult<Id>, // TODO: Change to TypeExpr
}
