use crate::SymbolKind;

use super::*;

#[derive(NazmcParse)]
pub(crate) struct SimplePath {
    pub(crate) top: SyntaxNode<Id>,
    pub(crate) inners: Vec<SyntaxNode<SimpleInnerPath>>,
}

#[derive(NazmcParse)]
pub(crate) struct SimpleInnerPath {
    pub(crate) double_colons: SyntaxNode<DoubleColonsSymbol>,
    pub(crate) inner: ParseResult<Id>,
}
