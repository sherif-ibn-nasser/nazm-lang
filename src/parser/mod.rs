use nazmc_diagnostics::span::{self, Span};
use nazmc_parse_derive::NazmcParse;
use tokens_iter::TokensIter;

use crate::TokenType;

pub(crate) mod ast;

pub(crate) mod tokens_iter;

/// The trait for all AST nodes to implement
pub(crate) trait NazmcParse
where
    Self: Sized,
{
    fn parse(iter: &mut TokensIter) -> Self;
}

pub(crate) struct ASTNode<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    span: Span,
    tree: Tree,
}

/// The default parsing method
pub(crate) enum ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    Parsed(ASTNode<Tree>),
    Unexpected { span: Span, found: TokenType },
}

// Parsing methods

pub(crate) enum Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    Some(ASTNode<Tree>),
    None,
}

pub(crate) struct ZeroOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    items: Vec<ParseResult<Tree>>,
    terminator: ParseResult<Terminator>,
}

pub(crate) struct OneOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    first: ParseResult<Tree>,
    rest: Vec<ParseResult<Tree>>,
    terminator: ParseResult<Terminator>,
}

// Parsing methods implementation

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

// Utility methods

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
    fn is_parsed(&self) -> bool {
        matches!(self, ParseResult::Parsed(_))
    }

    fn is_unexpected(&self) -> bool {
        matches!(self, ParseResult::Unexpected { .. })
    }

    fn unexpected_eof() -> Self {
        Self::Unexpected {
            span: Span::default(),
            found: TokenType::default(),
        }
    }
}

impl<Tree> Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn is_some(&self) -> bool {
        matches!(self, Self::Some(_))
    }

    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

// Spanned

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

impl<Tree> Spanned for Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn span(&self) -> Span {
        match self {
            Optional::Some(node) => node.span(),
            Optional::None => Span::default(),
        }
    }
}

impl<Tree> Spanned for Vec<ASTNode<Tree>>
where
    ParseResult<Tree>: NazmcParse,
{
    fn span(&self) -> Span {
        if self.is_empty() {
            Span::default()
        } else {
            self[0].span().merged_with(&self[self.len() - 1].span())
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

    // #[derive(NazmcParse)]
    // struct SimpleFn {
    //     _fn: ParseResult<FnKeyword>,
    //     _id: ParseResult<Id>,
    //     _open_psren: ParseResult<OpenParenthesisSymbol>,
    //     _params: ZeroOrMany<FnParam, CloseParenthesisSymbol>,
    // }

    // #[derive(NazmcParse)]
    // struct FnParam {
    //     _name: ParseResult<Id>,
    //     _colon: ParseResult<ColonSymbol>,
    //     _type: ParseResult<Id>,
    // }
}
