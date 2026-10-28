//! Prints the trace for each case the derive supports.
//!
//! Run with: `cargo run -p foible --example traces`

use foible::Trace;

#[derive(Debug, Trace)]
enum Baz {
    C,
}

#[derive(Debug, Trace)]
enum Bar {
    Tuple(u8, &'static str),
    Named { id: u8, name: &'static str },
    Wraps(Baz),
    EmptyTuple(),
    EmptyNamed {},
}

#[derive(Debug, Trace)]
enum Foo {
    Unit,
    #[from]
    Io(std::io::Error),
    #[from]
    Cause(Bar),
    #[from]
    NamedCause {
        err: Baz,
    },
    DataThenCause(Bar, &'static str),
    CauseThenData(&'static str, Bar),
    NamedCauseAndData {
        err: Bar,
        id: u8,
    },
    DebugOnly(NoDisplay),
}

#[derive(Debug)]
struct NoDisplay {
    _reason: &'static str,
}

#[derive(Debug, Trace)]
enum Retry<E> {
    Exhausted(E),
}

#[derive(Debug, Trace)]
enum RetryTraced<E: Trace> {
    Exhausted(E),
}

#[derive(Debug, Trace)]
#[trace(error)]
enum Boundary {
    #[from]
    Cause(Foo),
}

fn boundary() -> Result<(), Box<dyn std::error::Error>> {
    Err(Boundary::from(Foo::from(Bar::Wraps(Baz::C))))?;
    Ok(())
}

fn show(title: &str, err: &impl Trace) {
    println!("# {title}\n{}\n", err.trace());
}

fn main() {
    show("unit variant", &Foo::Unit);
    show(
        "wrapped non-Trace error (Display, as data)",
        &Foo::from(std::io::Error::new(std::io::ErrorKind::Other, "disk full")),
    );
    show(
        "data only: Debug fallback",
        &Foo::DebugOnly(NoDisplay { _reason: "n/a" }),
    );

    show("tuple data", &Bar::Tuple(0, "Baz"));
    show("named data", &Bar::Named { id: 0, name: "Baz" });
    show("empty tuple variant", &Bar::EmptyTuple());
    show("empty named variant", &Bar::EmptyNamed {});

    show("single cause", &Foo::from(Bar::Wraps(Baz::C)));
    show("named single cause", &Foo::from(Baz::C));
    show(
        "data then cause",
        &Foo::DataThenCause(Bar::Wraps(Baz::C), "users"),
    );
    show(
        "cause then data",
        &Foo::CauseThenData("users", Bar::Wraps(Baz::C)),
    );
    show(
        "named cause and data",
        &Foo::NamedCauseAndData {
            err: Bar::Wraps(Baz::C),
            id: 7,
        },
    );
    show(
        "deep chain with data in the middle",
        &Foo::from(Bar::Named {
            id: 1,
            name: "leaf",
        }),
    );

    show("generic, bare parameter (Debug)", &Retry::Exhausted(3));
    show(
        "generic, `E: Trace` (chain continues)",
        &RetryTraced::Exhausted(Bar::Wraps(Baz::C)),
    );

    println!(
        "# #[trace(error)]: Display is the trace, via Box<dyn Error>\n{}",
        boundary().unwrap_err()
    );
}
