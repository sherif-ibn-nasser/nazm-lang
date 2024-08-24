use nazmc_diagnostics::span::Span;

use crate::{LexerIter, Token, TokenType};

pub(crate) mod ast;

pub(crate) type ParseResult<'a, Tree> =  Result<Tree, ParseError<'a, Tree>>;

/// The trait for all AST nodes to implement
pub(crate) trait NazmcParse<'a> where Self: std::marker::Sized {
    fn parse(lexer: &mut LexerIter<'a>) -> ParseResult<'a, Self>;
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

pub(crate) enum ParseError<'a, Tree> {
    /// Triggered when a child in the node tree has a parse error
    IncompleteTree(Tree),
    /// Triggered when a mismatch in tokens happen
    UnexpectedToken { expected: TokenType, found: Token<'a> },
}