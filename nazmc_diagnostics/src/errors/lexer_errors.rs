
use crate::*;
use crate::span::*;

impl<'a> PhaseDiagnostics<'a> {

    pub fn report_unknown_token(&mut self, cursor: SpanCursor){

        let mut err = Diagnostic::new(
            DiagnosticLevel::Error,
            "رمز غير صالح"
        );
        
        let mut code_window = self.new_code_window(cursor);

        code_window.mark_error(
            Span {
                start: cursor,
                end: SpanCursor { line: cursor.line, col: cursor.col + 1 }
            },
            &[], // No labels
        );

        err.set_code_window(code_window);

        self.diagnostics.push(err)
    }
}