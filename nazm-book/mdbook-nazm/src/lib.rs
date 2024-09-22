// use markdown::Block;
use mdbook::{
    book::{Book, Chapter},
    errors::Result,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use nazmc_display_table::DisplayTable;
use nazmc_lexer::{LexerIter, LiteralKind, Token};
use pulldown_cmark::{CodeBlockKind, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;

pub struct NazmPreprocessor;

impl Preprocessor for NazmPreprocessor {
    fn name(&self) -> &str {
        "nazm"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ch) = item {
                handle_chapter(ch);
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn handle_chapter(ch: &mut Chapter) {
    let parser = Parser::new_ext(&ch.content, Options::all());

    let mut nazm_code_block_is_found = false;

    let events = parser.map(|event| match event {
        pulldown_cmark::Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
            if lang.clone().into_string() == "نظم" =>
        {
            nazm_code_block_is_found = true;
            pulldown_cmark::Event::Html(
                "\n\n<pre class=\"language-nazm\" data-highlighted=\"yes\"><code>".into(),
            )
        }
        pulldown_cmark::Event::Text(nazm_code) if nazm_code_block_is_found => {
            pulldown_cmark::Event::Html(nazm_code_to_html(nazm_code.into_string()).into())
        }
        pulldown_cmark::Event::End(TagEnd::CodeBlock) if nazm_code_block_is_found => {
            nazm_code_block_is_found = false;
            pulldown_cmark::Event::Html("</code></pre>\n\n".into())
        }
        other => other,
    });

    let mut content = String::new();

    let _ = cmark(events, &mut content);

    ch.content = content;
}

const COMMENT_STYLE: &str = "hljs-comment";
const TEXT_LITERAL_STYLE: &str = "hljs-string";
const NUM_LITERAL_STYLE: &str = "hljs-number";
const BOOL_LITERAL_STYLE: &str = "hljs-literal";
const ID_STYLE: &str = "hljs-title";
const KEYWORD_STYLE: &str = "hljs-keyword";

fn nazm_code_to_html(nazm_code: String) -> String {
    let mut display_table = DisplayTable::new();
    let (tokens, ..) = LexerIter::new(&nazm_code, &mut display_table).collect_all();

    let mut final_html = String::new();
    for Token { val, span: _, kind } in tokens {
        let html = match kind {
            nazmc_lexer::TokenKind::Eof => "",
            nazmc_lexer::TokenKind::Eol => "\n",
            nazmc_lexer::TokenKind::Space => val,
            nazmc_lexer::TokenKind::LineComment | nazmc_lexer::TokenKind::DelimitedComment => {
                &span(COMMENT_STYLE, val)
            }
            nazmc_lexer::TokenKind::Literal(LiteralKind::Char(_) | LiteralKind::Str(_)) => {
                &span(TEXT_LITERAL_STYLE, val)
            }
            nazmc_lexer::TokenKind::Literal(LiteralKind::Num(_)) => &span(NUM_LITERAL_STYLE, val),
            nazmc_lexer::TokenKind::Literal(LiteralKind::Bool(_)) => &span(BOOL_LITERAL_STYLE, val),
            nazmc_lexer::TokenKind::Id(_) => &span(ID_STYLE, val),
            nazmc_lexer::TokenKind::Symbol(_) => val,
            nazmc_lexer::TokenKind::Keyword(_) => &span(KEYWORD_STYLE, val),
        };
        final_html.push_str(html);
    }
    final_html
}

#[inline]
fn span(style: &str, val: &str) -> String {
    format!("<span class =\"{}\">{}</span>", style, val)
}
