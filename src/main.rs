mod diagnostics;
mod cli;
mod lexer;
mod span;

use std::{cell::RefCell, collections::HashMap};
use diagnostics::Diagnostics;
use lexer::*;
use owo_colors::{AnsiColors, OwoColorize};

fn main() {

    let mut files: HashMap<String, Vec<String>> = HashMap::new();

    let diagnostics = RefCell::new(Diagnostics::new());

    let file_content = cli::read_file();

    let lexer = LexerIter::new(&file_content);

    let mut bad_tokens = vec![];

    for (span, token_typ, val) in lexer {
        let color = match token_typ {
            TokenType::LineComment | TokenType::DelimitedComment => AnsiColors::BrightGreen,
            TokenType::Symbol(_) => AnsiColors::BrightYellow,
            TokenType::Id => AnsiColors::BrightWhite,
            TokenType::Keyword(_) | TokenType::Literal(LiteralTokenType::Bool(_))
            => AnsiColors::BrightBlue,
            TokenType::Literal(
                LiteralTokenType::Str(_) | LiteralTokenType::Char(_)
            ) => AnsiColors::BrightMagenta,
            TokenType::Literal(_) => AnsiColors::BrightCyan,
            _ => AnsiColors::White,
        };

        let mut val = format!("{}", val.color(color));

        if matches!(token_typ, TokenType::Keyword(_) | TokenType::Symbol(_)) {
            val = format!("{}", val.bold());
        }
        
        print!("{}", val);

        if let TokenType::Bad(errs) = token_typ {
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