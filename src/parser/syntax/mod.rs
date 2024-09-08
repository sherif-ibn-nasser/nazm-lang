mod terminal;

mod op;

mod path;

mod expr;

mod typ;

mod punctuated;

use super::*;

use paste::paste;

pub(crate) use terminal::*;

use op::*;

use path::*;

use punctuated::*;

use expr::*;

use typ::*;

#[derive(NazmcParse)]
pub(crate) enum VisModifier {
    Public(PublicKeyword),
    Private(PrivateKeyword),
}

#[derive(NazmcParse)]
pub(crate) struct FnParam {
    pub(crate) name: SyntaxNode<Id>,
    pub(crate) colon: ParseResult<ColonSymbol>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse)]
pub(crate) struct StructField {
    pub(crate) visibility: Optional<VisModifier>,
    pub(crate) name: SyntaxNode<Id>,
    pub(crate) colon: ParseResult<ColonSymbol>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse)]
pub(crate) struct StructFieldInitExpr {
    pub(crate) name: SyntaxNode<Id>,
    pub(crate) expr: Optional<StructFieldInitExplicitExpr>,
}

#[derive(NazmcParse)]
pub(crate) struct StructFieldInitExplicitExpr {
    pub(crate) colon: SyntaxNode<ColonSymbol>,
    pub(crate) expr: ParseResult<Expr>,
}

generateTrailingCommaWithCloseDelimiter!(CloseParenthesisSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseAngleBracketOrGreaterSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseCurlyBracesSymbol);

generatePunctuatedItem!(Type);

generatePunctuatedItem!(StructField);

generatePunctuatedItem!(FnParam);

generatePunctuatedItem!(Expr);

generatePunctuatedItem!(StructFieldInitExpr);

generateDelimitedPunctuated!(
    StructFieldsDecl,
    OpenCurlyBracesSymbol,
    StructField,
    CloseCurlyBracesSymbol
);

generateDelimitedPunctuated!(
    TupleType,
    OpenParenthesisSymbol,
    Type,
    CloseParenthesisSymbol
);

generateDelimitedPunctuated!(
    FnParamsDecl,
    OpenParenthesisSymbol,
    FnParam,
    CloseParenthesisSymbol
);

// Could be used for tuples, function calls and and nodrma paren expressions
generateDelimitedPunctuated!(
    ParenExpr,
    OpenParenthesisSymbol,
    Expr,
    CloseParenthesisSymbol
);

generateDelimitedPunctuated!(
    StructFieldsInitExpr,
    OpenCurlyBracesSymbol,
    StructFieldInitExpr,
    CloseCurlyBracesSymbol
);
