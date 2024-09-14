use std::{fmt::Display, path::Path};

use owo_colors::{OwoColorize, Style};
use span::{Span, SpanCursor};

pub mod span;

mod code_reporter;

use code_reporter::CodeReporter;

trait DiagnosticPrint<'a> {
    fn write(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        path: &'a str,
        file_lines: &'a [&'a str],
    ) -> std::fmt::Result;
    fn writeln(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        path: &'a str,
        file_lines: &'a [&'a str],
    ) -> std::fmt::Result {
        let _ = writeln!(f, "");
        self.write(f, path, file_lines)
    }
}

/// Represents the diagnostics for any compiler phase
pub struct PhaseDiagnostics<'a> {
    file_path: &'a Path,
    file_lines: &'a [&'a str],
    diagnostics: Vec<Diagnostic<'a>>,
}

impl<'a> PhaseDiagnostics<'a> {
    pub fn new(file_path: &'a Path, file_lines: &'a [&'a str]) -> Self {
        Self {
            file_path,
            file_lines,
            diagnostics: vec![],
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic<'a>) {
        self.diagnostics.push(diagnostic);
    }
}

impl<'a> Display for PhaseDiagnostics<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for d in &self.diagnostics {
            let _ = d.writeln(f, self.file_path.to_str().unwrap_or(""), self.file_lines);
        }
        Ok(())
    }
}

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

impl<'a> DiagnosticPrint<'a> for Diagnostic<'a> {
    fn write(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        path: &'a str,
        file_lines: &'a [&'a str],
    ) -> std::fmt::Result {
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
            let _ = code_window.write(f, path, file_lines);
        }

        for chained_diagnostic in &self.chained_diagnostics {
            let _ = chained_diagnostic.writeln(f, path, file_lines);
        }

        Ok(())
    }
}

pub struct CodeWindow<'a> {
    cursor: SpanCursor,
    code_reporter: CodeReporter<'a>,
}

impl<'a> CodeWindow<'a> {
    pub fn new(cursor: SpanCursor) -> CodeWindow<'a> {
        Self {
            cursor,
            code_reporter: CodeReporter::new(),
        }
    }

    pub fn mark_error(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter
            .mark(span, '^', Style::new().bold().red(), labels);
        self
    }

    pub fn mark_warning(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter
            .mark(span, '^', Style::new().bold().yellow(), labels);
        self
    }

    pub fn mark_help(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter
            .mark(span, '=', Style::new().bold().green(), labels);
        self
    }

    pub fn mark_note(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter
            .mark(span, '~', Style::new().bold().cyan(), labels);
        self
    }

    pub fn mark_secondary(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter
            .mark(span, '-', Style::new().bold().blue(), labels);
        self
    }

    pub fn mark_tertiary(&mut self, span: Span, labels: &'a [&'a str]) -> &mut Self {
        self.code_reporter
            .mark(span, '*', Style::new().bold().bright_magenta(), labels);
        self
    }
}

impl<'a> DiagnosticPrint<'a> for CodeWindow<'a> {
    fn write(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        path: &'a str,
        file_lines: &'a [&'a str],
    ) -> std::fmt::Result {
        let path = format!("{}:{}:{}", path, self.cursor.line + 1, self.cursor.col + 1);

        self.code_reporter.write(f, &path, file_lines)
    }
}
