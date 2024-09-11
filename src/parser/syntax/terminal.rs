use super::*;
use crate::lexer::*;
use paste::paste;
use std::fmt::Debug;

mod private {
    pub trait Sealed {}
}

pub(crate) trait TerminalGuard: private::Sealed + Debug {}

#[derive(Debug)]
pub(crate) struct Terminal<T>
where
    T: TerminalGuard,
{
    pub(crate) span: Span,
    pub(crate) data: T,
}

impl<T: TerminalGuard> Spanned for Terminal<T> {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl<T: TerminalGuard> Check for Terminal<T> {
    fn is_broken(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub(crate) struct Id {
    val: String,
}

impl private::Sealed for Id {}

impl TerminalGuard for Id {}

impl NazmcParse for ParseResult<Terminal<Id>> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(Token {
                val,
                span,
                kind: TokenKind::Id,
            }) => {
                let ok = Ok(Terminal {
                    span: *span,
                    data: Id {
                        val: val.to_string(),
                    },
                });
                iter.next_non_space_or_comment();
                ok
            }
            Some(token) => Err(ParseErr {
                span: token.span,
                found_token: token.kind.clone(),
            }),
            None => ParseErr::eof(iter.peek_start_span()),
        }
    }
}

macro_rules! create_keyword_parser {
    ($keyword: ident) => {
        paste! {

            #[derive(Debug)]
            pub(crate) struct [<$keyword Keyword>];

            impl private::Sealed for [<$keyword Keyword>] {}

            impl TerminalGuard for [<$keyword Keyword>] {}

            impl NazmcParse for ParseResult<Terminal<[<$keyword Keyword>]>>{

                fn parse(iter: &mut TokensIter) -> Self {

                    match iter.recent() {
                        Some(Token { span, kind: TokenKind::Keyword(KeywordKind::$keyword), .. }) => {
                            let ok = Ok(Terminal {
                                span: *span,
                                data: [<$keyword Keyword>],
                            });
                            iter.next_non_space_or_comment();
                            ok
                        }
                        Some(token) => Err(ParseErr {
                            span: token.span,
                            found_token: token.kind.clone(),
                        }),
                        None => ParseErr::eof(iter.peek_start_span()),
                    }
                }
            }
        }
    };
}

