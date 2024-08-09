use owo_colors::AnsiColors;

pub(crate) struct DiagnosticSpan{
    line: usize,
    col: usize,
    length: usize,
    msg: String,
    color: AnsiColors,
}