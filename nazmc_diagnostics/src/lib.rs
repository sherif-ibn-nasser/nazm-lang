use owo_colors::OwoColorize;
use std::fmt::Display;
mod code_window;
pub mod span;
pub use code_window::CodeWindow;

pub struct Diagnostic<'a> {
    level: DiagnosticLevel,
    msg: String,
    code_window: Option<CodeWindow<'a>>,
    chained_diagnostics: Vec<Diagnostic<'a>>,
}

impl<'a> Diagnostic<'a> {
    pub fn error(msg: String, code_window: Option<CodeWindow<'a>>) -> Self {
        Self::new(DiagnosticLevel::Error, msg, code_window)
    }

    pub fn help(msg: String, code_window: Option<CodeWindow<'a>>) -> Self {
        Self::new(DiagnosticLevel::Help, msg, code_window)
    }

    #[inline]
    fn new(level: DiagnosticLevel, msg: String, code_window: Option<CodeWindow<'a>>) -> Self {
        Diagnostic {
            level: level,
            msg: msg,
            code_window: code_window,
            chained_diagnostics: vec![],
        }
    }

    pub fn chain(&mut self, with: Diagnostic<'a>) -> &mut Self {
        self.chained_diagnostics.push(with);
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

        let _ = writeln!(f, "{} {}", ":".bold(), self.msg.bold());

        if let Some(code_window) = &self.code_window {
            let _ = write!(f, "{}", code_window);
        }

        for chained_diagnostic in &self.chained_diagnostics {
            let _ = writeln!(f, "{}", chained_diagnostic);
        }

        Ok(())
    }
}
