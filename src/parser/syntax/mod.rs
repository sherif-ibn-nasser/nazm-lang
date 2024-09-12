use super::*;
use paste::paste;

pub(crate) mod punctuated;
pub(crate) use punctuated::*;

pub(crate) mod terminal;
pub(crate) use terminal::*;

pub(crate) mod path;
pub(crate) use path::*;

pub(crate) mod typ;
pub(crate) use typ::*;

pub(crate) mod stm;
pub(crate) use stm::*;

pub(crate) mod expr;
pub(crate) use expr::*;

#[derive(NazmcParse, Debug)]
pub(crate) struct ColonWithType {
    pub(crate) colon: ColonSymbol,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum VisModifier {
    Public(PublicKeyword),
    Private(PrivateKeyword),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct FnParam {
    pub(crate) name: Id,
    pub(crate) typ: ParseResult<ColonWithType>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct StructField {
    pub(crate) visibility: Option<VisModifier>,
    pub(crate) name: Id,
    pub(crate) colon: ParseResult<ColonSymbol>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct StructFieldInitExpr {
    pub(crate) name: Id,
    pub(crate) expr: Option<StructFieldInitExplicitExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct StructFieldInitExplicitExpr {
    pub(crate) equal: EqualSymbol,
    pub(crate) expr: ParseResult<Expr>,
}

generateTrailingCommaWithCloseDelimiter!(CloseParenthesisSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseAngleBracketOrGreaterSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseCurlyBraceSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseSquareBracketSymbol);

generateTrailingCommaWithCloseDelimiter!(RArrowSymbol);

generatePunctuatedItem!(Type);

generatePunctuatedItem!(StructField);

generatePunctuatedItem!(FnParam);

generatePunctuatedItem!(Expr);

generatePunctuatedItem!(StructFieldInitExpr);

generatePunctuatedItem!(BindingDecl);

generatePunctuatedItem!(BindingDeclKind);

generateDelimitedPunctuated!(
    StructFieldsDecl,
    OpenCurlyBraceSymbol,
    StructField,
    CloseCurlyBraceSymbol
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
    OpenCurlyBraceSymbol,
    StructFieldInitExpr,
    CloseCurlyBraceSymbol
);

generateDelimitedPunctuated!(
    DestructedTuple,
    OpenParenthesisSymbol,
    BindingDeclKind,
    CloseParenthesisSymbol
);

#[derive(NazmcParse, Debug)]
pub(crate) enum BindingDeclKind {
    Id(Id),
    Destructed(Box<DestructedTuple>), // Box for the large size
}
