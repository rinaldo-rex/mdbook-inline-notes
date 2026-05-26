# Chapter 2: Advanced Notes

This chapter covers more complex note content.

## Notes with code blocks

The `Iterator` trait{{note:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

The heart of Rust's iterator system — just one required method.
}} powers Rust's functional programming patterns.

## Notes with lists

Error handling in Rust{{note:

Common patterns:
- **Recoverable**: `Result<T, E>` with `?` operator
- **Unrecoverable**: `panic!` macro
- **Optional values**: `Option<T>` with `?` operator (in functions returning `Option`)

Choose the right tool for the situation.
}} is versatile and explicit.

## Notes with blockquotes

The module system{{note:

> Modules let you organize code within a crate into groups for readability and reuse.
> They also control the *privacy* of items: public, private, or `pub(crate)`.

See [The Book](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects.html).
}} helps organize large codebases.

## Notes with inline code and links

Clippy{{note: Run `cargo clippy` to get **lint suggestions**. Configure with
`#[allow(clippy::some_lint)]` attributes. See the
[Clippy Lints](https://rust-lang.github.io/rust-clippy/) page.}} catches common
mistakes.

## Notes with tables

Rust edition differences{{note:

| Edition | Year  | Key Feature     |
|---------|-------|-----------------|
| 2015    | 2015  | Original        |
| 2018    | 2018  | NLL, `dyn Trait`|
| 2021    | 2021  | Closure captures |
| 2024    | 2024  | `gen` blocks     |

}} can trip up newcomers.
