mod cli;

use nazmc_display_table::DisplayTable;
use nazmc_parser::ParseCtx;
use owo_colors::{OwoColorize, XtermColors};
use std::{
    io::{self, Write},
    process::Command,
};

fn main() {
    let (file_path, file_content) = cli::read_file();

    // RTL printing
    let output = Command::new("printf").arg(r#""\e[2 k""#).output().unwrap();
    io::stdout()
        .write_all(&output.stdout[1..output.stdout.len() - 1])
        .unwrap();

    let mut display_table = DisplayTable::new();

    ParseCtx::new(&file_path, &file_content).parse(&mut display_table);

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
    // }
}
