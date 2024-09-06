use crate::{
    parser::{NazmcParse, ParseResult, SyntaxNode, TokensIter},
    KeywordKind, SymbolKind, Token, TokenKind,
};

use paste::paste;

pub(crate) struct Id {
    val: String,
}

impl NazmcParse for ParseResult<Id> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(Token {
                val,
                span,
                kind: TokenKind::Id,
            }) => {
                let ok = ParseResult::Parsed(SyntaxNode {
                    span: *span,
                    is_broken: false,
                    tree: Id {
                        val: val.to_string(),
                    },
                });
                iter.next_non_space_or_comment();
                ok
            }
            Some(token) => ParseResult::Unexpected {
                span: token.span,
                found: token.kind.clone(),
                is_start_failure: true,
            },
            None => ParseResult::unexpected_eof(iter.peek_start_span()),
        }
    }
}

macro_rules! create_keyword_parser {
    ($keyword: ident) => {
        paste! {
            pub(crate) struct [<$keyword Keyword>];

            impl NazmcParse for ParseResult<[<$keyword Keyword>]>{

                fn parse(iter: &mut TokensIter) -> Self {

                    match iter.recent() {
                        Some(Token { span, kind: TokenKind::Keyword(KeywordKind::$keyword), .. }) => {
                            let ok = ParseResult::Parsed(SyntaxNode {
                                span: *span,
                                is_broken: false,
                                tree: [<$keyword Keyword>],
                            });
                            iter.next_non_space_or_comment();
                            ok
                        }
                        Some(token) => ParseResult::Unexpected {
                            span: token.span,
                            found: token.kind.clone(),
                            is_start_failure: true,
                        },
                        None => ParseResult::unexpected_eof(iter.peek_start_span()),
                    }
                }
            }
        }
    };
}

macro_rules! create_symbol_parser {
    ($symbol: ident) => {
        paste! {
            pub(crate) struct [<$symbol Symbol>];

            impl NazmcParse for ParseResult<[<$symbol Symbol>]> {

                fn parse(iter: &mut TokensIter) -> Self {

                    match iter.recent() {
                        Some(Token { span, kind: TokenKind::Symbol(SymbolKind::$symbol), .. }) =>
                        {
                            let ok = ParseResult::Parsed(SyntaxNode {
                                span: *span,
                                is_broken: false,
                                tree: [<$symbol Symbol>],
                            });
                            iter.next_non_space_or_comment();
                            ok
                        }
                        Some(token) => ParseResult::Unexpected {
                            span: token.span,
                            found: token.kind.clone(),
                            is_start_failure: true,
                        },
                        None => ParseResult::unexpected_eof(iter.peek_start_span()),
                    }
                }
            }
        }
    };
}

create_keyword_parser!(Fn);
create_keyword_parser!(Let);
create_keyword_parser!(Mut);
create_keyword_parser!(Const);
create_keyword_parser!(Struct);
create_keyword_parser!(Public);
create_keyword_parser!(Private);
create_keyword_parser!(On);

