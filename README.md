# mdbook-inplace-notes

A preprocessor for [mdBook](https://github.com/rust-lang/mdBook) that adds inline
notes with hover-triggered popovers, inspired by [TipTap's Popover](https://tiptap.dev/docs/ui-components/primitives/popover).

It turns this:

```text
Normal text{{note: This is a helpful tip with **markdown**.}} in body.
```

into a superscript number that shows a styled popover on hover containing the
rendered markdown content.

## Features

- **Pure CSS hover popovers** — no JavaScript required
- **Auto-numbered superscript** markers (¹, ², ³…)
- **Full markdown** in note content (bold, italic, code, lists, blockquotes, etc.)
- **Theme-aware** — popover colors auto-adapt to the active mdBook theme (light, coal, navy, ayu, rust)
- **Self-contained** — CSS is injected inline, no extra files needed
- **Markdown fallback mode** — for non-HTML backends

## Installation

```sh
cargo install mdbook-inplace-notes
```

## Usage

Add it as a preprocessor in your `book.toml`:

```toml
[preprocessor.inplace-note]
```

Then in your markdown, use `{{note: content}}`:

```markdown
The `HashMap` type{{note: A hash map implemented with **quadratic probing** and
**SIMD** lookup. See [std::collections](https://doc.rust-lang.org/std/collections/).}}
is a common collection.
```

Hover over the superscript number to see the popover.

## Configuration

```toml
[preprocessor.inplace-note]
# Emit Markdown instead of HTML (for non-HTML renderers like the print backend).
# Default: false
markdown = false

# Popover font size (any valid CSS value: "80%", "14px", "1.2rem", etc.)
# Default: "1.6rem" (matches mdBook's body font size)
popover-font-size = "1.6rem"

# Popover max-width (any valid CSS value: "500px", "40vw", "30rem", etc.)
# Default: "380px"
popover-max-width = "380px"

# Popover min-width (any valid CSS value: "300px", "25vw", "20rem", etc.)
# Default: "200px"
popover-min-width = "200px"
```

## How It Works

The preprocessor runs after mdBook loads the book but before rendering. It:

1. Finds all `{{note: ...}}` markers in each chapter using regex
2. Renders the note content from markdown to HTML using `pulldown-cmark` (the
   same parser mdBook uses)
3. Replaces each marker with a styled `<span>` containing a `superscript` trigger
   and a hidden popover `<span>` that appears on hover
4. Injects a `<style>` block with the popover CSS at the top of the chapter
   (only when notes are present)

## License

MIT
