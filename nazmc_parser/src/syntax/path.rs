use super::*;

#[derive(NazmcParse, Debug)]
pub struct SimplePath {
    pub top: Id,
    pub inners: Vec<SimpleInnerPath>,
}

#[derive(NazmcParse, Debug)]
pub struct SimpleInnerPath {
    pub double_colons: DoubleColonsSymbol,
    pub inner: ParseResult<Id>,
}
