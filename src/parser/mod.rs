use std::path::Path;

use bpaf::params;
use nazmc_diagnostics::{span::SpanCursor, CodeWindow, Diagnostic};
use syntax::File;

pub(crate) mod parse_methods;
pub(crate) mod syntax;
pub(crate) mod tokens_iter;

pub(crate) use crate::LexerIter;
use crate::{SymbolKind, Token, TokenKind};
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
        primary_label: String,
        secondary_labels: Vec<(Span, Vec<String>)>,
    ) {
        let mut code_window = CodeWindow::new(span.start);

        code_window.mark_error(span, vec![primary_label]);

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
        let (found_token_span, found_token_val, primary_label) =
            if err.found_token_index < self.tokens.len() {
                let token = &self.tokens[err.found_token_index];
                (token.span, token.val, "رمز غير متوقع".to_string())
            } else {
                let last_span = self.tokens[self.tokens.len() - 1].span;
                (
                    Span::len_after(&last_span, 1),
                    "نهاية الملف",
                    "تم الوصول إلى نهاية الملف".to_string(),
                )
            };

        let msg = format!(
            "يُتوقع {}، ولكن تم العثور على `{}`",
            expected, found_token_val
        );

        self.report(msg, found_token_span, primary_label, secondary_labels);
    }

    fn report_expected_comma_or_item(
        &mut self,
        expected: &str,
        err: &ParseErr,
        secondary_labels: Vec<(Span, Vec<String>)>,
    ) {
        // Here we try to find if the error was occured as the comma is not found (so display that comma is expected)
        // or the comma is found but the the item after it is missing (so display that item is expected)
        let comma_err = 'label: {
            if err.found_token_index < self.tokens.len() {
                if let TokenKind::Symbol(SymbolKind::Comma) =
                    self.tokens[err.found_token_index].kind
                {
                    let mut i = err.found_token_index + 1;
                    while i < self.tokens.len() {
                        match &self.tokens[i].kind {
                            TokenKind::EOL
                            | TokenKind::DelimitedComment
                            | TokenKind::LineComment
                            | TokenKind::Space => i += 1,
                            _ => break,
                        }
                    }

                    break 'label ParseErr {
                        found_token_index: i,
                    };
                }
                err.clone()
            } else {
                err.clone()
            }
        };

        let expected = if comma_err.found_token_index > err.found_token_index {
            expected
        } else {
            "يُتوقع فاصلة `،`"
        };

        self.report_expected(expected, &comma_err, secondary_labels);
    }

    fn report_unclosed_delimiter(&mut self, open_delim_span: Span) {
        self.report(
            "لم يتم إغلاق القوس".to_string(),
            open_delim_span,
            "يجب إغلاق هذا القوس".to_string(),
            vec![],
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
                            "مُعامِل الوصول".to_string(),
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
                "".to_string(),
                vec![],
            );
            return;
        }

        if missing_name {
            self.report(
                "يجب إعطاء اسم للتصنيف".to_string(),
                struct_keyword.span,
                "".to_string(),
                vec![],
            );
        }
    }

    fn check_fn(&mut self, f: &Fn) {
        let Fn {
            fn_keyword,
            name,
            params_decl,
            return_type,
            body,
        } = f;

        let missing_name = name.is_err();
        let missing_params = params_decl.is_err();
        let no_return_type = return_type.is_none();
        let missing_body = body.is_err();

        if missing_name && missing_params && no_return_type && missing_body {
            self.report(
                "لم يتم تعريف الدالة".to_string(),
                fn_keyword.span,
                "".to_string(),
                vec![],
            );
            return;
        }

        if missing_name {
            self.report(
                "يجب إعطاء اسم للدالة".to_string(),
                fn_keyword.span,
                "".to_string(),
                vec![],
            );
        }

        match params_decl {
            Ok(params) => {
                let FnParams {
                    open_delim,
                    items,
                    close_delim,
                } = params;

                if close_delim.is_err() {
                    self.report_unclosed_delimiter(open_delim.span);
                }

                if let Some(PunctuatedFnParam {
                    first_item,
                    rest_items,
                    trailing_comma: _,
                }) = items
                {
                    match first_item {
                        Ok(param) => match &param.typ {
                            Ok(node) => self.check_type_result(&node.typ),
                            Err(err) => self.report_expected("نوع لمُعامِل الدالة", err, vec![]),
                        },
                        Err(err) => self.report_expected("مُعامِل دالة", err, vec![]),
                    }

                    for param in rest_items {
                        match param {
                            Ok(node) => match &node.item.typ {
                                Ok(node) => self.check_type_result(&node.typ),
                                Err(err) => self.report_expected("نوع لمُعامِل الدالة", err, vec![]),
                            },
                            Err(err) => {
                                self.report_expected_comma_or_item("مُعامِل دالة", err, vec![])
                            }
                        }
                    }
                }
            }
            Err(err) if !missing_name => {
                self.report_expected("مُعامِلات الدالة", err, vec![]);
            }
            _ => {}
        };

        if let Some(node) = return_type {
            self.check_type_result(&node.typ);
        }

        match body {
            Ok(body) => self.check_non_lambda_expr(body),
            Err(err) if !missing_params || !no_return_type => {
                self.report_expected("محتوى الدالة", err, vec![]);
            }
            _ => {}
        }
    }

    fn check_type_result(&mut self, typ: &ParseResult<Type>) {
        match typ {
            Ok(typ) => self.check_type(typ),
            Err(err) => self.report_expected("نوع", err, vec![]),
        }
    }

    fn check_type(&mut self, typ: &Type) {
        match typ {
            Type::Path(simple_path) => self.check_simple_path(simple_path),
            Type::Ptr(ptr_type) => self.check_type_result(&ptr_type.typ),
            Type::Ref(ref_type) => self.check_type_result(&ref_type.typ),
            Type::Slice(slice_type) => {
                self.check_type_result(&slice_type.typ);
                if let Some(ArraySizeExpr { semicolon: _, expr }) = &slice_type.array_size {
                    self.check_expr_result(expr);
                }
                if slice_type.close_bracket.is_err() {
                    self.report_unclosed_delimiter(slice_type.open_bracket.span);
                }
            }
            Type::Tuple(tuple_type) => {
                if let Some(PunctuatedType {
                    first_item,
                    rest_items,
                    trailing_comma: _,
                }) = &tuple_type.items
                {
                    self.check_type_result(first_item);

                    for param in rest_items {
                        match param {
                            Ok(node) => self.check_type(&node.item),
                            Err(err) => self.report_expected_comma_or_item("نوع", err, vec![]),
                        }
                    }
                }
                if tuple_type.close_delim.is_err() {
                    self.report_unclosed_delimiter(tuple_type.open_delim.span);
                }
            }
        }
    }

    fn check_simple_path(&mut self, simple_path: &SimplePath) {
        for SimpleInnerPath {
            double_colons: _,
            inner,
        } in &simple_path.inners
        {
            if let Err(err) = inner {
                self.report_expected("مسار", err, vec![]);
            }
        }
    }

    fn check_binding(&mut self, binding: &Binding) {
        self.check_binding_kind(&binding.kind);
        if let Some(ColonWithType { colon: _, typ }) = &binding.typ {
            self.check_type_result(typ);
        }
    }

    fn check_binding_kind(&mut self, binding_kind: &BindingKind) {
        if let BindingKind::Destructed(destructed_tuple) = binding_kind {
            if let Some(PunctuatedBindingKind {
                first_item,
                rest_items,
                trailing_comma: _,
            }) = &destructed_tuple.items
            {
                match first_item {
                    Ok(binding_kind) => self.check_binding_kind(binding_kind),
                    Err(err) => self.report_expected("مُعرِّف", err, vec![]),
                }

                for result in rest_items {
                    match result {
                        Ok(CommaWithBindingKind { comma: _, item }) => {
                            self.check_binding_kind(item)
                        }
                        Err(err) => self.report_expected_comma_or_item("مُعرِّف", err, vec![]),
                    }
                }
            }
            if destructed_tuple.close_delim.is_err() {
                self.report_unclosed_delimiter(destructed_tuple.open_delim.span);
            }
        }
    }

    fn check_block(&mut self, lambda: &LambdaExpr) {
        let LambdaExpr {
            open_curly,
            lambda_arrow: _,
            stms,
            last_expr,
            close_curly,
        } = lambda;

        for stm_result in stms {
            match stm_result {
                Ok(stm) => match stm {
                    Stm::Semicolon(_) => {}
                    Stm::Let(LetStm {
                        let_keyword: _,
                        mut_keyword: _,
                        binding,
                        let_assign,
                        semicolon,
                    }) => {
                        match binding {
                            Ok(binding) => self.check_binding(binding),
                            Err(err) => self.report_expected("مُعرِّف", err, vec![]),
                        }

                        if let Some(LetAssign { equal: _, expr }) = let_assign {
                            self.check_expr_result(expr);
                        }

                        self.check_semicolon_result(semicolon);
                    }
                    Stm::Expr(ExprStm::WithBlock(ExprWithBlockStm { expr, semicolon: _ })) => {
                        self.check_expr_with_block(expr);
                    }
                    Stm::Expr(ExprStm::Any(AnyExprStm { expr, semicolon })) => {
                        self.check_expr(expr);
                        self.check_semicolon_result(semicolon);
                    }
                },
                Err(err) => self.report_expected("جملة برمجية", err, vec![]),
            }
        }

        if let Some(expr) = last_expr {
            self.check_expr(expr);
        }

        if close_curly.is_err() {
            self.report_unclosed_delimiter(open_curly.span);
        }
    }

    fn check_semicolon_result(&mut self, semicolon: &ParseResult<SemicolonSymbol>) {
        if let Err(err) = semicolon {
            let mut i = err.found_token_index - 1;
            while let TokenKind::EOL
            | TokenKind::DelimitedComment
            | TokenKind::LineComment
            | TokenKind::Space = &self.tokens[i].kind
            {
                i -= 1;
            }

            let (found_token_span, found_token_val, secondary_label) =
                if err.found_token_index < self.tokens.len() {
                    let token = &self.tokens[err.found_token_index];
                    (token.span, token.val, "رمز غير متوقع".to_string())
                } else {
                    let last_span = self.tokens[self.tokens.len() - 1].span;
                    (
                        Span::len_after(&last_span, 1),
                        "نهاية الملف",
                        "تم الوصول إلى نهاية الملف".to_string(),
                    )
                };

            let msg = format!(
                "يُتوقع فاصلة منقوطة `؛`، ولكن تم العثور على `{}`",
                found_token_val
            );

            self.report(
                msg,
                Span::len_after(&self.tokens[i].span, 1),
                "قُم هنا بإضافة فاصلة منقوطة `؛`".to_string(),
                vec![(found_token_span, vec![secondary_label.to_string()])],
            );
        }
    }

    fn check_expr_result(&mut self, expr: &ParseResult<Expr>) {
        match expr {
            Ok(expr) => self.check_expr(expr),
            Err(err) => self.report_expected("تعبير برمجي", err, vec![]),
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        // TODO
    }

    fn check_expr_with_block(&mut self, expr: &ExprWithBlock) {
        // TODO
    }

    fn check_non_lambda_expr(&mut self, lambda: &LambdaExpr) {
        if let Some(arrow) = &lambda.lambda_arrow {
            self.report(
                "يُتوقع محتوى غير لامدا".to_string(),
                arrow.span().unwrap(),
                "تم العثور على تعبير لامدا".to_string(),
                vec![],
            );
        }

        self.check_block(lambda);
    }

    fn check_lambda_expr(&mut self, lambda: &LambdaExpr) {
        if let Some(LambdaArrow::WithParams(LambdaParams {
            first,
            rest,
            trailing_comma: _,
            r_arrow,
        })) = &lambda.lambda_arrow
        {
            self.check_binding(first);

            for CommaWithBinding { comma: _, item } in rest {
                self.check_binding(item);
            }

            if let Err(err) = r_arrow {
                self.report_expected("`->`", err, vec![]);
            }
        }

        self.check_block(lambda);
    }
}
