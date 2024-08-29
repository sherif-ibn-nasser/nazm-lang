use std::fmt::Debug;

use ast::{CloseParenthesisSymbol, FnKeyword, Id, OpenParenthesisSymbol};
/// This module defines the core components and traits required for parsing an Abstract Syntax Tree (AST)
/// in the Nazmc language parser. It provides the foundational structures and parsing logic for different
/// AST node types, ensuring that the syntax is correctly interpreted and processed.
use nazmc_diagnostics::span::Span;
use nazmc_parse_derive::NazmcParse;
use tokens_iter::TokensIter;

use crate::{Token, TokenType};

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
pub(crate) struct SyntaxNode<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    span: Span,
    is_broken: bool,
    tree: Tree,
}

/// The default result of a parsing attempt. `ParseResult` can either be `Parsed`, indicating
/// successful parsing, or `Unexpected`, indicating an unexpected token was encountered. This
/// enum is fundamental in error reporting and control flow within the parsing process.
pub(crate) enum ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    Parsed(SyntaxNode<Tree>),
    Unexpected {
        span: Span,
        found: TokenType,
        is_start_failure: bool,
    },
}

/// `Optional` represents an optional AST node. It either contains a parsed node (`Some`) or nothing (`None`).
pub(crate) enum Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    Some(SyntaxNode<Tree>),
    None,
}

/// Parses a sequence of items where the number of items can vary from zero to many.
///
/// The `ZeroOrMany` parser continues to parse items until a terminator or an unexpected token is encountered.
/// It handles variable-length sequences robustly and attempts to recover from errors by backtracking and
/// continuing parsing if possible.
///
/// # Parsing Logic
///
/// 1. **Initial Parsing:** Begins by parsing items and tracking both successfully parsed items and unexpected tokens.
/// 2. **Continue Parsing:** Continues to parse items until a terminator (specific end token) is found or an unexpected token is encountered.
/// 3. **Error Handling:** When an unexpected token is encountered, it attempts to recover by backtracking and re-parsing the terminator. If recovery fails, it treats the unexpected token as part of the results, skips it, and resumes parsing.
/// 4. **Return Results:** Returns the collected items and any terminator information or handles unexpected tokens as needed.
///
/// You can think of it as a loop that processes two variants: items (trees) and terminators.
/// The loop continues parsing items, and if parsing an item fails, it backtracks and tries to parse the terminator.
/// Once the terminator is found, the loop ends.
///
/// # Parameters
/// - `item_parser`: A parser for individual items in the sequence.
/// - `terminator_parser`: A parser for the terminator that signifies the end of the sequence.
///
/// # Returns
/// A result containing the successfully parsed items, terminator information if present, or details about any errors encountered during parsing.
///
/// # Errors
/// The parser will handle unexpected tokens by attempting recovery or including them in the results if recovery fails.

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

impl<ParseMethod> NazmcParse for Box<ParseMethod>
where
    ParseMethod: NazmcParse,
{
    fn parse(iter: &mut TokensIter) -> Self {
        let parsed = ParseMethod::parse(iter);
        Box::new(parsed)
    }
}

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

impl<Tree> NazmcParse for Vec<SyntaxNode<Tree>>
where
    ParseResult<Tree>: NazmcParse,
{
    fn parse(iter: &mut TokensIter) -> Self {
        // Parses multiple AST nodes into a `Vec`. It continues parsing until no more valid nodes are found.
        let mut items = vec![];
        loop {
            let peek_idx = iter.peek_idx;
            match ParseResult::parse(iter) {
                ParseResult::Parsed(tree) => items.push(tree),
                _ => {
                    iter.peek_idx = peek_idx; // Backtrack
                    break;
                }
            }
        }
        items
    }
}

