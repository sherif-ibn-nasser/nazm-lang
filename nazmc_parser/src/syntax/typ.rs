use super::*;

#[derive(NazmcParse, Debug)]
pub enum Type {
    Path(SimplePath),
    Ptr(Box<PtrType>),
    Ref(Box<RefType>),
    Slice(Box<SliceType>),
    Paren(Box<ParenType>),
}

#[derive(NazmcParse, Debug)]
pub struct PtrType {
    pub star: StarSymbol,
    pub mut_keyword: Option<MutKeyword>,
    pub typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub struct RefType {
    pub hash: HashSymbol,
    pub mut_keyword: Option<MutKeyword>,
    pub typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub struct SliceType {
    pub open_bracket: OpenSquareBracketSymbol,
    pub typ: ParseResult<Type>,
    pub array_size: Option<ArraySizeExpr>,
    pub close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse, Debug)]
pub struct ArraySizeExpr {
    pub semicolon: SemicolonSymbol,
    pub expr: ParseResult<Expr>,
}

#[derive(NazmcParse, Debug)]
pub struct ParenType {
    pub tuple: TupleType,
    pub lambda: Option<LambdaType>,
}

#[derive(NazmcParse, Debug)]
pub struct LambdaType {
    pub r_arrow: RArrowSymbol,
    pub typ: ParseResult<Type>,
}

generatePunctuatedItem!(Type);

generateDelimitedPunctuated!(
    TupleType,
    OpenParenthesisSymbol,
    Type,
    CloseParenthesisSymbol
);
