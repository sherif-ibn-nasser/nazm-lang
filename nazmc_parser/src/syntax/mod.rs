use super::*;
use paste::paste;

pub mod punctuated;
use punctuated::*;

pub mod terminal;
pub use terminal::*;

pub mod item;
pub use item::*;

pub mod path;
pub use path::*;

pub mod typ;
pub use typ::*;

pub mod stm;
pub use stm::*;

pub mod expr;
pub use expr::*;

generateTrailingCommaWithCloseDelimiter!(CloseParenthesisSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseAngleBracketOrGreaterSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseCurlyBraceSymbol);

generateTrailingCommaWithCloseDelimiter!(CloseSquareBracketSymbol);

generateTrailingCommaWithCloseDelimiter!(RArrowSymbol);

#[derive(NazmcParse, Debug)]
pub struct File {
    pub imports: Vec<ImportStm>,
    pub content: ZeroOrMany<FileItem, Eof>,
}

#[derive(NazmcParse, Debug)]
pub struct ImportStm {
    pub import_keyword: ImportKeyword,
    pub top: ParseResult<Id>,
    pub sec: ParseResult<DoubleColonsWithPathSegInImportStm>,
    pub segs: Vec<DoubleColonsWithPathSegInImportStm>,
    pub semicolon: ParseResult<SemicolonSymbol>,
}

#[derive(NazmcParse, Debug)]
pub struct DoubleColonsWithPathSegInImportStm {
    pub double_colons: DoubleColonsSymbol,
    pub seg: ParseResult<PathSegInImportStm>,
}

#[derive(NazmcParse, Debug)]
pub enum PathSegInImportStm {
    Id(Id),
    Star(StarSymbol),
}
