use foible::Trace;

#[derive(Debug, Trace)]
enum Inner {
    #[from]
    Io(std::io::Error),
    Missing,
}

#[derive(Debug, Trace)]
enum Outer {
    #[from]
    Inner(Inner),
    Msg(NoDisplay),
    Gone,
}

#[derive(Debug)]
struct NoDisplay;

#[test]
fn chain_through_trace_and_display() {
    let e: Outer = Inner::from(std::io::Error::new(std::io::ErrorKind::Other, "disk")).into();
    assert_eq!(e.trace(), "Outer::Inner\n  Inner::Io\n  disk");
}

#[test]
fn leaf_and_debug_fallback() {
    assert_eq!(Outer::Gone.trace(), "Outer::Gone");
    assert_eq!(Outer::Msg(NoDisplay).trace(), "Outer::Msg\n  NoDisplay");
    assert_eq!(
        Outer::from(Inner::Missing).trace(),
        "Outer::Inner\n  Inner::Missing"
    );
}
