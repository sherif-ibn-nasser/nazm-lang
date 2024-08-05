use std::fmt::Display;

mod ticket;

use ticket::DiagnosticTicket;

pub struct Diagnostics{
    tickets: Vec<DiagnosticTicket>,
}

impl Diagnostics {
    pub fn new() -> Self{
        Diagnostics { tickets: vec![] }
    }
}

impl Display for Diagnostics{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut diagnostics_str = String::new();

        for ticket in &self.tickets {
            diagnostics_str.push_str(&format!("{}\n", ticket))
        }

        write!(f, "{}", diagnostics_str)
    }
}