mod cli;
mod lexer;
mod parser;

use lexer::*;
use owo_colors::{OwoColorize, XtermColors};
use parser::{syntax::File, tokens_iter::TokensIter, NazmcParse, ParseResult};
use std::{
    io::{self, Write},
    process::Command,
};

fn main() {
    let (file_path, file_content) = cli::read_file();

    let (tokens, file_lines, lexer_diagnostics) = LexerIter::new(&file_content).collect_all();

    let mut tokens_iter = TokensIter::new(&tokens);

    tokens_iter.next(); // To init recent()

    let result = ParseResult::<File>::parse(&mut tokens_iter);

    println!("{:#?}", result);

    // RTL printing
    // let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
    // io::stdout()
    //     .write_all(&output.stdout[1..output.stdout.len() - 1])
    //     .unwrap();

    // let mut bad_tokens = vec![];

    // for Token { span, val, kind } in tokens {
    //     let color = match kind {
    //         TokenKind::LineComment | TokenKind::DelimitedComment => XtermColors::BrightTurquoise,
    //         TokenKind::Symbol(_) => XtermColors::UserBrightYellow,
    //         TokenKind::Id => XtermColors::LightAnakiwaBlue,
    //         TokenKind::Keyword(_) | TokenKind::Literal(LiteralKind::Bool(_)) => {
    //             XtermColors::FlushOrange
    //         }
    //         TokenKind::Literal(LiteralKind::Str(_) | LiteralKind::Char(_)) => {
    //             XtermColors::PinkSalmon
    //         }
    //         TokenKind::Literal(_) => XtermColors::ChelseaCucumber,
    //         _ => XtermColors::UserWhite,
    //     };

    //     let mut val = format!("{}", val.color(color));

    //     if matches!(kind, TokenKind::Keyword(_) | TokenKind::Symbol(_)) {
    //         val = format!("{}", val.bold());
    //     }

    //     print!("{}", val);

    //     if let TokenKind::Bad(errs) = kind {
    //         bad_tokens.push((val, span, errs));
    //     }
    // }

    // if bad_tokens.is_empty() {
    //     return;
    // }

    // println!("-----------------------------");

    // for (_0, _1, _2) in bad_tokens {
    //     println!("{}, {:?}, {:?}", _0, _1, _2)
    // }
}
