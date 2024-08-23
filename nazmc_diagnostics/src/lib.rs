use std::{fmt::Display, path::Path};

use itertools::Itertools;
use owo_colors::{OwoColorize, Style};
use span::{Span, SpanCursor};

pub mod span;
pub mod errors;

mod code_reporter;

use code_reporter::{BuiltCodeReporter, CodeReporter, UnderConstructionCodeReporter};

/// Represents the diagnostics for any compiler phase
pub struct PhaseDiagnostics<'a> {
    diagnostics: Vec<Diagnostic<'a, UnderConstructionCodeWindow<'a>>>,
}

impl<'a> PhaseDiagnostics<'a> {

    pub fn new() -> Self {
        Self { diagnostics: vec![], }
    }

    #[inline]
    fn new_code_window(&self, cursor: SpanCursor) -> UnderConstructionCodeWindow<'a> {
        UnderConstructionCodeWindow {
            cursor: cursor,
            file_path: NoPath,
            code_reporter: CodeReporter::new()
        }
    }

    fn build(self, file_path: &'a Path, file_lines: &'a [&'a str]) -> Vec<Diagnostic<'a, BuiltCodeWindow<'a>>> {
        self.diagnostics.into_iter().map(
            |d| d.build(file_path, file_lines, Style::new().bold().blue())
        ).collect_vec()
    }
}

type UnderConstructionCodeWindow<'a> = CodeWindow<'a, NoPath, UnderConstructionCodeReporter>;

type BuiltCodeWindow<'a> = CodeWindow<'a, &'a Path, BuiltCodeReporter<'a>>;

struct Diagnostic<'a, CodeWindowBuildingState> {
    level: DiagnosticLevel,
    msg: &'a str,
    code_window: Option<CodeWindowBuildingState>,
    chained_diagnostics: Vec<Diagnostic<'a, CodeWindowBuildingState>>,
}

impl<'a> Diagnostic<'a, UnderConstructionCodeWindow<'a>> {

    fn new(level: DiagnosticLevel, msg: &'a str) -> Self {
        Diagnostic { level: level, msg: msg, code_window: None, chained_diagnostics: vec![] }
    }
    
    fn set_code_window(&mut self, code_window: UnderConstructionCodeWindow<'a>) {
        self.code_window = Some(code_window);
    }

    fn chain(&mut self, with: Diagnostic<'a, UnderConstructionCodeWindow<'a>>) -> &mut Self {
        self.chained_diagnostics.push(with);
        self
    }

    fn build(self, file_path: &'a Path, file_lines: &'a [&'a str], line_nums_style: Style) -> Diagnostic<'a, BuiltCodeWindow<'a>> {
        Diagnostic {
            level: self.level,
            msg: self.msg,
            code_window: self.code_window.map(
                |code_window| code_window.build(file_path, file_lines, line_nums_style)
            ),
            chained_diagnostics: self.chained_diagnostics.into_iter()
                .map(
                    |d| d.build(file_path, file_lines, line_nums_style)
                ).collect_vec()
        }
    }
}

enum DiagnosticLevel {
    Error,
    ErrorWithCode(usize),
    Warning,
    Help,
    Note,
}

impl<'a> Display for Diagnostic<'a, BuiltCodeWindow<'a>> {
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

struct NoPath;

struct CodeWindow<'a, PathState, CodeReporterState> {
    cursor: SpanCursor,
    file_path: PathState,
    code_reporter: CodeReporter<'a, CodeReporterState>,
}

impl<'a> CodeWindow<'a, NoPath, UnderConstructionCodeReporter> {

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

    fn build(self, file_path: &'a Path, file_lines: &'a [&'a str], line_nums_style: Style) -> CodeWindow<'a, &'a Path, BuiltCodeReporter<'a>> {
        CodeWindow { 
            cursor: self.cursor,
            file_path: file_path,
            code_reporter: self.code_reporter.build(file_lines, line_nums_style)
        }
    }

}

impl<'a> Display for CodeWindow<'a, &'a Path, BuiltCodeReporter<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let _ = writeln!(
            f,
            "{}{} {}:{}:{}",
            "المسار".bold().blue(),
            ":".bold(),
            self.file_path.display().bold(),
            self.cursor.line.bold(),
            self.cursor.col.bold()
        );

        write!(f, "{}", self.code_reporter)
        
    }
}