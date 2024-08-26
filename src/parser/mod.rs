/// This module defines the core components and traits required for parsing an Abstract Syntax Tree (AST)
/// in the Nazmc language parser. It provides the foundational structures and parsing logic for different
/// AST node types, ensuring that the syntax is correctly interpreted and processed.
use nazmc_diagnostics::span::Span;
use nazmc_parse_derive::NazmcParse;
use tokens_iter::TokensIter;

use crate::TokenType;

pub(crate) mod ast;

pub(crate) mod tokens_iter;

/// The `NazmcParse` trait must be implemented by all AST nodes. It defines a `parse` method that takes
/// a mutable reference to a `TokensIter` and returns an instance of the implementing type.
pub(crate) trait NazmcParse
where
    Self: Sized,
{
    fn parse(iter: &mut TokensIter) -> Self;
}

/// Represents an AST node that wraps around a successful parse result. It includes the `Span`
/// information for the node and the parsed `tree` itself. This structure also manages error
/// recovery by resetting tokens if parsing fails.
pub(crate) struct ASTNode<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    span: Span,
    tree: Tree,
}

/// The default result of a parsing attempt. `ParseResult` can either be `Parsed`, indicating
/// successful parsing, or `Unexpected`, indicating an unexpected token was encountered. This
/// enum is fundamental in error reporting and control flow within the parsing process.
pub(crate) enum ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    Parsed(ASTNode<Tree>),
    Unexpected { span: Span, found: TokenType },
}

/// `Optional` represents an optional AST node. It either contains a parsed node (`Some`) or nothing (`None`).
pub(crate) enum Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    Some(ASTNode<Tree>),
    None,
}

/// `ZeroOrMany` represents zero or more occurrences of a certain AST node type, followed by a terminator.
/// It is useful for parsing lists of items with a terminator. The generated list may include `Unexpected`
/// results, as it continues parsing until the terminator is encountered, even if unexpected tokens are found.
pub(crate) struct ZeroOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    items: Vec<ParseResult<Tree>>,
    terminator: ParseResult<Terminator>,
}

/// `OneOrMany` represents a sequence that starts with at least one occurrence of a specific AST node type, followed by a terminator.
/// It ensures that at least the first item is successfully parsed. The implementation may change in the future and might be rewritten in terms of other components.
pub(crate) struct OneOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    first: ParseResult<Tree>,
    rest: Vec<ParseResult<Tree>>,
    terminator: ParseResult<Terminator>,
}

/// Implementations of the `NazmcParse` trait for different parsing structures.

impl<Tree> NazmcParse for Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn parse(iter: &mut TokensIter) -> Self {
        match ParseResult::parse(iter) {
            ParseResult::Parsed(tree) => Self::Some(tree),
            ParseResult::Unexpected { .. } => Self::None,
        }
    }
}

impl<Tree> NazmcParse for Vec<ASTNode<Tree>>
where
    ParseResult<Tree>: NazmcParse,
{
    fn parse(iter: &mut TokensIter) -> Self {
        // Parses multiple AST nodes into a `Vec`. It continues parsing until no more valid nodes are found.
        let mut items = vec![];
        while let ParseResult::Parsed(tree) = ParseResult::parse(iter) {
            items.push(tree)
        }
        items
    }
}

impl<Tree, Terminator> NazmcParse for ZeroOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn parse(iter: &mut TokensIter) -> Self {
        let mut items = vec![];

        loop {
            // No more tokens
            if iter.peek().is_none() {
                return Self {
                    items,
                    terminator: ParseResult::unexpected_eof(),
                };
            }

            match ParseResult::<Tree>::parse(iter) {
                parsed_node @ ParseResult::Parsed(..) => {
                    items.push(parsed_node);
                }
                ParseResult::Unexpected { .. } => {
                    // Check for terminator
                    if let terminator @ ParseResult::Parsed(..) =
                        ParseResult::<Terminator>::parse(iter)
                    {
                        return Self { items, terminator };
                    }
                    // Skip this unexpected token
                    iter.next_non_space_or_comment();
                }
            }
        }
    }
}

impl<Tree, Terminator> NazmcParse for OneOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn parse(iter: &mut TokensIter) -> Self {
        let first = match ParseResult::parse(iter) {
            parsed_node @ ParseResult::Parsed(..) => parsed_node,
            unexpected_node @ ParseResult::Unexpected { .. } => {
                return Self {
                    first: unexpected_node,
                    rest: vec![],
                    terminator: ParseResult::default(),
                };
            }
        };

        let zero_or_many = ZeroOrMany::parse(iter);

        Self {
            first,
            rest: zero_or_many.items,
            terminator: zero_or_many.terminator,
        }
    }
}

/// Additional utility methods for `ParseResult`, `Optional`, and the `Spanned` trait implementation.

impl<Tree> Default for ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn default() -> Self {
        Self::unexpected_eof()
    }
}

impl<Tree> ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    /// Checks if the result is a successfully parsed node.
    fn is_parsed(&self) -> bool {
        matches!(self, ParseResult::Parsed(_))
    }

    /// Checks if the result is an unexpected token.
    fn is_unexpected(&self) -> bool {
        matches!(self, ParseResult::Unexpected { .. })
    }

    /// Returns an `Unexpected` result indicating an unexpected end of file.
    fn unexpected_eof() -> Self {
        Self::Unexpected {
            span: Span::default(),
            found: TokenType::EOF,
        }
    }
}

impl<Tree> Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    /// Checks if the optional node contains a value.
    fn is_some(&self) -> bool {
        matches!(self, Self::Some(_))
    }

    /// Checks if the optional node is empty.
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// The `Spanned` trait allows retrieval of the `Span` associated with an AST node,
/// which indicates the location of the node in the source code.
pub(crate) trait Spanned {
    fn span(&self) -> Span;
}

impl<T> Spanned for ASTNode<T>
where
    ParseResult<T>: NazmcParse,
{
    fn span(&self) -> Span {
        self.span
    }
}

impl<Tree> Spanned for ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn span(&self) -> Span {
        match self {
            Self::Parsed(tree) => tree.span,
            Self::Unexpected { span, .. } => *span,
        }
    }
}

impl<Tree, Terminator> Spanned for ZeroOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn span(&self) -> Span {
        if self.items.is_empty() {
            self.terminator.span()
        } else {
            self.items[0].span().merged_with(&self.terminator.span())
        }
    }
}

impl<Tree, Terminator> Spanned for OneOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn span(&self) -> Span {
        if self.first.is_parsed() {
            self.first.span().merged_with(&self.terminator.span())
        } else if self.rest.is_empty() {
            self.terminator.span()
        } else {
            self.rest[0].span().merged_with(&self.terminator.span())
        }
    }
}

#[cfg(test)]

mod tests {
    use nazmc_parse_derive::NazmcParse;

    use super::{
        ast::{CloseParenthesisSymbol, ColonSymbol, FnKeyword, Id, OpenParenthesisSymbol},
        ASTNode, OneOrMany, Optional, ParseResult, ZeroOrMany,
    };

    // TODO:

    #[derive(NazmcParse)]
    struct SimpleFn {
        _fn: ParseResult<FnKeyword>,
        _id: ParseResult<Id>,
        _open_psren: ParseResult<OpenParenthesisSymbol>,
        _params: ASTNode<CloseParenthesisSymbol>,
    }

    // #[derive(NazmcParse)]
    // struct FnParam {
    //     _name: ParseResult<Id>,
    //     _colon: ParseResult<ColonSymbol>,
    //     _type: ParseResult<Id>,
    // }
}
