use expr::Expr;
use syntax::*;

use crate::parser::*;

macro_rules! generateDelimitedPunctuated {
    ($name:ident, $open_delim:ident, $item:ident, $close_delim:ident ) => {
        paste! {
            pub(crate) struct $name {
                pub(crate) open_delim: SyntaxNode<$open_delim>,
                pub(crate) items: Optional<[<Punctuated $item>]>,
                pub(crate) close_delim: ParseResult<$close_delim>,
            }

            #[derive(NazmcParse)]
            pub(crate) struct [<$open_delim With $item With $close_delim>] {
                pub(crate) open_delim: SyntaxNode<$open_delim>,
                pub(crate) parsed_item_with_close_delim: ParseResult<[<Parsed $item With $close_delim>]>,
            }

            #[derive(NazmcParse)]
            pub(crate) enum [<Parsed $item With $close_delim>] {
                NoItems($close_delim),
                WithItems(Box<[<$item sWith $close_delim>]>),
            }

            #[derive(NazmcParse)]
            pub(crate) struct [<$item sWith $close_delim>] {
                pub(crate) first_item: ParseResult<$item>,
                pub(crate) items_with_terminator: ZeroOrMany<[<CommaWith $item>], [<CommaWith $close_delim>]>,
            }

            impl NazmcParse for ParseResult<$name> {
                fn parse(iter: &mut TokensIter) -> Self {
                    let parse_result = ParseResult::<[<$open_delim With $item With $close_delim>]>::parse(iter);

                    let decl_impl_node = match parse_result {
                        ParseResult::Parsed(decl_impl) => decl_impl,
                        ParseResult::Unexpected {
                            span,
                            found,
                            is_start_failure,
                        } => {
                            return ParseResult::Unexpected {
                                span,
                                found,
                                is_start_failure,
                            }
                        }
                    };

                    let is_broken = decl_impl_node.is_broken;
                    let span = decl_impl_node.span;
                    let open_delim = decl_impl_node.tree.open_delim;

                    // The unexpected case is unreachable as it will be include in WithParams case, so we can safely unwrap it
                    let close = decl_impl_node.tree.parsed_item_with_close_delim.unwrap();

                    let close_decl_withitems = match close.tree {
                        [<Parsed $item With $close_delim>]::NoItems(close_delim) => {
                            return ParseResult::Parsed(SyntaxNode {
                                span,
                                is_broken,
                                tree: $name {
                                    open_delim,
                                    items: Optional::None,
                                    close_delim: ParseResult::Parsed(SyntaxNode {
                                        span: close.span,
                                        is_broken: close.is_broken,
                                        tree: close_delim,
                                    }),
                                },
                            })
                        }
                        [<Parsed $item With $close_delim>]::WithItems(close_decl_with_iteems) => close_decl_with_iteems,
                    };

                    let first_item = close_decl_withitems.first_item;
                    let _restitems = close_decl_withitems.items_with_terminator.items;
                    let (_trailing_comma, close_delim) = match close_decl_withitems.items_with_terminator.terminator {
                        ParseResult::Parsed(node) => (
                            node.tree._comma,
                            ParseResult::Parsed(node.tree.close_delim),
                        ),
                        ParseResult::Unexpected {
                            span,
                            found,
                            is_start_failure,
                        } => (
                            Optional::None,
                            ParseResult::Unexpected {
                                span,
                                found,
                                is_start_failure,
                            },
                        ),
                    };

                    let mut items_span = first_item.span().unwrap();

                    if let Optional::Some(comma_node) = &_trailing_comma {
                        items_span = items_span.merged_with(&comma_node.span)
                    } else if let Option::Some(last_param) = _restitems.last() {
                        items_span = items_span.merged_with(&last_param.span().unwrap())
                    }

                    let items = SyntaxNode {
                        span: items_span,
                        is_broken: !first_item.is_parsed_and_valid()
                            || _restitems.iter().any(|p| !p.is_parsed_and_valid())
                            || !_trailing_comma.is_parsed_and_valid(),
                        tree: [<Punctuated $item>] {
                            first_item,
                            _restitems,
                            _trailing_comma,
                        },
                    };

                    ParseResult::Parsed(SyntaxNode {
                        span,
                        is_broken,
                        tree: $name {
                            open_delim,
                            items: Optional::Some(items),
                            close_delim,
                        },
                    })
                }
            }

        }
    };
}

macro_rules! generatePunctuatedItem {
    ($item:ident) => {
        paste! {

            pub(crate) struct [<Punctuated $item>] {
                pub(crate) first_item: ParseResult<$item>,
                pub(crate) _restitems: Vec<ParseResult<[<CommaWith $item>]>>,
                pub(crate) _trailing_comma: Optional<CommaSymbol>,
            }

            impl NazmcParse for ParseResult<[<Punctuated $item>]> {
                fn parse(_iter: &mut TokensIter) -> Self {
                    unreachable!() // Just  added to usee it as Optional
                }
            }

            #[derive(NazmcParse)]
            pub(crate) struct [<CommaWith $item>] {
                _comma: SyntaxNode<CommaSymbol>,
                _item: SyntaxNode<$item>,
            }
        }
    };
}

macro_rules! generateTrailingCommaWithCloseDelimiter {
    ($close_delim: ident) => {
        paste! {
            #[derive(NazmcParse)]
            pub(crate) struct [<CommaWith $close_delim>] {
                _comma: Optional<CommaSymbol>,
                close_delim: SyntaxNode<$close_delim>,
            }
        }
    };
}

pub(crate) use generateDelimitedPunctuated;
pub(crate) use generatePunctuatedItem;
pub(crate) use generateTrailingCommaWithCloseDelimiter;