macro_rules! create_symbol_parser {
    ($symbol: ident) => {
        paste! {

            #[derive(Debug)]
            pub(crate) struct [<$symbol Symbol>];

            impl private::Sealed for [<$symbol Symbol>] {}

            impl TerminalGuard for [<$symbol Symbol>] {}

            impl NazmcParse for ParseResult<Terminal<[<$symbol Symbol>]>>{

                fn parse(iter: &mut TokensIter) -> Self {

                    match iter.recent() {
                        Some(Token { span, kind: TokenKind::Symbol(SymbolKind::$symbol), .. }) => {
                            let ok = Ok(Terminal {
                                span: *span,
                                data: [<$symbol Symbol>],
                            });
                            iter.next_non_space_or_comment();
                            ok
                        }
                        Some(token) => Err(ParseErr {
                            span: token.span,
                            found_token: token.kind.clone(),
                        }),
                        None => ParseErr::eof(iter.peek_start_span()),
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
create_keyword_parser!(If);
create_keyword_parser!(Else);
create_keyword_parser!(When);
create_keyword_parser!(Loop);
create_keyword_parser!(While);
create_keyword_parser!(Do);
create_keyword_parser!(Break);
create_keyword_parser!(Continue);
create_keyword_parser!(Return);
create_keyword_parser!(Run);

create_symbol_parser!(Comma);
create_symbol_parser!(Semicolon);
create_symbol_parser!(QuestionMark);
create_symbol_parser!(OpenParenthesis);
create_symbol_parser!(CloseParenthesis);
create_symbol_parser!(OpenCurlyBrace);
create_symbol_parser!(CloseCurlyBrace);
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

#[derive(Debug)]
pub(crate) struct DoubleColonsSymbol;

#[derive(Debug)]
pub(crate) struct RArrow;

#[derive(Debug)]
/// The parse method is written by hand to avoid backtracking
pub(crate) enum BinOp {
    LOr,
    LAnd,
    EqualEqual,
    NotEqual,
    GE,
    GT,
    LE,
    LT,
    OpenOpenRange,
    CloseOpenRange,
    OpenCloseRange,
    CloseCloseRange,
    BOr,
    Xor,
    BAnd,
    Shr,
    Shl,
    Plus,
    Minus,
    Times,
    Div,
    Mod,
    Assign,
    PlusAssign,
    MinusAssign,
    TimesAssign,
    DivAssign,
    ModAssign,
    BAndAssign,
    BOrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
}

#[derive(Debug)]
/// The parse method is written by hand to avoid backtracking
///
/// Note that there is no unary plus operator
pub(crate) enum UnaryOp {
    Minus,
    LNot,
    BNot,
    Deref,
    Borrow,
    BorrowMut,
}

impl private::Sealed for DoubleColonsSymbol {}
impl private::Sealed for RArrow {}
impl private::Sealed for BinOp {}
impl private::Sealed for UnaryOp {}

impl TerminalGuard for DoubleColonsSymbol {}
impl TerminalGuard for RArrow {}
impl TerminalGuard for BinOp {}
impl TerminalGuard for UnaryOp {}

macro_rules! match_peek_symbols {
    ($iter:ident, $symbol0:ident, $symbol1:ident, $symbol2:ident) => {
        match_peek_symbols!($iter, 0, $symbol0)
            && match_peek_symbols!($iter, 1, $symbol1)
            && match_peek_symbols!($iter, 2, $symbol2)
    };
    ($iter:ident, $symbol0:ident, $symbol1:ident) => {
        match_peek_symbols!($iter, 0, $symbol0) && match_peek_symbols!($iter, 1, $symbol1)
    };
    ($iter:ident, $symbol:ident) => {
        match_peek_symbols!($iter, 0, $symbol)
    };
    ($iter:ident, $nth: literal, $symbol:ident) => {
        matches!(
            $iter.peek_nth($nth),
            Some(Token {
                kind: TokenKind::Symbol(SymbolKind::$symbol),
                ..
            })
        )
    };
}

impl NazmcParse for ParseResult<Terminal<DoubleColonsSymbol>> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    kind: TokenKind::Symbol(SymbolKind::Colon),
                    ..
                },
            ) if match_peek_symbols!(iter, Colon) => {
                let mut span = token.span;
                span.end.col += 1;
                iter.peek_idx += 1; // Eat next colon
                iter.next_non_space_or_comment();
                Ok(Terminal {
                    span,
                    data: DoubleColonsSymbol,
                })
            }
            Some(token) => Err(ParseErr {
                span: token.span,
                found_token: token.kind.clone(),
            }),
            None => ParseErr::eof(iter.peek_start_span()),
        }
    }
}

impl NazmcParse for ParseResult<Terminal<RArrow>> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    kind: TokenKind::Symbol(SymbolKind::Minus),
                    ..
                },
            ) if match_peek_symbols!(iter, CloseAngleBracketOrGreater) => {
                let mut span = token.span;
                span.end.col += 1;
                iter.peek_idx += 1; // Eat next '>'
                iter.next_non_space_or_comment();
                Ok(Terminal { span, data: RArrow })
            }
            Some(token) => Err(ParseErr {
                span: token.span,
                found_token: token.kind.clone(),
            }),
            None => ParseErr::eof(iter.peek_start_span()),
        }
    }
}

