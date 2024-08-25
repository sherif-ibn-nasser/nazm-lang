use nazmc_diagnostics::span::Span;
use nazmc_parse_derive::NazmcParse;
use tokens_iter::TokensIter;

use crate::TokenType;

pub(crate) mod ast;

pub(crate) mod tokens_iter;

pub(crate) enum ParseError<Tree> {
    /// Triggered when a child in the node tree has a parse error
    IncompleteTree(Tree),
    /// Triggered when a mismatch in tokens happen
    UnexpectedToken { expected: TokenType, found: (Span, TokenType) },
}

/// The trait for all AST nodes to implement
pub(crate) trait NazmcParse where Self: std::marker::Sized {
    fn parse(iter: &mut TokensIter) -> Self;
}

pub(crate) type Required<Tree> =  Result<Tree, ParseError<Tree>>;

impl<T> NazmcParse for Option<T> where Required<T>: NazmcParse{

    fn parse(iter: &mut TokensIter) -> Self {
        
        let peek_idx = iter.peek_idx;

        match Required::<T>::parse(iter) {
            Ok(tree) => Some(tree),
            Err(_) => {
                iter.peek_idx = peek_idx;
                None
            },
        }
    }
}