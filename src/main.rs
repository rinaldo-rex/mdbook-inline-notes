//! mdbook preprocessor that converts `{{note: ...}}` markers into inline
//! hover-triggered popovers.
//!
//! Notes are included like: `Some text{{note: This is a note with **markdown**}} in body.`
//!
//! The `markdown` boolean config value in `book.toml` indicates that Markdown
//! should be emitted for non-HTML backends instead of HTML popovers.
use clap::{Arg, Command};
use log::warn;
use mdbook_preprocessor::{
    book::{Book, BookItem},
    errors::Error,
    Preprocessor, PreprocessorContext,
};
use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::sync::LazyLock;
use std::{io, process};

/// Name of this preprocessor.
const NAME: &str = "inplace-note-preprocessor";

/// Regex to match `{{note: ...}}` markers (dot-matches-newline for multiline content).
static NOTE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)\{\{note:\s*(?P<content>.*?)\}\}").unwrap());

/// The CSS injected once per chapter when notes are present.
///
/// All colors use `var()` references to mdBook's CSS custom properties
/// so the popover automatically adapts to the current theme (light, coal,
/// navy, ayu, rust, or any custom theme).
const POPOVER_CSS: &str = r#"<style>
.inplace-note-trigger {
  font-size: 0.75em;
  vertical-align: super;
  line-height: 0;
  color: var(--links, #0969da);
  font-weight: 600;
  cursor: help;
}
/* Inline wrapper — surrounds trigger in inline context */
.inplace-note-inline {
  position: relative;
  display: inline;
}
/* Popover for inline-only notes (no block elements) */
.inplace-note-inline .inplace-note-popover-inline {
  display: none;
  position: absolute;
  bottom: calc(100% + 10px);
  left: 50%;
  transform: translateX(-50%);
  background: var(--theme-popup-bg, var(--bg, #fff));
  color: var(--fg, #000);
  border: 1px solid var(--theme-popup-border, #d0d7de);
  border-radius: 8px;
  padding: 10px 14px;
  min-width: 200px;
  max-width: 380px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.15);
  z-index: 1000;
  font-size: 1.6rem;
  font-weight: normal;
  line-height: 1.5;
  text-align: left;
  white-space: normal;
  cursor: auto;
}
.inplace-note-inline .inplace-note-popover-inline::after {
  content: "";
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 7px solid transparent;
  border-top-color: var(--theme-popup-bg, var(--bg, #fff));
}
.inplace-note-inline .inplace-note-popover-inline::before {
  content: "";
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 8px solid transparent;
  border-top-color: var(--theme-popup-border, #d0d7de);
}
.inplace-note-inline:hover .inplace-note-popover-inline {
  display: block;
}
/* Block wrapper — used when note content has block elements */
.inplace-note-block {
  position: relative;
  display: inline-block;
  vertical-align: top;
}
.inplace-note-block .inplace-note-popover-block {
  display: none;
  position: absolute;
  bottom: calc(100% + 10px);
  left: 50%;
  transform: translateX(-50%);
  background: var(--theme-popup-bg, var(--bg, #fff));
  color: var(--fg, #000);
  border: 1px solid var(--theme-popup-border, #d0d7de);
  border-radius: 8px;
  padding: 12px 16px;
  min-width: 240px;
  max-width: 420px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.15);
  z-index: 1000;
  font-size: 1.6rem;
  font-weight: normal;
  line-height: 1.5;
  text-align: left;
  white-space: normal;
  cursor: auto;
}
.inplace-note-block .inplace-note-popover-block::after {
  content: "";
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 7px solid transparent;
  border-top-color: var(--theme-popup-bg, var(--bg, #fff));
}
.inplace-note-block .inplace-note-popover-block::before {
  content: "";
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 8px solid transparent;
  border-top-color: var(--theme-popup-border, #d0d7de);
}
.inplace-note-block:hover .inplace-note-popover-block {
  display: block;
}
/* Popover inner content styling */
.inplace-note-content a {
  color: var(--links, #0969da);
}
.inplace-note-content a:hover {
  color: var(--links, #0969da);
}
.inplace-note-content p {
  margin: 0.25em 0;
}
.inplace-note-content p:first-child {
  margin-top: 0;
}
.inplace-note-content p:last-child {
  margin-bottom: 0;
}
.inplace-note-content pre {
  margin: 0.5em 0;
  padding: 8px 12px;
  font-size: var(--code-font-size, 0.875em);
  background: var(--quote-bg, #f6f8fa);
}
.inplace-note-content code {
  font-size: var(--code-font-size, 0.875em);
  color: var(--inline-code-color, #301900);
}
.inplace-note-content pre code {
  color: var(--fg, #000);
}
.inplace-note-content ul,
.inplace-note-content ol {
  margin: 0.25em 0;
  padding-left: 1.5em;
}
.inplace-note-content blockquote {
  margin: 0.5em 0;
  padding: 0 0.75em;
  border-left: 3px solid var(--theme-popup-border, #d0d7de);
  color: var(--fg, #656d76);
}
.inplace-note-content table {
  margin: 0.5em 0;
  border-collapse: collapse;
  font-size: 0.85em;
}
.inplace-note-content th,
.inplace-note-content td {
  padding: 4px 8px;
  border: 1px solid var(--table-border-color, #d0d7de);
}
.inplace-note-content th {
  background: var(--table-header-bg, #f6f8fa);
}
.inplace-note-content td {
  background: var(--theme-popup-bg, var(--bg, #fff));
}
</style>"#;

pub fn make_app() -> Command {
    Command::new("inplace-note-preprocessor")
        .about("An mdbook preprocessor which converts {{note: ...}} markers into hover popovers")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    env_logger::init();
    let matches = make_app().get_matches();
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        let renderer = sub_args
            .get_one::<String>("renderer")
            .expect("Required argument");

        if InplaceNote::supports_renderer(renderer) {
            process::exit(0);
        } else {
            process::exit(1);
        }
    } else {
        let (ctx, book) =
            mdbook_preprocessor::parse_input(io::stdin()).expect("Failed to parse input");
        let preprocessor = InplaceNote::new(&ctx);

        let processed_book = preprocessor
            .run(&ctx, book)
            .expect("Failed to process book");
        serde_json::to_writer(io::stdout(), &processed_book)
            .expect("Failed to emit processed book");
    }
}

/// A pre-processor that expands `{{note: ...}}` markers into inline hover popovers.
#[derive(Default)]
pub struct InplaceNote {
    /// Whether to emit Markdown instead of HTML (for non-HTML backends).
    md_notes: bool,
}

impl InplaceNote {
    fn new(ctx: &PreprocessorContext) -> Self {
        if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
            warn!(
                "The {NAME} plugin was built against version {} of mdbook, \
                 but we're being called from version {}",
                mdbook_preprocessor::MDBOOK_VERSION,
                ctx.mdbook_version
            );
        }

        let md_notes = ctx
            .config
            .get::<bool>("preprocessor.inplace-note.markdown")
            .ok()
            .flatten()
            .unwrap_or(false);

        if !md_notes {
            let html_renderers = ["html", "linkcheck"];
            if !html_renderers.contains(&ctx.renderer.as_str()) {
                warn!(
                    "Emitting HTML notes for renderer '{}' which may not be HTML-based",
                    ctx.renderer,
                );
            }
        }

        Self { md_notes }
    }

    /// Indicate whether a renderer is supported.  Markdown mode can target any renderer.
    fn supports_renderer(renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

impl Preprocessor for InplaceNote {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chap) = item {
                let mut notes: Vec<(String, bool)> = vec![]; // (content, has_blocks)
                chap.content = NOTE_RE
                    .replace_all(&chap.content, |caps: &regex::Captures| {
                        let content = caps.name("content").unwrap().as_str().to_owned();
                        let rendered = render_note_html(&content);
                        let has_blocks = note_has_blocks(&rendered);
                        notes.push((rendered, has_blocks));
                        let idx = notes.len();

                        if self.md_notes {
                            format!("[^{}]", idx)
                        } else if has_blocks {
                            // Block content: emit marker inside a block wrapper div
                            // that closes the current paragraph and starts a new one.
                            format!("\n\n<div class=\"inplace-note-block\"><sup class=\"inplace-note-trigger\">{idx}</sup><div class=\"inplace-note-popover-block\"><div class=\"inplace-note-content\">{content_html}</div></div></div>\n\n", idx = idx, content_html = notes[idx - 1].0)
                        } else {
                            // Inline-only: use the inline span approach.
                            format!(
                                "<span class=\"inplace-note-inline\"><sup class=\"inplace-note-trigger\">{idx}</sup><span class=\"inplace-note-popover-inline\"><span class=\"inplace-note-content\">{content_html}</span></span></span>",
                                idx = idx,
                                content_html = notes[idx - 1].0
                            )
                        }
                    })
                    .to_string();

                // Inject CSS before chapter content (only if HTML mode and notes exist).
                if !self.md_notes && !notes.is_empty() {
                    chap.content = format!("{POPOVER_CSS}\n{chap_content}", chap_content = chap.content);
                }

                // In markdown mode, append note definitions at the end of the chapter.
                if self.md_notes && !notes.is_empty() {
                    chap.content += "\n---\n";
                    for (idx, (rendered, _)) in notes.into_iter().enumerate() {
                        chap.content += &format!("\n\n[^{}]: {}", idx + 1, rendered);
                    }
                }
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool, Error> {
        Ok(Self::supports_renderer(renderer))
    }
}

/// Render markdown content to HTML for display inside the popover.
///
/// If the content is a single paragraph (no block elements inside),
/// the outer `<p>` tags are stripped so the content flows naturally
/// inside the inline popover span.
fn render_note_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);
    let mut html = String::new();
    html::push_html(&mut html, parser);

    // Strip wrapping <p>...</p> if the content is a single paragraph
    // with no nested block-level elements.
    let trimmed = html.trim();
    if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") {
        let inner = &trimmed[3..trimmed.len() - 4];
        let block_tags = [
            "<p>", "<ul>", "<ol>", "<pre>", "<blockquote>",
            "<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>",
            "<table>", "<hr",
        ];
        if !block_tags.iter().any(|tag| inner.contains(tag)) {
            return inner.to_string();
        }
    }

    html
}

/// Check whether rendered HTML contains block-level elements.
fn note_has_blocks(html: &str) -> bool {
    let trimmed = html.trim();
    let block_tags = [
        "<p>", "<ul>", "<ol>", "<pre>", "<blockquote>",
        "<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>",
        "<table>", "<hr",
    ];
    // Check if the content has more than one paragraph or any block tag.
    // A single <p> wrapper doesn't count.
    let inner = if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") {
        &trimmed[3..trimmed.len() - 4]
    } else {
        trimmed
    };
    block_tags.iter().any(|tag| inner.contains(tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_simple_note() {
        let input = "Hello{{note: a tip}} world";
        let mut notes = vec![];
        let result = NOTE_RE
            .replace_all(input, |caps: &regex::Captures| {
                let content = caps.name("content").unwrap().as_str().to_owned();
                notes.push(content);
                format!("[{}]", notes.len())
            })
            .to_string();
        assert_eq!(result, "Hello[1] world");
        assert_eq!(notes, vec!["a tip"]);
    }

    #[test]
    fn test_regex_multiline_note() {
        let input = "Text{{note: line1\nline2}} end";
        let mut notes = vec![];
        NOTE_RE.replace_all(input, |caps: &regex::Captures| {
            notes.push(caps.name("content").unwrap().as_str().to_owned());
            ""
        });
        assert_eq!(notes, vec!["line1\nline2"]);
    }

    #[test]
    fn test_regex_multiple_notes() {
        let input = "A{{note: first}} B{{note: second}} C";
        let mut notes = vec![];
        let result = NOTE_RE
            .replace_all(input, |caps: &regex::Captures| {
                let content = caps.name("content").unwrap().as_str().to_owned();
                notes.push(content);
                format!("[{}]", notes.len())
            })
            .to_string();
        assert_eq!(result, "A[1] B[2] C");
        assert_eq!(notes, vec!["first", "second"]);
    }

    #[test]
    fn test_render_note_html_inline() {
        let html = render_note_html("just some text");
        // Should NOT have <p> wrapper
        assert!(!html.contains("<p>"), "unexpected <p>: {html}");
        assert_eq!(html.trim(), "just some text");
    }

    #[test]
    fn test_render_note_html_with_formatting() {
        let html = render_note_html("**bold** and *italic*");
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
        assert!(!html.contains("<p>"), "unexpected <p>: {html}");
    }

    #[test]
    fn test_render_note_html_with_code() {
        let html = render_note_html("use `foo()` here");
        assert!(html.contains("<code>foo()</code>"));
    }

    #[test]
    fn test_render_note_html_with_block_elements() {
        // Content with a list should keep <p> tags
        let html = render_note_html("para\n\n- item 1\n- item 2");
        assert!(html.contains("<ul>"), "expected <ul> in: {html}");
        assert!(html.contains("<li>"), "expected <li> in: {html}");
    }

    #[test]
    fn test_render_note_html_empty() {
        let html = render_note_html("");
        assert!(html.is_empty() || html == "<p>\n</p>\n");
    }

    #[test]
    fn test_note_has_blocks_inline() {
        let html = render_note_html("just text");
        assert!(!note_has_blocks(&html));
    }

    #[test]
    fn test_note_has_blocks_with_list() {
        let html = render_note_html("text\n\n- item");
        assert!(note_has_blocks(&html));
    }

    #[test]
    fn test_note_has_blocks_with_codeblock() {
        let html = render_note_html("```rust\nlet x = 1;\n```");
        assert!(note_has_blocks(&html));
    }

    #[test]
    fn test_note_has_blocks_with_table() {
        let html = render_note_html("| a | b |\n|---|---|\n| 1 | 2 |");
        assert!(note_has_blocks(&html));
    }
}
