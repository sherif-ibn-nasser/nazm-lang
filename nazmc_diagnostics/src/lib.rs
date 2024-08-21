use std::{fmt::Display, path::Path, usize};

use const_colors::{bold, cyan, end, green, magenta, red, yellow};
use nazmc_diagnostics_macros::nazmc_error_code;

mod error_codes;

pub struct FileDiagnostics<'a> {
    diagnostics: Vec<String>,
    file_path: &'a Path,
}

#[derive(Default)]
struct DiagnosticBuilder<Level: DiagnosticLevel, M = NoMsg, P = NoPath, CW = NoCodeWindow, CD = NoChainedDiagnostics> {
    level: Level,
    msg: M,
    path: P,
    code_window: CW,
    chained_diagnostic: CD,
}

pub trait DiagnosticLevel {
    const NAME: &'static str;
}

/* Diagnostic builder levels states */

pub trait ErrorLevel : Default {
    const CODE: &'static str;
}

impl<E> DiagnosticLevel for E where E: ErrorLevel {
    const NAME: &'static str = E::CODE;
}

#[derive(Default)]
/// Default error diagnostic level (no codes)

struct Error;

impl DiagnosticLevel for Error {
    const NAME: &'static str = "";
}

// impl DiagnosticLevel for Error {
//     const NAME: &'static str = concat!(red!(), bold!(), "خطأ", end!());
// }

#[derive(Default)]
struct Warning;

impl DiagnosticLevel for Warning {
    const NAME: &'static str = concat!(yellow!(), bold!(), "تنبيه", end!());
}

#[derive(Default)]
struct Help;

impl DiagnosticLevel for Help {
    const NAME: &'static str = concat!(green!(), bold!(), "مساعدة", end!());
}

#[derive(Default)]
struct Note;

impl DiagnosticLevel for Note {
    const NAME: &'static str = concat!(magenta!(), bold!(), "ملحوظة", end!());
}

/* Diagnostic builder msg states */

#[derive(Default)]
struct NoMsg;

type Msg<'a> = &'a str;

/* Diagnostic builder path states */

#[derive(Default)]
struct NoPath;

struct WithPath<'a>(&'a Path);

/* Diagnostic builder code window states */

#[derive(Default)]
struct NoCodeWindow;

#[derive(Default)]
struct CodeWindow;

/* Diagnostic builder chained diagnostic states */

#[derive(Default)]
struct NoChainedDiagnostics;

#[derive(Default)]
struct ChainedDiagnostics(Vec<String>);

impl <L: DiagnosticLevel, M, P, CW, CD> DiagnosticBuilder<L, M, P, CW, CD> {
    
    fn error() -> DiagnosticBuilder<Error> {
        DiagnosticBuilder {
            level: Error,
            ..Default::default()
        }
    }

    fn error_with_code<E: ErrorLevel>(error_code: E) -> DiagnosticBuilder<E> {
        DiagnosticBuilder {
            level: error_code,
            ..Default::default()
        }
    }

    fn warning() -> DiagnosticBuilder<Warning> {
        DiagnosticBuilder {
            level: Warning,
            ..Default::default()
        }
    }

    fn help() -> DiagnosticBuilder<Help> {
        DiagnosticBuilder {
            level: Help,
            ..Default::default()
        }
    }

    fn note() -> DiagnosticBuilder<Note> {
        DiagnosticBuilder {
            level: Note,
            ..Default::default()
        }
    }

}

// impl <L, P, CW, CD> DiagnosticBuilder<L, NoMsg, P, CW, CD> {
//     fn msg(self, msg: Msg) -> DiagnosticBuilder<L, Msg, P, CW, CD> {
//         DiagnosticBuilder::<L, Msg, P, CW, CD> {
//             level: self.level,
//             msg: msg,
//             path: self.path,
//             code_window: self.code_window,
//             chained_diagnostic: self.chained_diagnostic,
//         }
//     }
// }


// impl<'a> Display for FileDiagnostics<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut diagnostics_str = String::new();

//         for diagnostic in &self.diagnostics {
//             diagnostics_str.push_str(&format!("{}\n", diagnostic))
//         }

//         write!(f, "{}", diagnostics_str)
//     }
// }