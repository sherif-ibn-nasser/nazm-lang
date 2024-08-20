mod diagnostics;
mod cli;
mod lexer;
mod span;
mod code_reporter;

use std::{cell::RefCell, collections::HashMap, io::{self, Write}, process::Command};
use diagnostics::Diagnostics;
use lexer::*;
use owo_colors::{OwoColorize, XtermColors};

fn main() {

    let mut files: HashMap<String, Vec<String>> = HashMap::new();

    let diagnostics = RefCell::new(Diagnostics::new());

    let file_content = cli::read_file();

    let lexer = LexerIter::new(&file_content);


    // RTL printing
    let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
    io::stdout().write_all(&output.stdout[1..output.stdout.len()-1]).unwrap();

    let mut bad_tokens = vec![];

    for Token { span, val, typ }  in lexer {

        let color = match typ {
            TokenType::LineComment | TokenType::DelimitedComment =>XtermColors::BrightTurquoise,
            TokenType::Symbol(_) => XtermColors::UserBrightYellow,
            TokenType::Id => XtermColors::LightAnakiwaBlue,
            TokenType::Keyword(_) | TokenType::Literal(LiteralTokenType::Bool(_))
            => XtermColors::FlushOrange,
            TokenType::Literal(
                LiteralTokenType::Str(_) | LiteralTokenType::Char(_)
            ) => XtermColors::PinkSalmon,
            TokenType::Literal(_) => XtermColors::ChelseaCucumber,
            _ => XtermColors::UserWhite,
        };

        let mut val = format!("{}", val.color(color));

        if matches!(typ, TokenType::Keyword(_) | TokenType::Symbol(_)) {
            val = format!("{}", val.bold());
        }
        
        print!("{}", val);

        if let TokenType::Bad(errs) = typ {
            bad_tokens.push((val, span, errs));
        }
    }

    if bad_tokens.is_empty() {
        return;
    }

    println!("-----------------------------");

    for (_0, _1, _2) in bad_tokens {
        println!("{}, {:?}, {:?}", _0, _1, _2)
    }
}