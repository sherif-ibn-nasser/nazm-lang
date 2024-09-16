use super::*;

#[derive(NazmcParse, Debug)]
pub(crate) enum Type {
    Path(SimplePath),
    Ptr(Box<PtrType>),
    Ref(Box<RefType>),
    Slice(Box<SliceType>),
    Tuple(Box<TupleType>),
}

#[derive(NazmcParse, Debug)]
pub(crate) struct PtrType {
    pub(crate) star: StarSymbol,
    pub(crate) mut_keyword: Option<MutKeyword>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct RefType {
    pub(crate) hash: HashSymbol,
    pub(crate) mut_keyword: Option<MutKeyword>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct SliceType {
    pub(crate) open_bracket: OpenSquareBracketSymbol,
    pub(crate) typ: ParseResult<Type>,
    pub(crate) array_size: Option<ArraySizeExpr>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct ArraySizeExpr {
    pub(crate) semicolon: SemicolonSymbol,
    pub(crate) expr: ParseResult<Expr>,
}

generatePunctuatedItem!(Type);

generateDelimitedPunctuated!(
    TupleType,
    OpenParenthesisSymbol,
    Type,
    CloseParenthesisSymbol
);
