use std::path::Path;

use nazmc_diagnostics::{span::SpanCursor, CodeWindow, Diagnostic};
use syntax::File;

pub(crate) mod parse_methods;
pub(crate) mod syntax;
pub(crate) mod tokens_iter;

pub(crate) use crate::LexerIter;
use crate::Token;
pub(crate) use nazmc_diagnostics::{span::Span, PhaseDiagnostics};
pub(crate) use nazmc_parse_derive::*;
pub(crate) use parse_methods::*;
pub(crate) use syntax::*;
pub(crate) use tokens_iter::TokensIter;

pub struct ParseCtx<'a> {
    file_path: &'a Path,
    file_content: &'a str,
}

impl<'a> ParseCtx<'a> {
    pub fn new(file_path: &'a Path, file_content: &'a str) -> Self {
        Self {
            file_path,
            file_content,
        }
    }

    pub fn parse(&mut self) {
        let (tokens, file_lines) = LexerIter::new(self.file_content).collect_all();

        let mut tokens_iter = TokensIter::new(&tokens);

        tokens_iter.next_non_space_or_comment(); // To init recent()

        let ZeroOrMany { items, terminator } = ParseResult::<File>::parse(&mut tokens_iter)
            .unwrap()
            .content;

        let mut reporter = ParseErrorsReporter::new(self.file_path, &file_lines, &tokens);
        reporter.check_file_items(&items);

        println!("{}", reporter.diagnostics);
    }
}

struct ParseErrorsReporter<'a> {
    tokens: &'a [Token<'a>],
    diagnostics: PhaseDiagnostics<'a>,
}

impl<'a> ParseErrorsReporter<'a> {
    fn new(file_path: &'a Path, file_lines: &'a [&'a str], tokens: &'a [Token<'a>]) -> Self {
        Self {
            tokens,
            diagnostics: PhaseDiagnostics::new(file_path, file_lines),
        }
    }

    fn report(&mut self, expected: &str, err: &ParseErr) {
        let (found_token_span, found_token_val) = if err.found_token_index < self.tokens.len() {
            let token = &self.tokens[err.found_token_index];
            (token.span, token.val)
        } else {
            let last_span = self.tokens[self.tokens.len() - 1].span;

            (
                Span {
                    start: last_span.end,
                    end: SpanCursor {
                        line: last_span.end.line,
                        col: last_span.end.col + 1,
                    },
                },
                "نهاية الملف",
            )
        };

        let mut code_window = CodeWindow::new(found_token_span.start);

        let msg = format!(
            "يُتوقع {}، ولكن تم العثور على `{}`",
            expected, found_token_val
        );

        code_window.mark_error(found_token_span, &["رمز غير متوقع"]);

        let diagnostic = Diagnostic::error(msg, Some(code_window));

        self.diagnostics.push(diagnostic);
    }

    fn check_file_items(&mut self, items: &[ParseResult<FileItem>]) {
        for item in items {
            if let Err(err) = item {
                self.report("عنصر في الملف {دالة أو تصنيف}", err);
            }
        }
    }
}