impl NazmcParse for ParseResult<Terminal<BinOp>> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    kind: TokenKind::Symbol(symbol_kind),
                    ..
                },
            ) => {
                let mut span = token.span;
                let (op_kind, peek_inc) = match symbol_kind {
                    SymbolKind::OpenAngleBracketOrLess
                        if match_peek_symbols!(iter, Dot, Dot, OpenAngleBracketOrLess) =>
                    {
                        (BinOp::OpenOpenRange, 3)
                    }
                    SymbolKind::OpenAngleBracketOrLess if match_peek_symbols!(iter, Dot, Dot) => {
                        (BinOp::OpenCloseRange, 2)
                    }
                    SymbolKind::OpenAngleBracketOrLess
                        if match_peek_symbols!(iter, OpenAngleBracketOrLess, Equal) =>
                    {
                        (BinOp::ShrAssign, 2)
                    }
                    SymbolKind::OpenAngleBracketOrLess
                        if match_peek_symbols!(iter, OpenAngleBracketOrLess) =>
                    {
                        (BinOp::Shr, 1)
                    }
                    SymbolKind::OpenAngleBracketOrLess if match_peek_symbols!(iter, Equal) => {
                        (BinOp::LE, 1)
                    }
                    SymbolKind::OpenAngleBracketOrLess => (BinOp::LT, 0),

                    SymbolKind::CloseAngleBracketOrGreater
                        if match_peek_symbols!(iter, CloseAngleBracketOrGreater, Equal) =>
                    {
                        (BinOp::ShlAssign, 2)
                    }
                    SymbolKind::CloseAngleBracketOrGreater
                        if match_peek_symbols!(iter, CloseAngleBracketOrGreater) =>
                    {
                        (BinOp::Shl, 1)
                    }
                    SymbolKind::CloseAngleBracketOrGreater if match_peek_symbols!(iter, Equal) => {
                        (BinOp::GE, 1)
                    }
                    SymbolKind::CloseAngleBracketOrGreater => (BinOp::GT, 0),

                    SymbolKind::Dot if match_peek_symbols!(iter, Dot, OpenAngleBracketOrLess) => {
                        (BinOp::CloseOpenRange, 2)
                    }
                    SymbolKind::Dot if match_peek_symbols!(iter, Dot) => {
                        (BinOp::CloseCloseRange, 1)
                    }

                    SymbolKind::Equal if match_peek_symbols!(iter, Equal) => (BinOp::EqualEqual, 1),
                    SymbolKind::Equal => (BinOp::Assign, 0),
                    SymbolKind::ExclamationMark if match_peek_symbols!(iter, Equal) => {
                        (BinOp::NotEqual, 1)
                    }

                    SymbolKind::BitOr if match_peek_symbols!(iter, BitOr) => (BinOp::LOr, 1),
                    SymbolKind::BitOr if match_peek_symbols!(iter, Equal) => (BinOp::BOrAssign, 1),
                    SymbolKind::BitOr => (BinOp::BOr, 0),

                    SymbolKind::Xor if match_peek_symbols!(iter, Equal) => (BinOp::XorAssign, 1),
                    SymbolKind::Xor => (BinOp::Xor, 0),

                    SymbolKind::BitAnd if match_peek_symbols!(iter, BitAnd) => (BinOp::LAnd, 1),
                    SymbolKind::BitAnd if match_peek_symbols!(iter, Equal) => {
                        (BinOp::BAndAssign, 1)
                    }
                    SymbolKind::BitAnd => (BinOp::BAnd, 0),

                    SymbolKind::Plus if match_peek_symbols!(iter, Equal) => (BinOp::PlusAssign, 1),
                    SymbolKind::Plus => (BinOp::Plus, 0),

                    SymbolKind::Minus if match_peek_symbols!(iter, Equal) => {
                        (BinOp::MinusAssign, 1)
                    }
                    SymbolKind::Minus => (BinOp::Minus, 0),

                    SymbolKind::Star if match_peek_symbols!(iter, Equal) => (BinOp::TimesAssign, 1),
                    SymbolKind::Star => (BinOp::Times, 0),

                    SymbolKind::Slash if match_peek_symbols!(iter, Equal) => (BinOp::DivAssign, 1),
                    SymbolKind::Slash => (BinOp::Div, 0),

                    SymbolKind::Modulo if match_peek_symbols!(iter, Equal) => (BinOp::ModAssign, 1),
                    SymbolKind::Modulo => (BinOp::Mod, 0),

                    _ => {
                        return Err(ParseErr {
                            span: token.span,
                            found_token: token.kind.clone(),
                        });
                    }
                };

                iter.peek_idx += peek_inc;
                span.end.col += peek_inc;
                iter.next_non_space_or_comment();

                Ok(Terminal {
                    span,
                    data: op_kind,
                })
            }
            Some(token) => Err(ParseErr {
                span: token.span,
                found_token: token.kind.clone(),
            }),
            None => ParseErr::eof(iter.peek_start_span()),
        }
    }
}

