use std::{fmt::Display, path::Path, usize};

use owo_colors::{OwoColorize, Style};
use span::{Span, SpanCursor};

pub mod span;
pub mod errors;

mod code_reporter;

use code_reporter::CodeReporter;


pub struct FileDiagnostics<'a> {
    diagnostics: Vec<Diagnostic<'a>>,
    file_path: &'a Path,
}

struct Diagnostic<'a> {
    level: DiagnosticLevel,
    msg: &'a str,
    code_window: Option<CodeWindow<'a>>,
    chained_diagnostics: Vec<Diagnostic<'a>>,
}

impl<'a> Diagnostic<'a> {

    fn new(level: DiagnosticLevel, msg: &'a str) -> Self {
        Diagnostic { level: level, msg: msg, code_window: None, chained_diagnostics: vec![] }
    }
    
    fn set_code_window(&mut self, code_window: CodeWindow<'a>) {
        self.code_window = Some(code_window);
    }

    fn chain(&mut self, with: Diagnostic<'a>) -> &mut Self {
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
            DiagnosticLevel::ErrorWithCode(error_code) => write!(f, "{}{}{}{}", "خطأ".bold().red(), "[".bold() ,error_code.bold().red(), "]". bold()),
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

struct CodeWindow<'a> {
    file_path: &'a str,
    cursor: SpanCursor,
    code_reporter: CodeReporter<'a>,
}

impl<'a> CodeWindow<'a> {

    fn new(&mut self, file_path: &'a str, files_lines: &'a [&'a str], cursor: SpanCursor) -> Self {
        Self {
            file_path: file_path,
            cursor: cursor,
            code_reporter: CodeReporter::new(files_lines, Style::new().bold().blue())
        }
    }

    fn mark_error(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter.mark(span, '^', Style::new().bold().red(), labels);
        self
    }

    fn mark_warning(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter.mark(span, '^', Style::new().bold().yellow(), labels);
        self
    }

    fn mark_help(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter.mark(span, '=', Style::new().bold().green(), labels);
        self
    }

    fn mark_note(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter.mark(span, '~', Style::new().bold().cyan(), labels);
        self
    }

    fn mark_secondary(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter.mark(span, '-', Style::new().bold().blue(), labels);
        self
    }

    fn mark_tertiary(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter.mark(span, '*', Style::new().bold().bright_magenta(), labels);
        self
    }

}

impl<'a> Display for CodeWindow<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let _ = writeln!(
            f,
            "{}{} {}:{}:{}",
            "المسار".bold().blue(),
            ":".bold(),
            self.file_path.bold(),
            self.cursor.line.bold(),
            self.cursor.col.bold()
        );

        write!(f, "{}", self.code_reporter)
        
    }
}