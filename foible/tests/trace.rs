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

// The derive must not rely on prelude names resolving to the std items.
#[allow(dead_code, non_camel_case_types)]
mod shadowed_prelude {
    use foible::Trace;

    struct From;
    struct Vec;
    struct String;
    struct Option;
    struct Some;
    struct None;
    struct Result;
    struct Ok;
    struct Err;
    struct Box;
    struct Self_;

    #[derive(Debug, Trace)]
    enum Inner {
        Leaf,
    }

    #[derive(Debug, Trace)]
    enum Shadowed {
        #[from]
        Inner(Inner),
        Leaf,
    }

    #[test]
    fn derive_survives_shadowed_prelude() {
        let e = Shadowed::from(Inner::Leaf);
        assert_eq!(e.trace(), "Shadowed::Inner\n  Inner::Leaf");
    }
}
