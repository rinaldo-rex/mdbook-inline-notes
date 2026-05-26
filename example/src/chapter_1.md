# Chapter 1: Getting Started

Welcome to the demo of **mdbook-inplace-notes**!

## Simple inline notes

The `HashMap` type{{note: A hash map implemented with **quadratic probing** and
**SIMD** lookup. See [std::collections](https://doc.rust-lang.org/std/collections/)
for details.}} is Rust's standard key-value store.

Here's a `Vec`{{note: A contiguous, growable array type, written `Vec<T>`. It
allocates on the heap and provides O(1) indexing.}} — the most common collection.

## Notes with inline formatting

The `?` operator{{note: The `?` operator is syntactic sugar for propagating
errors. It's equivalent to `match` with early return on `Err`.}} simplifies error
handling in Rust.

Use `cargo build --release`{{note: The `--release` flag enables optimizations
(equivalent to `opt-level = 3` in `Cargo.toml`). Builds are slower but runtime is
much faster.}} for production builds.

## Notes with block content

Ownership in Rust{{note:

Rust's ownership rules:
- Each value has exactly one **owner**
- When the owner goes out of scope, the value is **dropped**
- Values can be **moved** or **borrowed**

See [The Book](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) for more.

}} is what makes Rust unique.

## Multiple notes in one paragraph

Rust has several string{{note: `String` is an owned, heap-allocated, growable
UTF-8 string.}} types{{note: `&str` is a borrowed string slice pointing to
valid UTF-8 data.}} for different use cases.
