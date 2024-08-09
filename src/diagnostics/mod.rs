use std::fmt::Display;

use owo_colors::AnsiColors;

mod span;

pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

pub(crate) struct Diagnostic {
    typ: DiagnosticType,
    msg: String,
    code_window: Option<CodeWindow>,
    sub_diagnostics: Vec<Diagnostic>,
}

enum DiagnosticType{
    Error,
    Warning,
    Help,
    Note,
}

pub(crate) struct CodeWindow {
    file_path: String,
    lines: Vec<LineCode>,
}

pub(crate) struct LineCode {
    /// The line index in the file
    index: usize,
    /// Markers to display
    markers: Vec<LineCodeMarker>,
}

pub(crate) struct LineCodeMarker{
    /// The symbol to repeat
    symbol: char,
    /// Color,
    color: AnsiColors,
    /// The start column index (characters count)
    col: usize,
    /// The length (characters count, or number of repeatations)
    len: usize,
    /// The label to display
    label: String,
}

impl Diagnostics {
    pub fn new() -> Self{
        Diagnostics { diagnostics: vec![] }
    }
}

impl Diagnostic {
    pub fn new() -> Self{
        todo!()
    }
}

impl CodeWindow {
    pub fn new() -> Self{
        todo!()
    }
}

impl LineCode {
    pub fn new() -> Self{
        todo!()
    }
}

impl LineCodeMarker {
    pub fn new() -> Self{
        todo!()
    }
}

impl Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut diagnostics_str = String::new();

        for diagnostic in &self.diagnostics {
            diagnostics_str.push_str(&format!("{}\n", diagnostic))
        }

        write!(f, "{}", diagnostics_str)
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for CodeWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for LineCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for LineCodeMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}