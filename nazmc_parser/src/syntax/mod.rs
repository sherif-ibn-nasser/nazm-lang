use super::*;
use paste::paste;

pub mod punctuated;
pub use punctuated::*;

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
    pub content: ZeroOrMany<FileItem, Eof>,
}
