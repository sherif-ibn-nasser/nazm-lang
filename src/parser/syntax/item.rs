use super::*;

#[derive(NazmcParse, Debug)]
pub(crate) struct Item {
    pub(crate) visibility: Option<VisModifier>,
    pub(crate) kind: ItemKind,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum ItemKind {
    Struct(Struct),
    Fn(Fn),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct Struct {
    pub(crate) struct_keyword: StructKeyword,
    pub(crate) name: ParseResult<Id>,
    pub(crate) fields_decl: ParseResult<StructFieldsDecl>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct StructField {
    pub(crate) visibility: Option<VisModifier>,
    pub(crate) name: Id,
    pub(crate) colon: ParseResult<ColonSymbol>,
    pub(crate) typ: ParseResult<Type>,
}

generatePunctuatedItem!(StructField);

generateDelimitedPunctuated!(
    StructFieldsDecl,
    OpenCurlyBraceSymbol,
    StructField,
    CloseCurlyBraceSymbol
);

#[derive(NazmcParse, Debug)]
pub(crate) struct Fn {
    pub(crate) fn_keyword: FnKeyword,
    pub(crate) name: ParseResult<Id>,
    pub(crate) params_decl: ParseResult<FnParamsDecl>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct FnParam {
    pub(crate) name: Id,
    pub(crate) typ: ParseResult<ColonWithType>,
}

generatePunctuatedItem!(FnParam);

generateDelimitedPunctuated!(
    FnParamsDecl,
    OpenParenthesisSymbol,
    FnParam,
    CloseParenthesisSymbol
);