impl<Tree, Terminator> NazmcParse for ZeroOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    /// see [ZeroOrMany]
    fn parse(iter: &mut TokensIter) -> Self {
        let mut items = vec![];

        loop {
            // No more tokens
            if iter.peek().is_none() {
                return Self {
                    items,
                    terminator: ParseResult::unexpected_eof(iter.peek_start_span()),
                };
            }
            let old_peek_idx = iter.peek_idx;
            match ParseResult::<Tree>::parse(iter) {
                parsed_node @ ParseResult::Parsed(..) => {
                    items.push(parsed_node);
                }
                unexpected_token @ ParseResult::Unexpected { .. } => {
                    let new_peek_idx = iter.peek_idx;

                    iter.peek_idx = old_peek_idx; // Try to backtrack and parse the terminator

                    // Check for terminator
                    if let terminator @ ParseResult::Parsed(..) =
                        ParseResult::<Terminator>::parse(iter)
                    {
                        return Self { items, terminator };
                    }

                    // Backtracking doesn't work either
                    // so add this unexpected result to items
                    // then reset to failure and skip this unexpected token
                    items.push(unexpected_token);
                    iter.peek_idx = new_peek_idx;
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
                    terminator: ParseResult::unexpected_eof(iter.peek_start_span()),
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

impl<Tree> Debug for ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsed(tree) => f
                .debug_struct("Parsed")
                .field("is_broken", &tree.is_broken)
                .finish(),
            Self::Unexpected {
                span,
                found,
                is_start_failure,
            } => f
                .debug_struct("Unexpected")
                .field("span", span)
                .field("found", found)
                .field("is_start_failure", is_start_failure)
                .finish(),
        }
    }
}

impl<Tree> ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    /// Checks if the result is an unexpected token.
    pub(crate) fn is_unexpected(&self) -> bool {
        matches!(self, ParseResult::Unexpected { .. })
    }

    /// Returns an `Unexpected` result indicating an unexpected end of file.
    pub(crate) fn unexpected_eof(span: Span) -> Self {
        Self::Unexpected {
            span,
            found: TokenType::EOF,
            is_start_failure: true,
        }
    }

    pub(crate) fn unwrap(self) -> SyntaxNode<Tree> {
        let ParseResult::Parsed(tree) = self else {
            panic!("Calling `unwrap` on {:?}", self);
        };
        tree
    }
}

impl<Tree> Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    /// Checks if the optional node contains a value.
    pub(crate) fn is_some(&self) -> bool {
        matches!(self, Self::Some(_))
    }

    /// Checks if the optional node contains a successfully parsed node.
    pub(crate) fn is_some_and_valid(&self) -> bool {
        matches!(
            self,
            Self::Some(SyntaxNode {
                is_broken: false,
                ..
            })
        )
    }

    /// Checks if the optional node contains a broken parsed node.
    pub(crate) fn is_some_and_broken(&self) -> bool {
        matches!(
            self,
            Self::Some(SyntaxNode {
                is_broken: true,
                ..
            })
        )
    }

    /// Checks if the optional node is empty.
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub(crate) fn unwrap(self) -> SyntaxNode<Tree> {
        match self {
            Self::Some(tree) => tree,
            Self::None => panic!("Calling `unwrap` on Optional::None"),
        }
    }
}

pub(crate) trait IsParsed {
    fn is_parsed(&self) -> bool {
        self.is_parsed_and_broken() || self.is_parsed_and_valid()
    }

    fn is_parsed_and_valid(&self) -> bool;

    fn is_parsed_and_broken(&self) -> bool;
}

impl<ParseMethod> IsParsed for Box<ParseMethod>
where
    ParseMethod: NazmcParse + IsParsed,
{
    fn is_parsed_and_valid(&self) -> bool {
        ParseMethod::is_parsed_and_valid(self)
    }

    fn is_parsed_and_broken(&self) -> bool {
        ParseMethod::is_parsed_and_broken(self)
    }

    fn is_parsed(&self) -> bool {
        ParseMethod::is_parsed(self)
    }
}

impl<Tree> IsParsed for ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    /// Checks if the result is a parsed node.
    fn is_parsed(&self) -> bool {
        matches!(self, ParseResult::Parsed(_))
    }

    /// Checks if the result is a successfully parsed node.
    fn is_parsed_and_valid(&self) -> bool {
        matches!(
            self,
            ParseResult::Parsed(SyntaxNode {
                is_broken: false,
                ..
            })
        )
    }

    /// Checks if the result is a broken parsed node.
    fn is_parsed_and_broken(&self) -> bool {
        matches!(
            self,
            ParseResult::Parsed(SyntaxNode {
                is_broken: true,
                ..
            })
        )
    }
}

impl<Tree> IsParsed for Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn is_parsed(&self) -> bool {
        true // It is always parsed
    }

    fn is_parsed_and_valid(&self) -> bool {
        self.is_some_and_valid() || self.is_none() // None is parsed and valid
    }

    fn is_parsed_and_broken(&self) -> bool {
        self.is_some_and_broken()
    }
}

impl<Tree> IsParsed for Vec<SyntaxNode<Tree>>
where
    ParseResult<Tree>: NazmcParse,
{
    fn is_parsed(&self) -> bool {
        true // The vec is always is parsed as it may parse with no nodes
    }

    fn is_parsed_and_valid(&self) -> bool {
        self.iter().all(|tree| !tree.is_broken)
    }

    fn is_parsed_and_broken(&self) -> bool {
        self.iter().any(|tree| tree.is_broken)
    }
}

