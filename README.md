# foible

Lightweight typed error chaining for Rust — `#[derive(Trace)]` gives you a
readable trace of the chain of underlying errors without writing any `Display`
or `std::error::Error` boilerplate.

## Why

Rust's `std::error::Error::source()` is useful, but hand-writing it (and
`Display`, and `From` impls) for every error enum in a project is repetitive.
`foible` derives just the chaining behaviour you need, and lets you render the
whole chain at the boundary of your application (e.g. with `anyhow`) when you
actually need a readable trace.

## Usage

Add the dependency:

```toml
[dependencies]
foible = "0.2"
```

Derive `Trace` on an error enum. Single-field tuple variants are treated as
wrapped errors and are followed when building the trace; annotate a variant
with `#[from]` to also get a generated `From` impl:

```rust
use foible::Trace;

#[derive(Debug, Trace)]
enum MyError {
    #[from]
    Io(std::io::Error),

    NotFound,
}

let err = MyError::from(std::io::Error::other("disk full"));
assert_eq!(err.trace(), "MyError::Io\n  disk full");
```

- Each link in the chain is one line, outermost first.
- Unit and struct-style variants (`NotFound`, `Variant { .. }`) end the chain.
- For a single unnamed-field variant (`Variant(Inner)`), the inner value is
  rendered using the first of these it implements: `Trace` (the chain
  continues), `Display`, then `Debug`. The choice is made on the concrete
  field type.
- Generic enums are supported. Every type parameter gets a `Debug` bound. A
  field whose type is a bare parameter `E` is rendered with `Debug` unless you
  declare `E: Trace` (or `Display`) on the enum. `#[from]` is rejected on such
  a field, as it would conflict with the standard `impl<T> From<T> for T`.
- `#[from]` on a single unnamed-field variant additionally generates
  `impl From<Inner> for MyError`.

At the boundary of your application, render the trace, for example:

```rust
.map_err(|e| anyhow::anyhow!("{}", e.trace()))
```

## Crates

- [`foible`](./foible) — the public `Trace` trait and derive macro re-export.
- [`foible-derive`](./foible-derive) — proc-macro internals for `foible`. Not
  intended to be used directly.

## License

Licensed under the [MIT license](./LICENSE).
