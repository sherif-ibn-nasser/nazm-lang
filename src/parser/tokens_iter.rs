use nazmc_diagnostics::span::Span;

use crate::{Token, TokenType};

pub(crate) struct TokensIter<'a> {
    pub(crate) peek_idx: usize,
    tokens: &'a [Token<'a>],
}

impl<'a> TokensIter<'a> {
    pub(crate) fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            peek_idx: 0,
            tokens,
        }
    }

    #[inline]
    pub(crate) fn peek(&self) -> Option<&Token<'a>> {
        self.peek_nth(0)
    }

    #[inline]
    pub(crate) fn peek_nth(&self, nth: usize) -> Option<&Token<'a>> {
        self.nth(self.peek_idx + nth)
    }

    #[inline]
    pub(crate) fn recent(&self) -> Option<&Token<'a>> {
        self.recent_nth(0)
    }

    #[inline]
    pub(crate) fn recent_nth(&self, nth: usize) -> Option<&Token<'a>> {
        if self.peek_idx > nth {
            self.nth(self.peek_idx - nth - 1)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn nth(&self, nth: usize) -> Option<&Token<'a>> {
        if nth < self.tokens.len() {
            Some(&self.tokens[nth])
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn next(&mut self) -> Option<&Token<'a>> {
        if self.peek_idx == self.tokens.len() {
            return None;
        }
        self.peek_idx += 1;
        self.recent()
    }

    pub(crate) fn next_non_space_or_comment(&mut self) -> Option<&Token<'a>> {
        while let Some(Token {
            typ:
                TokenType::EOL | TokenType::DelimitedComment | TokenType::LineComment | TokenType::Space,
            ..
        }) = self.next()
        {}

        self.peek()
    }

    /// Returns a zero-column span located after the recent token or the zero span if no recent found
    pub(crate) fn peek_start_span(&self) -> Span {
        match self.recent() {
            Some(token) => Span::after(&token.span),
            None => Span::default(),
        }
    }
}
