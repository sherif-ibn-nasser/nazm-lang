use std::path::Path;

use crate::*;
use crate::span::*;


struct LexerDiagnostics<'a> {
    file_path: &'a Path,
    file_lines: &'a [&'a str],
}

impl<'a> LexerDiagnostics<'a> {


    pub fn report_unknown_token(
        &self,
        token_val: &str,
        span: Span,
    ){
        DiagnosticBuilder::
            error()
            .path(&self.file_path)
            .msg(&format!("الرمز `{}` غير معروف", token_val))
            
        ;
    }
}