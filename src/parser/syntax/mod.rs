use super::*;
use paste::paste;

pub(crate) mod terminal;
pub(crate) use terminal::*;

// mod path;

// mod expr;

// mod typ;

// mod punctuated;

// mod symbols;

// mod stm;

// use path::*;

// use punctuated::*;

// use expr::*;

// use typ::*;

// use stm::*;

// use symbols::*;

// #[derive(NazmcParse)]
// pub(crate) struct ColonWithType {
//     pub(crate) colon: SyntaxNode<ColonSymbol>,
//     pub(crate) typ: ParseResult<Type>,
// }

// #[derive(NazmcParse)]
// pub(crate) enum VisModifier {
//     Public(PublicKeyword),
//     Private(PrivateKeyword),
// }

// #[derive(NazmcParse)]
// pub(crate) struct FnParam {
//     pub(crate) name: SyntaxNode<Id>,
//     pub(crate) typ: ParseResult<ColonWithType>,
// }

// #[derive(NazmcParse)]
// pub(crate) struct StructField {
//     pub(crate) visibility: Optional<VisModifier>,
//     pub(crate) name: SyntaxNode<Id>,
//     pub(crate) colon: ParseResult<ColonSymbol>,
//     pub(crate) typ: ParseResult<Type>,
// }

// #[derive(NazmcParse)]
// pub(crate) struct StructFieldInitExpr {
//     pub(crate) name: SyntaxNode<Id>,
//     pub(crate) expr: Optional<StructFieldInitExplicitExpr>,
// }

// #[derive(NazmcParse)]
// pub(crate) struct StructFieldInitExplicitExpr {
//     pub(crate) equal: SyntaxNode<EqualSymbol>,
//     pub(crate) expr: ParseResult<Expr>,
// }

// generateTrailingCommaWithCloseDelimiter!(CloseParenthesisSymbol);

// generateTrailingCommaWithCloseDelimiter!(CloseAngleBracketOrGreaterSymbol);

// generateTrailingCommaWithCloseDelimiter!(CloseCurlyBraceSymbol);

// generateTrailingCommaWithCloseDelimiter!(CloseSquareBracketSymbol);

// generateTrailingCommaWithCloseDelimiter!(RArrow);

// generatePunctuatedItem!(Type);

// generatePunctuatedItem!(StructField);

// generatePunctuatedItem!(FnParam);

// generatePunctuatedItem!(Expr);

// generatePunctuatedItem!(StructFieldInitExpr);

// generatePunctuatedItem!(BindingDecl);

// generatePunctuatedItem!(BindingDeclKind);

// generateDelimitedPunctuated!(
//     StructFieldsDecl,
//     OpenCurlyBraceSymbol,
//     StructField,
//     CloseCurlyBraceSymbol
// );

// generateDelimitedPunctuated!(
//     TupleType,
//     OpenParenthesisSymbol,
//     Type,
//     CloseParenthesisSymbol
// );

// generateDelimitedPunctuated!(
//     FnParamsDecl,
//     OpenParenthesisSymbol,
//     FnParam,
//     CloseParenthesisSymbol
// );

// // Could be used for tuples, function calls and and nodrma paren expressions
// generateDelimitedPunctuated!(
//     ParenExpr,
//     OpenParenthesisSymbol,
//     Expr,
//     CloseParenthesisSymbol
// );

// generateDelimitedPunctuated!(
//     StructFieldsInitExpr,
//     OpenCurlyBraceSymbol,
//     StructFieldInitExpr,
//     CloseCurlyBraceSymbol
// );

// generateDelimitedPunctuated!(
//     DestructedTuple,
//     OpenParenthesisSymbol,
//     BindingDeclKind,
//     CloseParenthesisSymbol
// );

// #[derive(NazmcParse)]
// pub(crate) enum BindingDeclKind {
//     Id(Id),
//     Destructed(Box<DestructedTuple>), // Box for the large size
// }
