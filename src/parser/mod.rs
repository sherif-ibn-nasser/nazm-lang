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

    fn report(
        &mut self,
        msg: String,
        span: Span,
        error_multiline_label: Vec<String>,
        secondary_labels: Vec<(Span, Vec<String>)>,
    ) {
        let mut code_window = CodeWindow::new(span.start);

        if !error_multiline_label.is_empty() {
            code_window.mark_error(span, error_multiline_label);
        }

        for (span, multiline_label) in secondary_labels {
            code_window.mark_secondary(span, multiline_label);
        }

        let diagnostic = Diagnostic::error(msg, Some(code_window));

        self.diagnostics.push(diagnostic);
    }

    fn report_expected(
        &mut self,
        expected: &str,
        err: &ParseErr,
        secondary_labels: Vec<(Span, Vec<String>)>,
    ) {
        let (found_token_span, found_token_val, error_multiline_label) =
            if err.found_token_index < self.tokens.len() {
                let token = &self.tokens[err.found_token_index];
                (token.span, token.val, vec!["رمز غير متوقع".to_string()])
            } else {
                let last_span = self.tokens[self.tokens.len() - 1].span;
                (
                    Span::len_after(&last_span, 1),
                    "نهاية الملف",
                    vec!["تم الوصول إلى نهاية الملف".to_string()],
                )
            };

        let msg = format!(
            "يُتوقع {}، ولكن تم العثور على `{}`",
            expected, found_token_val
        );

        self.report(
            msg,
            found_token_span,
            error_multiline_label,
            secondary_labels,
        );
    }

    fn span_of_err(&self, err: &ParseErr) -> Span {
        if err.found_token_index < self.tokens.len() {
            self.tokens[err.found_token_index].span
        } else {
            Span::after(&self.tokens[self.tokens.len() - 1].span)
        }
    }

    fn check_file_items(&mut self, items: &[ParseResult<FileItem>]) {
        let expected = "عنصر ملف (دالة أو تصنيف)";
        for item in items {
            let node = match item {
                Ok(node) => node,
                Err(err) => {
                    self.report_expected(expected, err, vec![]);
                    continue;
                }
            };

            let item = match node {
                FileItem::WithVisModifier(ItemWithVisibility { visibility, item }) => match item {
                    Ok(item) => item,
                    Err(_) => {
                        self.report(
                            "يُتوقع عنصر ملف (دالة أو تصنيف) بعد مُعامِل الوصول".to_string(),
                            visibility.span,
                            vec!["مُعامِل الوصول".to_string()],
                            vec![],
                        );
                        continue;
                    }
                },
                FileItem::WithoutModifier(item) => item,
            };

            match item {
                Item::Struct(s) => self.check_struct(s),
                Item::Fn(f) => self.check_fn(f),
            }
        }
    }

    fn check_struct(&mut self, s: &Struct) {
        let Struct {
            struct_keyword,
            name,
            kind,
        } = s;

        let missing_name = name.is_err();
        let missing_decl = kind.is_err();

        if missing_name && missing_decl {
            self.report(
                "لم يتم تعريف التصنيف".to_string(),
                struct_keyword.span,
                vec!["".to_string()],
                vec![],
            );
            return;
        }

        if missing_name {
            self.report(
                "يجب إعطاء اسم للتصنيف".to_string(),
                struct_keyword.span,
                vec!["".to_string()],
                vec![],
            );
        }
    }

    fn check_fn(&mut self, f: &Fn) {
        let Fn {
            fn_keyword,
            name,
            params_decl,
            body,
        } = f;

        let missing_name = name.is_err();
        let missing_params = params_decl.is_err();
        let missing_body = body.is_err();

        if missing_name && missing_params && missing_body {
            self.report(
                "لم يتم تعريف الدالة".to_string(),
                fn_keyword.span,
                vec!["".to_string()],
                vec![],
            );
            return;
        }

        let sec_labels_span = if let Ok(name) = name {
            fn_keyword.span.merged_with(&name.span)
        } else {
            fn_keyword.span
        };

        let sec_labels_span_start_line = sec_labels_span.start.line;

        let sec_labels = vec![(sec_labels_span, vec!["عند هذه الدالة".to_string()])];

        if let Err(err) = name {
            let err_span = self.span_of_err(err);
            let sec_labels = if err_span.start.line == sec_labels_span_start_line {
                vec![]
            } else {
                sec_labels.clone()
            };
            self.report_expected("اسم للدالة", err, sec_labels);
        }

        match params_decl {
            Ok(params) => {}
            Err(err) if !missing_name => {
                let err_span = self.span_of_err(err);
                let sec_labels = if err_span.start.line == sec_labels_span_start_line {
                    vec![]
                } else {
                    sec_labels.clone()
                };
                self.report_expected("مُعامِلات الدالة", err, sec_labels);
            }
            _ => {}
        };

        match body {
            Ok(body) => {}
            Err(err) if !missing_params => {
                let err_span = self.span_of_err(err);
                let sec_labels = if err_span.start.line == sec_labels_span_start_line {
                    vec![]
                } else {
                    sec_labels.clone()
                };
                self.report_expected("محتوى للدالة", err, sec_labels);
            }
            _ => {}
        }
    }
}