impl NazmcParse for ParseResult<Terminal<UnaryOp>> {
    fn parse(iter: &mut TokensIter) -> Self {
        match iter.recent() {
            Some(
                token @ Token {
                    val: _,
                    span,
                    kind: TokenKind::Symbol(symbol_kind),
                },
            ) => {
                let mut span = *span;

                let op_kind = match symbol_kind {
                    SymbolKind::Minus if !match_peek_symbols!(iter, Equal) => UnaryOp::Minus,
                    SymbolKind::ExclamationMark if !match_peek_symbols!(iter, Equal) => {
                        UnaryOp::LNot
                    }
                    SymbolKind::BitNot if !match_peek_symbols!(iter, Equal) => UnaryOp::BNot,
                    SymbolKind::Star if !match_peek_symbols!(iter, Equal) => UnaryOp::Deref,
                    SymbolKind::Hash => {
                        let peek_idx = iter.peek_idx;
                        if let Some(Token {
                            span: mut_keyword_span,
                            kind: TokenKind::Keyword(KeywordKind::Mut),
                            ..
                        }) = iter.next_non_space_or_comment()
                        {
                            span = span.merged_with(mut_keyword_span);
                            UnaryOp::Borrow
                        } else {
                            iter.peek_idx = peek_idx;
                            UnaryOp::BorrowMut
                        }
                    }
                    _ => {
                        return Err(ParseErr {
                            span: token.span,
                            found_token: token.kind.clone(),
                        });
                    }
                };
                iter.next_non_space_or_comment();
                Ok(Terminal {
                    span,
                    data: op_kind,
                })
            }
            Some(token) => Err(ParseErr {
                span: token.span,
                found_token: token.kind.clone(),
            }),
            None => ParseErr::eof(iter.peek_start_span()),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::LexerIter;

    use super::*;

    #[test]
    fn test() {
        let content = "دالة البداية(/* تعليق */){}";

        let lexer = LexerIter::new(content);
        let (tokens, ..) = lexer.collect_all();
        let mut iter = TokensIter::new(&tokens);
        iter.next(); // Initialize the value of recent

        let _fn = ParseResult::<Terminal<FnKeyword>>::parse(&mut iter);
        let _id = ParseResult::<Terminal<Id>>::parse(&mut iter);
        let _open_paren = ParseResult::<Terminal<OpenParenthesisSymbol>>::parse(&mut iter);
        let _close_paren = ParseResult::<Terminal<CloseParenthesisSymbol>>::parse(&mut iter);
        let _open_curly = ParseResult::<Terminal<OpenCurlyBraceSymbol>>::parse(&mut iter);
        let _close_curly = ParseResult::<Terminal<CloseCurlyBraceSymbol>>::parse(&mut iter);

        assert!(!_fn.unwrap().is_broken());
        assert!(!_id.unwrap().is_broken());
        assert!(!_open_paren.unwrap().is_broken());
        assert!(!_close_paren.unwrap().is_broken());
        assert!(!_open_curly.unwrap().is_broken());
        assert!(!_close_curly.unwrap().is_broken());
    }

    #[test]
    fn test_fail() {
        let content = "دالة البداية(عدد: ص8){}";

        let lexer = LexerIter::new(content);

        let (tokens, ..) = lexer.collect_all();
        let mut iter = TokensIter::new(&tokens);
        iter.next(); // Initialize the value of recent

        let _fn = ParseResult::<Terminal<FnKeyword>>::parse(&mut iter);
        let _id = ParseResult::<Terminal<Id>>::parse(&mut iter);
        let _open_paren = ParseResult::<Terminal<OpenParenthesisSymbol>>::parse(&mut iter);
        let _close_paren = ParseResult::<Terminal<CloseParenthesisSymbol>>::parse(&mut iter);

        assert!(!_fn.unwrap().is_broken());
        assert!(!_id.unwrap().is_broken());
        assert!(!_open_paren.unwrap().is_broken());
        assert!(matches!(
            _close_paren.unwrap_err(),
            ParseErr {
                found_token: TokenKind::Id,
                ..
            }
        ));
    }
}