impl<Tree, Terminator> IsParsed for ZeroOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn is_parsed(&self) -> bool {
        self.items.iter().all(|item| item.is_parsed()) && self.terminator.is_parsed()
    }

    fn is_parsed_and_valid(&self) -> bool {
        self.items.iter().all(|item| item.is_parsed_and_valid())
            && self.terminator.is_parsed_and_valid()
    }

    fn is_parsed_and_broken(&self) -> bool {
        self.items.iter().any(|item| item.is_parsed_and_broken())
            || self.terminator.is_parsed_and_broken()
    }
}

impl<Tree, Terminator> IsParsed for OneOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn is_parsed(&self) -> bool {
        self.first.is_parsed()
            && self.rest.iter().all(|item| item.is_parsed())
            && self.terminator.is_parsed()
    }

    fn is_parsed_and_valid(&self) -> bool {
        self.first.is_parsed_and_valid()
            && self.rest.iter().all(|item| item.is_parsed_and_valid())
            && self.terminator.is_parsed_and_valid()
    }

    fn is_parsed_and_broken(&self) -> bool {
        self.first.is_parsed_and_broken()
            || self.rest.iter().any(|item| item.is_parsed_and_broken())
            || self.terminator.is_parsed_and_broken()
    }
}

/// The `Spanned` trait allows retrieval of the `Span` associated with an AST node,
/// which indicates the location of the node in the source code.
pub(crate) trait Spanned {
    fn span(&self) -> Option<Span>;
}

impl<ParseMethod> Spanned for Box<ParseMethod>
where
    ParseMethod: NazmcParse + Spanned,
{
    fn span(&self) -> Option<Span> {
        ParseMethod::span(self)
    }
}

impl<T> Spanned for SyntaxNode<T>
where
    ParseResult<T>: NazmcParse,
{
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl<Tree> Spanned for ParseResult<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn span(&self) -> Option<Span> {
        match self {
            Self::Parsed(tree) => Some(tree.span),
            Self::Unexpected { span, .. } => Some(*span),
        }
    }
}

impl<Tree> Spanned for Optional<Tree>
where
    ParseResult<Tree>: NazmcParse,
{
    fn span(&self) -> Option<Span> {
        match self {
            Self::Some(tree) => Some(tree.span),
            Self::None => None,
        }
    }
}

impl<Tree> Spanned for Vec<SyntaxNode<Tree>>
where
    ParseResult<Tree>: NazmcParse,
{
    fn span(&self) -> Option<Span> {
        if self.is_empty() {
            None
        } else {
            Some(self[0].span.merged_with(&self.last().unwrap().span))
        }
    }
}

impl<Tree, Terminator> Spanned for ZeroOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn span(&self) -> Option<Span> {
        if self.items.is_empty() {
            self.terminator.span()
        } else {
            Some(
                self.items[0]
                    .span()
                    .unwrap()
                    .merged_with(&self.terminator.span().unwrap()),
            )
        }
    }
}

impl<Tree, Terminator> Spanned for OneOrMany<Tree, Terminator>
where
    ParseResult<Tree>: NazmcParse,
    ParseResult<Terminator>: NazmcParse,
{
    fn span(&self) -> Option<Span> {
        if self.first.is_parsed() {
            Some(
                self.first
                    .span()
                    .unwrap()
                    .merged_with(&self.terminator.span().unwrap()),
            )
        } else if self.rest.is_empty() {
            self.terminator.span()
        } else {
            Some(
                self.rest[0]
                    .span()
                    .unwrap()
                    .merged_with(&self.terminator.span().unwrap()),
            )
        }
    }
}

#[cfg(test)]
mod tests {

    use ast::*;

    use crate::LexerIter;

    use super::*;

    #[derive(NazmcParse)]
    pub(crate) enum TermBinOp {
        Plus(SyntaxNode<PlusSymbol>),
        Minus(SyntaxNode<MinusSymbol>),
    }

    #[derive(NazmcParse)]
    pub(crate) struct SimpleFn {
        pub(crate) _fn: SyntaxNode<FnKeyword>,
        pub(crate) _id: ParseResult<Id>,
        pub(crate) _params_decl: ParseResult<FnParams>,
    }

