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

mod generics {
    use foible::Trace;
    use std::fmt::Debug;

    #[derive(Debug, Trace)]
    enum Leaf {
        Gone,
    }

    // Only `Debug` is known about `E`, so the wrapped value is rendered with `Debug`.
    #[derive(Debug, Trace)]
    enum Retry<E> {
        Exhausted(E),
        Cancelled,
    }

    // Declaring `E: Trace` makes the chain continue into `E`.
    #[derive(Debug, Trace)]
    enum Chained<E: Trace> {
        Failed(E),
    }

    #[allow(dead_code)]
    #[derive(Debug, Trace)]
    enum Where<'a, E, const N: usize>
    where
        E: Debug + 'a,
    {
        #[from]
        Many(Vec<E>),
        Borrowed(&'a E),
        Arr([u8; N]),
    }

    #[test]
    fn generic_params() {
        assert_eq!(Retry::Exhausted(3).trace(), "Retry::Exhausted\n  3");
        assert_eq!(Retry::<u8>::Cancelled.trace(), "Retry::Cancelled");
        assert_eq!(
            Chained::Failed(Leaf::Gone).trace(),
            "Chained::Failed\n  Leaf::Gone"
        );
    }

    #[test]
    fn lifetimes_consts_and_from() {
        let e: Where<'_, u8, 2> = vec![1u8].into();
        assert_eq!(e.trace(), "Where::Many\n  [1]");
        assert_eq!(Where::<u8, 2>::Arr([1, 2]).trace(), "Where::Arr\n  [1, 2]");
    }
}
