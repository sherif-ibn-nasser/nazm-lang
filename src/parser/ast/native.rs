use nazmc_diagnostics::span::Span;

use crate::{parser::{NazmcParse, ParseError, ParseResult}, LexerIter, Token, TokenType, KeywordType, SymbolType};

use paste::paste;

pub(crate) struct Id { span: Span, val: String }

impl<'a> NazmcParse<'a> for Id {
    fn parse(lexer: &mut LexerIter<'a>) -> ParseResult<'a, Self> {
        match lexer.next_non_space_or_comment() {
            Some(Token { val, span, typ: TokenType::Id })
                => ParseResult::Ok(Id { span, val: val.to_string() }),
            Some(token) => ParseResult::Err(
                ParseError::UnexpectedToken {
                    expected: TokenType::Id,
                    found: token,
                }
            ),
            None => ParseResult::Err(
                ParseError::UnexpectedToken {
                    expected: TokenType::Id,
                    found: Token::default(), // EOF
                }
            ),
        }
    }
    
}

macro_rules! create_keyword_parser {
    ($keyword: ident) => {

        paste! {
            pub(crate) struct [<$keyword Keyword>] { span: Span }

            impl<'a> NazmcParse<'a> for [<$keyword Keyword>]{
                fn parse(lexer: &mut LexerIter<'a>) -> ParseResult<'a, Self> {
                    match lexer.next_non_space_or_comment() {
                        Some(Token { span, typ: TokenType::Keyword(KeywordType::$keyword), .. })
                            => ParseResult::Ok([<$keyword Keyword>] { span }),
                        Some(token) => ParseResult::Err(
                            ParseError::UnexpectedToken {
                                expected: TokenType::Keyword(KeywordType::$keyword),
                                found: token,
                            }
                        ),
                        None => ParseResult::Err(
                            ParseError::UnexpectedToken {
                                expected: TokenType::Keyword(KeywordType::$keyword),
                                found: Token::default(), // EOF
                            }
                        ),
                    }
                }
            }
        }
        
    };
}

macro_rules! create_symbol_parser {
    ($symbol: ident) => {

        paste! {
            pub(crate) struct [<$symbol Symbol>] { span: Span }

            impl<'a> NazmcParse<'a> for [<$symbol Symbol>]{
                fn parse(lexer: &mut LexerIter<'a>) -> ParseResult<'a, Self> {
                    match lexer.next_non_space_or_comment() {
                        Some(Token { span, typ: TokenType::Symbol(SymbolType::$symbol), .. })
                            => ParseResult::Ok([<$symbol Symbol>] { span }),
                        Some(token) => ParseResult::Err(
                            ParseError::UnexpectedToken {
                                expected: TokenType::Symbol(SymbolType::$symbol),
                                found: token,
                            }
                        ),
                        None => ParseResult::Err(
                            ParseError::UnexpectedToken {
                                expected: TokenType::Symbol(SymbolType::$symbol),
                                found: Token::default(), // EOF
                            }
                        ),
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

#[cfg(test)]
mod tests {
    use crate::{parser::{NazmcParse, ParseError}, LexerIter, SymbolType, Token, TokenType};

    use super::{CloseCurlyBracesSymbol, CloseParenthesisSymbol, FnKeyword, Id, OpenCurlyBracesSymbol, OpenParenthesisSymbol};

    #[test]
    fn test() {
        let content = "دالة البداية(/* تعليق */){}";

        let mut lexer = LexerIter::new(content);

        let _fn = FnKeyword::parse(&mut lexer);
        let _id = Id::parse(&mut lexer);
        let _open_paren = OpenParenthesisSymbol::parse(&mut lexer);
        let _close_paren = CloseParenthesisSymbol::parse(&mut lexer);
        let _open_curly = OpenCurlyBracesSymbol::parse(&mut lexer);
        let _close_curly = CloseCurlyBracesSymbol::parse(&mut lexer);

        assert!(_fn.is_ok());
        assert!(_id.is_ok());
        assert!(_open_paren.is_ok());
        assert!(_close_paren.is_ok());
        assert!(_open_curly.is_ok());
        assert!(_close_curly.is_ok());
    }

    #[test]
    fn test_fail() {
        let content = "دالة البداية(عدد: ص8){}";

        let mut lexer = LexerIter::new(content);

        let _fn = FnKeyword::parse(&mut lexer);
        let _id = Id::parse(&mut lexer);
        let _open_paren = OpenParenthesisSymbol::parse(&mut lexer);
        let _close_paren = CloseParenthesisSymbol::parse(&mut lexer);

        assert!(_fn.is_ok());
        assert!(_id.is_ok());
        assert!(_open_paren.is_ok());
        assert!(
            matches!(
                _close_paren,
                Err(
                    ParseError::UnexpectedToken {
                        expected: TokenType::Symbol(SymbolType::CloseParenthesis),
                        found: Token { typ: TokenType::Id, val: "عدد", .. }
                    }
                )
            )
        );
    }
}