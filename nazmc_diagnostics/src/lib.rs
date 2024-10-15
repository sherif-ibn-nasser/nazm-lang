use owo_colors::OwoColorize;
use std::fmt::Display;
mod code_window;
pub mod span;
pub use code_window::CodeWindow;

pub fn eprint_diagnostics(diagnostics: Vec<Diagnostic>) {
    let last_idx = diagnostics.len();
    for (i, d) in diagnostics.iter().enumerate() {
        eprintln!("{}", d);
        if i != last_idx {
            eprintln!();
        }
    }
}

pub struct Diagnostic<'a> {
    level: DiagnosticLevel,
    msg: String,
    code_windows: Vec<CodeWindow<'a>>,
    chained_diagnostics: Vec<Diagnostic<'a>>,
}

impl<'a> Diagnostic<'a> {
    pub fn error(msg: String, code_windows: Vec<CodeWindow<'a>>) -> Self {
        Self::new(DiagnosticLevel::Error, msg, code_windows)
    }

    pub fn help(msg: String, code_windows: Vec<CodeWindow<'a>>) -> Self {
        Self::new(DiagnosticLevel::Help, msg, code_windows)
    }

    #[inline]
    fn new(level: DiagnosticLevel, msg: String, code_windows: Vec<CodeWindow<'a>>) -> Self {
        Diagnostic {
            level,
            msg,
            code_windows,
            chained_diagnostics: vec![],
        }
    }

    pub fn chain(&mut self, with: Diagnostic<'a>) -> &mut Self {
        self.chained_diagnostics.push(with);
        self
    }

    pub fn push_code_window(&mut self, code_window: CodeWindow<'a>) -> &mut Self {
        self.code_windows.push(code_window);
        self
    }
}

enum DiagnosticLevel {
    Error,
    ErrorWithCode(usize),
    Warning,
    Help,
    Note,
}

impl<'a> Display for Diagnostic<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = match self.level {
            DiagnosticLevel::Error => write!(f, "{}", "خطأ".bold().red()),
            DiagnosticLevel::ErrorWithCode(error_code) => write!(
                f,
                "{}{}{}{}",
                "خطأ".bold().red(),
                "[".bold(),
                error_code.bold().red(),
                "]".bold()
            ),
            DiagnosticLevel::Warning => write!(f, "{}", "خطأ".bold().red()),
            DiagnosticLevel::Help => write!(f, "{}", "مساعدة".bold().green()),
            DiagnosticLevel::Note => write!(f, "{}", "ملحوظة".bold().cyan()),
        };

        let _ = write!(f, "{} {}", ":".bold(), self.msg.bold());

        for code_window in &self.code_windows {
            let _ = write!(f, "\n{}", code_window);
        }

        for chained_diagnostic in &self.chained_diagnostics {
            let _ = write!(f, "\n{}", chained_diagnostic);
        }

        Ok(())
    }
}
