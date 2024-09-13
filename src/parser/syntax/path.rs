use super::*;

#[derive(NazmcParse, Debug)]
pub(crate) struct SimplePath {
    pub(crate) top: Id,
    pub(crate) inners: Vec<SimpleInnerPath>,
}

#[derive(NazmcParse, Debug)]
pub(crate) struct SimpleInnerPath {
    pub(crate) double_colons: DoubleColonsSymbol,
    pub(crate) inner: ParseResult<Id>,
}
