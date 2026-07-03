# foible

Lightweight typed error chaining for Rust — `#[derive(AsSource)]` gives you a
walkable chain of underlying errors without writing any `Display` or
`std::error::Error` boilerplate.

## Why

Rust's `std::error::Error::source()` is useful, but hand-writing it (and
`Display`, and `From` impls) for every error enum in a project is repetitive.
`foible` derives just the chaining behaviour you need, and lets you convert to
a standard error type at the boundary of your application (e.g. with
`anyhow`) when you actually need one.

## Usage

Add the dependency:

```toml
[dependencies]
foible = "0.1"
```

Derive `AsSource` on an error enum. Single-field tuple variants are treated as
wrapped errors and are returned from `next_source()`; annotate a variant with
`#[from]` to also get a generated `From` impl:

```rust
use foible::AsSource;

#[derive(Debug, AsSource)]
enum MyError {
    #[from]
    Io(std::io::Error),

    NotFound,
}

fn next_source(err: &MyError) -> Option<&dyn std::fmt::Debug> {
    err.next_source()
}
```

- Unit variants (like `NotFound`) return `None` from `next_source()`.
- Struct-style variants (`Variant { .. }`) also return `None`.
- Single unnamed-field variants (`Variant(Inner)`) return `Some(&inner)`.
- `#[from]` on a single unnamed-field variant additionally generates
  `impl From<Inner> for MyError`.

At the boundary of your application, convert to a standard error type as
needed, for example:

```rust
.map_err(|e| anyhow::anyhow!("{e:?}"))
```

## Crates

- [`foible`](./foible) — the public `AsSource` trait and derive macro re-export.
- [`foible-macros`](./foible-macros) — proc-macro internals for `foible`. Not
  intended to be used directly.

## License

Licensed under the [MIT license](./LICENSE).
