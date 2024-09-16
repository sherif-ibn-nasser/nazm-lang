use super::*;

#[derive(NazmcParse, Debug)]
pub(crate) enum FileItem {
    WithVisModifier(ItemWithVisibility),
    WithoutModifier(Item),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct ItemWithVisibility {
    pub(crate) visibility: VisModifier,
    pub(crate) item: ParseResult<Item>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum Item {
    Struct(Struct),
    Fn(Fn),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct Struct {
    pub(crate) struct_keyword: StructKeyword,
    pub(crate) name: ParseResult<Id>,
    pub(crate) kind: ParseResult<StructKind>,
}

#[derive(NazmcParse, Debug)]
pub(crate) enum StructKind {
    Unit(SemicolonSymbol),
    Tuple(TupleType),
    Fields(StructFields),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct StructField {
    pub(crate) visibility: Option<VisModifier>,
    pub(crate) name: Id,
    pub(crate) typ: ParseResult<ColonWithType>,
}

generatePunctuatedItem!(StructField);

generateDelimitedPunctuated!(
    StructFields,
    OpenCurlyBraceSymbol,
    StructField,
    CloseCurlyBraceSymbol
);

#[derive(NazmcParse, Debug)]
pub(crate) struct Fn {
    pub(crate) fn_keyword: FnKeyword,
    pub(crate) name: ParseResult<Id>,
    pub(crate) params_decl: ParseResult<FnParams>,
    pub(crate) return_type: Option<ColonWithType>,
    /// This must be checked that it doesn't have a lambda arrow
    pub(crate) body: ParseResult<LambdaExpr>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct FnParam {
    pub(crate) name: Id,
    pub(crate) typ: ParseResult<ColonWithType>,
}

generatePunctuatedItem!(FnParam);

generateDelimitedPunctuated!(
    FnParams,
    OpenParenthesisSymbol,
    FnParam,
    CloseParenthesisSymbol
);
