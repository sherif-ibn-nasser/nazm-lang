use super::*;

#[derive(NazmcParse)]
pub(crate) enum Type {
    Path(SimplePath),
    Ptr(Box<PtrType>),
    Ref(Box<RefType>),
    Paren(Box<ParenType>),
    Slice(Box<SliceType>),
    Tuple(Box<TupleType>),
}

#[derive(NazmcParse)]
pub(crate) struct PtrType {
    pub(crate) star: SyntaxNode<StarSymbol>,
    pub(crate) mut_keyword: Optional<MutKeyword>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse)]
pub(crate) struct RefType {
    pub(crate) hash: SyntaxNode<HashSymbol>,
    pub(crate) mut_keyword: Optional<MutKeyword>,
    pub(crate) typ: ParseResult<Type>,
}

#[derive(NazmcParse)]
pub(crate) struct ParenType {
    pub(crate) open_paren: SyntaxNode<OpenParenthesisSymbol>,
    pub(crate) typ: ParseResult<Type>,
    pub(crate) close_paren: ParseResult<CloseParenthesisSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct SliceType {
    pub(crate) open_bracket: SyntaxNode<OpenSquareBracketSymbol>,
    pub(crate) typ: ParseResult<Type>,
    pub(crate) array_size: Optional<ArraySizeExpr>,
    pub(crate) close_bracket: ParseResult<CloseSquareBracketSymbol>,
}

#[derive(NazmcParse)]
pub(crate) struct ArraySizeExpr {
    pub(crate) semicolon: SyntaxNode<SemicolonSymbol>,
    pub(crate) expr: ParseResult<Expr>,
}