    #[derive(NazmcParse)]
    pub(crate) struct FnParams {
        pub(crate) _open_paren: SyntaxNode<OpenParenthesisSymbol>,
        pub(crate) _params: ZeroOrMany<FnParamWithComma, FnParamWithCloseParenthesis>,
    }

    #[derive(NazmcParse)]
    pub(crate) struct FnParamWithComma {
        _fn_param: SyntaxNode<FnParam>,
        _comma: SyntaxNode<CommaSymbol>,
    }

    #[derive(NazmcParse)]
    pub(crate) struct FnParamWithCloseParenthesis {
        _fn_param: Optional<FnParam>,
        _close_paren: SyntaxNode<CloseParenthesisSymbol>,
    }

    #[derive(NazmcParse)]
    pub(crate) struct FnParam {
        pub(crate) _name: SyntaxNode<Id>,
        pub(crate) _colon: ParseResult<ColonSymbol>,
        pub(crate) _type: ParseResult<Id>,
    }

    #[test]
    fn test_wrong_params() {
        let (tokens, ..) =
            LexerIter::new("دالة البداية(123 دالة، ت: ح 444، س: ص، ع: ك،) {}").collect_all();
        let mut tokens_iter = TokensIter::new(&tokens);
        tokens_iter.next(); // Init recent

        let parse_result = <ParseResult<SimpleFn>>::parse(&mut tokens_iter);

        let ParseResult::Parsed(fn_node) = parse_result else {
            panic!();
        };

        assert!(fn_node.is_broken);
        assert!(!fn_node.tree._fn.is_broken);

        let params_decl = fn_node.tree._params_decl.unwrap();
        assert!(params_decl.is_broken);
        assert!(!params_decl.tree._open_paren.is_broken);

        let params = params_decl.tree._params;
        assert!(params.terminator.is_parsed_and_valid()); // It found the close paren

        for item in params.items {
            println!("{:?}", item)
        }

        println!("-----");
        println!("{:?}", params.terminator);
    }

    #[test]
    fn test_enum() {
        let (tokens, ..) = LexerIter::new("+-  /** */ - +").collect_all();
        let mut tokens_iter = TokensIter::new(&tokens);
        tokens_iter.next(); // Init recent

        let parse_result = <ParseResult<TermBinOp>>::parse(&mut tokens_iter);
        assert!(parse_result.is_parsed_and_valid());
        let op = parse_result.unwrap().tree;
        assert!(matches!(
            op,
            TermBinOp::Plus(SyntaxNode {
                is_broken: false,
                ..
            })
        ));

        let parse_result = <ParseResult<TermBinOp>>::parse(&mut tokens_iter);
        assert!(parse_result.is_parsed_and_valid());
        let op = parse_result.unwrap().tree;
        assert!(matches!(
            op,
            TermBinOp::Minus(SyntaxNode {
                is_broken: false,
                ..
            })
        ));

        let parse_result = <ParseResult<TermBinOp>>::parse(&mut tokens_iter);
        assert!(parse_result.is_parsed_and_valid());
        let op = parse_result.unwrap().tree;
        assert!(matches!(
            op,
            TermBinOp::Minus(SyntaxNode {
                is_broken: false,
                ..
            })
        ));

        let parse_result = <ParseResult<TermBinOp>>::parse(&mut tokens_iter);
        assert!(parse_result.is_parsed_and_valid());
        let op = parse_result.unwrap().tree;
        assert!(matches!(
            op,
            TermBinOp::Plus(SyntaxNode {
                is_broken: false,
                ..
            })
        ));
    }

    #[test]
    fn test_zero_params() {
        let (tokens, ..) = LexerIter::new("دالة البداية() {}").collect_all();
        let mut tokens_iter = TokensIter::new(&tokens);
        tokens_iter.next(); // Init recent

        let parse_result = <ParseResult<SimpleFn>>::parse(&mut tokens_iter);

        let ParseResult::Parsed(fn_node) = parse_result else {
            panic!();
        };

        assert!(!fn_node.tree._fn.is_broken);
        assert!(fn_node.tree._id.is_parsed_and_valid());

        let params_decl = fn_node.tree._params_decl.unwrap();
        assert!(!params_decl.is_broken);
        assert!(!params_decl.tree._open_paren.is_broken);

        let params = params_decl.tree._params;
        assert!(params.items.is_empty());
        let terminator = params.terminator.unwrap();

        assert!(!terminator.is_broken);
        assert!(terminator.tree._fn_param.is_none()); // No params
        assert!(!terminator.tree._close_paren.is_broken);
    }

