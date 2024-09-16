use syntax::*;

use crate::parser::*;

macro_rules! generateDelimitedPunctuated {
    ($name:ident, $open_delim:ident, $item:ident, $close_delim:ident ) => {
        paste! {

            #[derive(SpannedAndCheck, Debug)]
            pub(crate) struct $name {
                pub(crate) open_delim: $open_delim,
                pub(crate) items: Option<[<Punctuated $item>]>,
                pub(crate) close_delim: ParseResult<$close_delim>,
            }

            #[derive(NazmcParse, Debug)]
            pub(crate) struct [<$open_delim With $item With $close_delim>] {
                pub(crate) open_delim: $open_delim,
                pub(crate) parsed_item_with_close_delim: ParseResult<[<Parsed $item With $close_delim>]>,
            }

            #[derive(NazmcParse, Debug)]
            pub(crate) enum [<Parsed $item With $close_delim>] {
                NoItems($close_delim),
                WithItems(Box<[<$item sWith $close_delim>]>),
            }

            #[derive(NazmcParse, Debug)]
            pub(crate) struct [<$item sWith $close_delim>] {
                pub(crate) first_item: ParseResult<$item>,
                pub(crate) items_with_terminator: ZeroOrMany<[<CommaWith $item>], [<CommaWith $close_delim>]>,
            }

            impl NazmcParse for ParseResult<$name> {
                fn parse(iter: &mut TokensIter) -> Self {
                    let decl_impl_node = ParseResult::<[<$open_delim With $item With $close_delim>]>::parse(iter)?;

                    let open_delim = decl_impl_node.open_delim;

                    // The unexpected case is unreachable as it will be include in WithParams case, so we can safely unwrap it
                    let close = decl_impl_node.parsed_item_with_close_delim.unwrap();

                    let close_decl_with_items = match close {
                        [<Parsed $item With $close_delim>]::NoItems(close_delim) => {
                            return Ok($name {
                                open_delim,
                                items: None,
                                close_delim: Ok(close_delim),
                            })
                        }
                        [<Parsed $item With $close_delim>]::WithItems(close_decl_with_iteems) => close_decl_with_iteems,
                    };

                    let first_item = close_decl_with_items.first_item;

                    let rest_items = close_decl_with_items.items_with_terminator.items;

                    let (trailing_comma, close_delim) = match close_decl_with_items.items_with_terminator.terminator {
                        Ok(node) => (node.comma, Ok(node.close_delim)),
                        Err(err) => (None, Err(err)),
                    };

                    let items = [<Punctuated $item>] {
                        first_item,
                        rest_items,
                        trailing_comma,
                    };

                    Ok($name {
                        open_delim,
                        items: Some(items),
                        close_delim,
                    })
                }
            }

        }
    };
}

macro_rules! generatePunctuatedItem {
    ($item:ident) => {
        paste! {

            #[derive(SpannedAndCheck, Debug)]
            pub(crate) struct [<Punctuated $item>] {
                pub(crate) first_item: ParseResult<$item>,
                pub(crate) rest_items: Vec<ParseResult<[<CommaWith $item>]>>,
                pub(crate) trailing_comma: Option<CommaSymbol>,
            }

            impl NazmcParse for ParseResult<[<Punctuated $item>]> {
                fn parse(_iter: &mut TokensIter) -> Self {
                    unreachable!() // Just  added to usee it as Optional
                }
            }

            #[derive(NazmcParse, Debug)]
            pub(crate) struct [<CommaWith $item>] {
                pub(crate) comma: CommaSymbol,
                pub(crate) item: $item,
            }
        }
    };
}

macro_rules! generateTrailingCommaWithCloseDelimiter {
    ($close_delim: ident) => {
        paste! {
            #[derive(NazmcParse, Debug)]
            pub(crate) struct [<CommaWith $close_delim>] {
                comma: Option<CommaSymbol>,
                close_delim: $close_delim,
            }
        }
    };
}

pub(crate) use generateDelimitedPunctuated;
pub(crate) use generatePunctuatedItem;
pub(crate) use generateTrailingCommaWithCloseDelimiter;
