
use std::marker::PhantomData;

use ast::{FnKeyword, Id};
use nazmc_diagnostics::span::Span;
use nazmc_parse_derive::NazmcParse;

use crate::{LexerIter, Token, TokenType};

pub(crate) mod ast;

pub(crate) type Required<Tree: NazmcParse> =  Result<Tree, ParseError<Tree>>;

pub(crate) type Optional<Tree: NazmcParse> =  Option<Tree>;

pub(crate) type ZeroOrMany<Tree: NazmcParse> =  Vec<Required<Tree>>;

pub(crate) enum ParseError<Tree> {
    /// Triggered when a child in the node tree has a parse error
    IncompleteTree(Tree),
    /// Triggered when a mismatch in tokens happen
    UnexpectedToken { expected: TokenType, found: (Span, TokenType) },
}

/// The trait for all AST nodes to implement
pub(crate) trait NazmcParse where Self: std::marker::Sized {
    fn parse(lexer: &mut LexerIter) -> Required<Self>;
}

impl<'a> LexerIter<'a> {
    fn next_non_space_or_comment(&mut self) -> Option<Token<'a>> {
        
        let mut next = self.next();

        while let Some(
            Token {
                typ: TokenType::EOL | TokenType::DelimitedComment | TokenType::LineComment | TokenType::Space,
                ..
            }
        ) = next { next = self.next(); }

        next
    }
}


#[derive(NazmcParse)]
pub(crate) struct S {
    span: Span,
    fn_keyword: FnKeyword,
}

pub(crate) fn assert_nazmc_parse_is_implemented<T: NazmcParse>() {}