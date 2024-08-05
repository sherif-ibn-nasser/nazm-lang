mod span;

use std::{ops::Range, fmt::Display};

use span::DiagnosticSpan;

pub(crate) struct DiagnosticTicket{
    typ: DiagnosticTicketType,
    msg: String,
    file_path: String,
    lines_range: Range<usize>,
    spans: Vec<DiagnosticSpan>,
}

enum DiagnosticTicketType{
    Error,
    Warning,
    Note,
    Help,
}

impl Display for DiagnosticTicket{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