create_symbol_parser!(LessDotDotLess);
create_symbol_parser!(LessDotDot);
create_symbol_parser!(DotDotLess);
create_symbol_parser!(ShrEqual);
create_symbol_parser!(ShlEqual);
create_symbol_parser!(PowerEqual);
create_symbol_parser!(LessEqual);
create_symbol_parser!(Shr);
create_symbol_parser!(GreaterEqual);
create_symbol_parser!(Shl);
create_symbol_parser!(StarEqual);
create_symbol_parser!(Power);
create_symbol_parser!(SlashEqual);
create_symbol_parser!(PLusEqual);
create_symbol_parser!(PlusPlus);
create_symbol_parser!(MinusEqual);
create_symbol_parser!(MinusMinus);
create_symbol_parser!(BitOrEqual);
create_symbol_parser!(LogicalOr);
create_symbol_parser!(BitAndEqual);
create_symbol_parser!(LogicalAnd);
create_symbol_parser!(ModuloEqual);
create_symbol_parser!(BitNotEqual);
create_symbol_parser!(XorEqual);
create_symbol_parser!(EqualEqual);
create_symbol_parser!(NotEqual);
create_symbol_parser!(DoubleColons);
create_symbol_parser!(DotDot);
create_symbol_parser!(Comma);
create_symbol_parser!(Semicolon);
create_symbol_parser!(QuestionMark);
create_symbol_parser!(OpenParenthesis);
create_symbol_parser!(CloseParenthesis);
create_symbol_parser!(OpenCurlyBraces);
create_symbol_parser!(CloseCurlyBraces);
create_symbol_parser!(OpenSquareBracket);
create_symbol_parser!(CloseSquareBracket);
create_symbol_parser!(Dot);
create_symbol_parser!(OpenAngleBracketOrLess);
create_symbol_parser!(CloseAngleBracketOrGreater);
create_symbol_parser!(Star);
create_symbol_parser!(Slash);
create_symbol_parser!(Plus);
create_symbol_parser!(Minus);
create_symbol_parser!(BitOr);
create_symbol_parser!(BitAnd);
create_symbol_parser!(Modulo);
create_symbol_parser!(BitNot);
create_symbol_parser!(Xor);
create_symbol_parser!(ExclamationMark);
create_symbol_parser!(Colon);
create_symbol_parser!(Equal);
create_symbol_parser!(Hash);

#[cfg(test)]
mod tests {
    use crate::{
        parser::{IsParsed, NazmcParse, ParseResult, TokensIter},
        LexerIter, TokenKind,
    };

    use super::{
        CloseCurlyBracesSymbol, CloseParenthesisSymbol, FnKeyword, Id, OpenCurlyBracesSymbol,
        OpenParenthesisSymbol,
    };

    #[test]
    fn test() {
        let content = "دالة البداية(/* تعليق */){}";

        let lexer = LexerIter::new(content);
        let (tokens, ..) = lexer.collect_all();
        let mut iter = TokensIter::new(&tokens);
        iter.next(); // Initialize the value of recent

        let _fn = ParseResult::<FnKeyword>::parse(&mut iter);
        let _id = ParseResult::<Id>::parse(&mut iter);
        let _open_paren = ParseResult::<OpenParenthesisSymbol>::parse(&mut iter);
        let _close_paren = ParseResult::<CloseParenthesisSymbol>::parse(&mut iter);
        let _open_curly = ParseResult::<OpenCurlyBracesSymbol>::parse(&mut iter);
        let _close_curly = ParseResult::<CloseCurlyBracesSymbol>::parse(&mut iter);

        assert!(_fn.is_parsed());
        assert!(_id.is_parsed());
        assert!(_open_paren.is_parsed());
        assert!(_close_paren.is_parsed());
        assert!(_open_curly.is_parsed());
        assert!(_close_curly.is_parsed());
    }

    #[test]
    fn test_fail() {
        let content = "دالة البداية(عدد: ص8){}";

        let lexer = LexerIter::new(content);

        let (tokens, ..) = lexer.collect_all();
        let mut iter = TokensIter::new(&tokens);
        iter.next(); // Initialize the value of recent

        let _fn = ParseResult::<FnKeyword>::parse(&mut iter);
        let _id = ParseResult::<Id>::parse(&mut iter);
        let _open_paren = ParseResult::<OpenParenthesisSymbol>::parse(&mut iter);
        let _close_paren = ParseResult::<CloseParenthesisSymbol>::parse(&mut iter);

        assert!(_fn.is_parsed());
        assert!(_id.is_parsed());
        assert!(_open_paren.is_parsed());
        assert!(matches!(
            _close_paren,
            ParseResult::Unexpected {
                found: TokenKind::Id,
                ..
            }
        ));
    }
}
