use super::*;

#[derive(NazmcParse, Debug)]
pub enum FileItem {
    WithVisModifier(ItemWithVisibility),
    WithoutModifier(Item),
}

#[derive(NazmcParse, Debug)]
pub struct ItemWithVisibility {
    pub visibility: VisModifier,
    pub item: ParseResult<Item>,
}

#[derive(NazmcParse, Debug)]
pub enum Item {
    Struct(Struct),
    Fn(Fn),
}

#[derive(NazmcParse, Debug)]
pub struct Struct {
    pub struct_keyword: StructKeyword,
    pub name: ParseResult<Id>,
    pub kind: ParseResult<StructKind>,
}

#[derive(NazmcParse, Debug)]
pub enum StructKind {
    Unit(SemicolonSymbol),
    Tuple(TupleStructFields),
    Fields(StructFields),
}

#[derive(NazmcParse, Debug)]
pub struct TupleStructField {
    pub visibility: Option<VisModifier>,
    pub typ: ParseResult<Type>,
}

generatePunctuatedItem!(TupleStructField);

generateDelimitedPunctuated!(
    TupleStructFields,
    OpenParenthesisSymbol,
    TupleStructField,
    CloseParenthesisSymbol
);

#[derive(NazmcParse, Debug)]
pub struct StructField {
    pub visibility: Option<VisModifier>,
    pub name: Id,
    pub typ: ParseResult<ColonWithType>,
}

generatePunctuatedItem!(StructField);

generateDelimitedPunctuated!(
    StructFields,
    OpenCurlyBraceSymbol,
    StructField,
    CloseCurlyBraceSymbol
);

#[derive(NazmcParse, Debug)]
pub struct Fn {
    pub fn_keyword: FnKeyword,
    pub name: ParseResult<Id>,
    pub params_decl: ParseResult<FnParams>,
    pub return_type: Option<ColonWithType>,
    /// This must be checked that it doesn't have a lambda arrow
    pub body: ParseResult<LambdaExpr>,
}

#[derive(NazmcParse, Debug)]
pub struct FnParam {
    pub name: Id,
    pub typ: ParseResult<ColonWithType>,
}

generatePunctuatedItem!(FnParam);

generateDelimitedPunctuated!(
    FnParams,
    OpenParenthesisSymbol,
    FnParam,
    CloseParenthesisSymbol
);
