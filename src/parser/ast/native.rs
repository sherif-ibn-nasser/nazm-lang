use nazmc_diagnostics::span::Span;

use crate::{parser::{NazmcParse, ParseError, Required, TokensIter}, KeywordType, SymbolType, Token, TokenType};

use paste::paste;

pub(crate) struct Id { span: Span, val: String }

impl NazmcParse for Required<Id> {

    fn parse(iter: &mut TokensIter) -> Self {
        
        let peek_idx = iter.peek_idx;

        match iter.next_non_space_or_comment() {
            Some(Token { val, span, typ: TokenType::Id })
                => Required::Ok(Id { span: *span, val: val.to_string() }),
            Some(token) => {
                let err = Required::Err(
                    ParseError::UnexpectedToken {
                        expected: TokenType::Id,
                        found: (token.span, token.typ.clone()),
                    }
                );
                iter.peek_idx = peek_idx;
                err
            }
            None => {
                let err = Required::Err(
                    ParseError::UnexpectedToken {
                        expected: TokenType::Id,
                        found: (Span::default(), TokenType::EOF),
                    }
                );
                iter.peek_idx = peek_idx;
                err
            },
        }
    }
    
}

macro_rules! create_keyword_parser {
    ($keyword: ident) => {

        paste! {
            pub(crate) struct [<$keyword Keyword>] { span: Span }

            impl NazmcParse for Required<[<$keyword Keyword>]>{

                fn parse(iter: &mut TokensIter) -> Self {

                    let peek_idx = iter.peek_idx;

                    match iter.next_non_space_or_comment() {
                        Some(Token { span, typ: TokenType::Keyword(KeywordType::$keyword), .. })
                            => Required::Ok([<$keyword Keyword>] { span: *span }),
                        Some(token) => {
                            let err = Required::Err(
                                ParseError::UnexpectedToken {
                                    expected: TokenType::Keyword(KeywordType::$keyword),
                                    found: (token.span, token.typ.clone()),
                                }
                            );
                            iter.peek_idx = peek_idx;
                            err
                        },
                        None => {
                            let err = Required::Err(
                                ParseError::UnexpectedToken {
                                    expected: TokenType::Keyword(KeywordType::$keyword),
                                    found: (Span::default(), TokenType::EOF),
                                }
                            );
                            iter.peek_idx = peek_idx;
                            err
                        },
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

            impl NazmcParse for Required<[<$symbol Symbol>]> {

                fn parse(iter: &mut TokensIter) -> Self {

                    let peek_idx = iter.peek_idx;

                    match iter.next_non_space_or_comment() {
                        Some(Token { span, typ: TokenType::Symbol(SymbolType::$symbol), .. })
                            => Required::Ok([<$symbol Symbol>] { span: *span }),
                        Some(token) => {
                            let err = Required::Err(
                                ParseError::UnexpectedToken {
                                    expected: TokenType::Symbol(SymbolType::$symbol),
                                    found: (token.span, token.typ.clone()),
                                }
                            );
                            iter.peek_idx = peek_idx;
                            err
                        },
                        None => {
                            let err = Required::Err(
                                ParseError::UnexpectedToken {
                                    expected: TokenType::Symbol(SymbolType::$symbol),
                                    found: (Span::default(), TokenType::EOF),
                                }
                            );
                            iter.peek_idx = peek_idx;
                            err
                        },
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
    use crate::{parser::{NazmcParse, ParseError, Required, TokensIter}, LexerIter, SymbolType, TokenType};

    use super::{CloseCurlyBracesSymbol, CloseParenthesisSymbol, FnKeyword, Id, OpenCurlyBracesSymbol, OpenParenthesisSymbol};

    #[test]
    fn test() {
        let content = "دالة البداية(/* تعليق */){}";

        let mut lexer = LexerIter::new(content);
        let (tokens, ..) = lexer.collect_all();
        let mut iter = TokensIter::new(&tokens);

        let _fn = Required::<FnKeyword>::parse(&mut iter);
        let _id = Required::<Id>::parse(&mut iter);
        let _open_paren = Required::<OpenParenthesisSymbol>::parse(&mut iter);
        let _close_paren =Required::<CloseParenthesisSymbol>::parse(&mut iter);
        let _open_curly = Required::<OpenCurlyBracesSymbol>::parse(&mut iter);
        let _close_curly =Required::<CloseCurlyBracesSymbol>::parse(&mut iter);

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

        let lexer = LexerIter::new(content);

        let (tokens, ..) = lexer.collect_all();
        let mut iter = TokensIter::new(&tokens);

        let _fn = Required::<FnKeyword>::parse(&mut iter);
        let _id = Required::<Id>::parse(&mut iter);
        let _open_paren = Required::<OpenParenthesisSymbol>::parse(&mut iter);
        let _close_paren = Required::<CloseParenthesisSymbol>::parse(&mut iter);

        assert!(_fn.is_ok());
        assert!(_id.is_ok());
        assert!(_open_paren.is_ok());
        assert!(
            matches!(
                _close_paren,
                Err(
                    ParseError::UnexpectedToken {
                        expected: TokenType::Symbol(SymbolType::CloseParenthesis),
                        found: (_, TokenType::Id)
                    }
                )
            )
        );
    }
}