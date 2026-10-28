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

Derive `Trace` on an error enum. Each variant renders as one line, followed
by the trace of its cause, if it has one:

```rust
use foible::Trace;

#[derive(Debug, Trace)]
enum DbError {
    Timeout { after_ms: u32 },
}

#[derive(Debug, Trace)]
enum MyError {
    #[from]
    Io(std::io::Error),

    Db(DbError, &'static str),

    NotFound,
}

let err = MyError::Db(DbError::Timeout { after_ms: 30 }, "users");
assert_eq!(
    err.trace(),
    "MyError::Db(users)\n  DbError::Timeout(after_ms: 30)"
);

let err = MyError::from(std::io::Error::new(std::io::ErrorKind::Other, "disk full"));
assert_eq!(err.trace(), "MyError::Io(disk full)");
```

- Each field is either the **cause** or **data**. A field whose type implements
  `Trace` is the cause: its trace follows on the next line, indented. Every
  other field is data and is written inline as `Variant(a; b)` (or
  `Variant(name: a; other: b)` for named fields), using `Display`, falling
  back to `Debug`. Variants without data have no parentheses.
- Data is written as is. A `Display` value containing newlines (some
  `io::Error` messages, for example) will span several lines and break the
  one-line-per-link layout. `foible` deliberately doesn't escape or re-indent
  it; if that matters, give the field a type whose `Display` doesn't emit
  newlines.
- A variant can have at most one `Trace` field. A second one is a compile
  error. For anything more complex, such as aggregate errors, implement
  `Trace` by hand.
- The choice is made on the concrete field type, in the order `Trace`,
  `Display`, `Debug`.
- `#[from]` on a variant with exactly one field (tuple or named) additionally
  generates `impl From<Inner> for MyError`. It is a compile error on any other
  variant.
- Generic enums are supported. Every type parameter gets a `Debug` bound. A
  field whose type is a bare parameter `E` is data (`Debug`) unless you declare
  `E: Trace` (or `Display`) on the enum. `#[from]` is rejected on such a field,
  as it would conflict with the standard `impl<T> From<T> for T`.

## Boundaries and `std::error::Error`

`trace()` returns the rendered `String`, so at a boundary you can log or return
it directly (`display_trace()` does the same without allocating). To hand an error to code that expects `std::error::Error` (`?`
into `Box<dyn Error>`, `anyhow`, `eyre`, another crate's `#[from]`), opt in
with `#[trace(error)]`:

```rust
use foible::Trace;
use std::error::Error;

#[derive(Debug, Trace)]
enum DbError {
    Timeout { after_ms: u32 },
}

#[derive(Debug, Trace)]
#[trace(error)]
enum MyError {
    #[from]
    Db(DbError),
}

fn run() -> Result<(), Box<dyn Error>> {
    Err(MyError::from(DbError::Timeout { after_ms: 30 }))?;
    Ok(())
}

let err = run().unwrap_err();
assert_eq!(err.to_string(), "MyError::Db\n  DbError::Timeout(after_ms: 30)");
assert!(err.downcast_ref::<MyError>().is_some());
```

This generates `Display` (the full trace) and `std::error::Error` (with
`source()` returning `None`, so tools that print `Display` and then walk
`source()` don't repeat the chain). The type isn't erased: `downcast_ref` on
the boxed error gets your enum back.

It is opt-in because it takes over `Display` for the type. Don't use it if you
want to write your own `Display` or `Error` impl.

## `no_std`

`foible` is `#![no_std]`. Features:

- `std` (default): implies `alloc`.
- `alloc`: enables `trace() -> String`.

Without either, `err.display_trace()` returns a value that implements
`Display`, so you can write a trace straight into a logger or a fixed buffer
without allocating:

```toml
foible = { version = "0.2", default-features = false }
```

`#[trace(error)]` generates a `std::error::Error` impl, so it needs `std`.

## Minimum supported Rust version

Rust 1.71.

## Crates

- [`foible`](./foible) — the public `Trace` trait and derive macro re-export.
- [`foible-derive`](./foible-derive) — proc-macro internals for `foible`. Not
  intended to be used directly.

## License

Licensed under the [MIT license](./LICENSE).