    #[test]
    fn test_one_param_no_trailing_comma() {
        let (tokens, ..) = LexerIter::new("دالة البداية(س: ص8) {}").collect_all();
        let mut tokens_iter = TokensIter::new(&tokens);
        tokens_iter.next(); // Init recent

        let parse_result = <ParseResult<SimpleFn>>::parse(&mut tokens_iter);

        let ParseResult::Parsed(fn_node) = parse_result else {
            panic!();
        };

        assert!(!fn_node.tree._fn.is_broken);
        assert!(fn_node.tree._id.is_parsed_and_valid());

        let params_decl = fn_node.tree._params_decl.unwrap();
        assert!(!params_decl.is_broken);
        assert!(!params_decl.tree._open_paren.is_broken);

        let params = params_decl.tree._params;
        assert!(params.items.is_empty());
        let terminator = params.terminator.unwrap();

        assert!(!terminator.is_broken);
        assert!(terminator.tree._fn_param.is_some_and_valid()); // The first param
        assert!(!terminator.tree._close_paren.is_broken);
    }

    #[test]
    fn test_one_param_with_trailing_comma() {
        let (tokens, ..) = LexerIter::new("دالة البداية(س: ص8،) {}").collect_all();
        let mut tokens_iter = TokensIter::new(&tokens);
        tokens_iter.next(); // Init recent

        let parse_result = <ParseResult<SimpleFn>>::parse(&mut tokens_iter);

        let ParseResult::Parsed(fn_node) = parse_result else {
            panic!();
        };

        assert!(!fn_node.tree._fn.is_broken);
        assert!(fn_node.tree._id.is_parsed_and_valid());

        let params_decl = fn_node.tree._params_decl.unwrap();
        assert!(!params_decl.is_broken);
        assert!(!params_decl.tree._open_paren.is_broken);

        let params = params_decl.tree._params;
        assert!(params.items.len() == 1); // One param with a comma found
        assert!(params.items[0].is_parsed_and_valid());
        let terminator = params.terminator.unwrap();

        assert!(!terminator.is_broken);
        assert!(terminator.tree._fn_param.is_none()); // No params after the trailing comma
        assert!(!terminator.tree._close_paren.is_broken);
    }

    #[test]
    fn test_two_params_no_trailing_comma() {
        let (tokens, ..) = LexerIter::new("دالة البداية(س: ص8، ك: م) {}").collect_all();
        let mut tokens_iter = TokensIter::new(&tokens);
        tokens_iter.next(); // Init recent

        let parse_result = <ParseResult<SimpleFn>>::parse(&mut tokens_iter);

        let ParseResult::Parsed(fn_node) = parse_result else {
            panic!();
        };

        assert!(!fn_node.tree._fn.is_broken);
        assert!(fn_node.tree._id.is_parsed_and_valid());

        let params_decl = fn_node.tree._params_decl.unwrap();
        assert!(!params_decl.is_broken);
        assert!(!params_decl.tree._open_paren.is_broken);

        let params = params_decl.tree._params;
        assert!(params.items.len() == 1); // One param with a comma are found
        assert!(params.items[0].is_parsed_and_valid());
        let terminator = params.terminator.unwrap();

        assert!(!terminator.is_broken);
        assert!(terminator.tree._fn_param.is_some_and_valid()); // A param after the lat comma is found
        assert!(!terminator.tree._close_paren.is_broken);
    }

    #[test]
    fn test_two_params_with_trailing_comma() {
        let (tokens, ..) = LexerIter::new("دالة البداية(س: ص8، ك: م،) {}").collect_all();
        let mut tokens_iter = TokensIter::new(&tokens);
        tokens_iter.next(); // Init recent

        let parse_result = <ParseResult<SimpleFn>>::parse(&mut tokens_iter);

        let ParseResult::Parsed(fn_node) = parse_result else {
            panic!();
        };

        assert!(!fn_node.tree._fn.is_broken);
        assert!(fn_node.tree._id.is_parsed_and_valid());

        let params_decl = fn_node.tree._params_decl.unwrap();
        assert!(!params_decl.is_broken);
        assert!(!params_decl.tree._open_paren.is_broken);

        let params = params_decl.tree._params;
        assert!(params.items.len() == 2); // Two param with a comma are found
        assert!(params.items[0].is_parsed_and_valid());
        assert!(params.items[1].is_parsed_and_valid());
        let terminator = params.terminator.unwrap();

        assert!(!terminator.is_broken);
        assert!(terminator.tree._fn_param.is_none()); // No params found after the trailing comma
        assert!(!terminator.tree._close_paren.is_broken);
    }
}
