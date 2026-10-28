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
    assert_eq!(e.trace(), "Outer::Inner\n  Inner::Io(disk)");
}

#[test]
fn leaf_and_debug_fallback() {
    assert_eq!(Outer::Gone.trace(), "Outer::Gone");
    assert_eq!(Outer::Msg(NoDisplay).trace(), "Outer::Msg(NoDisplay)");
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
        assert_eq!(Retry::Exhausted(3).trace(), "Retry::Exhausted(3)");
        assert_eq!(Retry::<u8>::Cancelled.trace(), "Retry::Cancelled");
        assert_eq!(
            Chained::Failed(Leaf::Gone).trace(),
            "Chained::Failed\n  Leaf::Gone"
        );
    }

    #[test]
    fn lifetimes_consts_and_from() {
        let e: Where<'_, u8, 2> = vec![1u8].into();
        assert_eq!(e.trace(), "Where::Many([1])");
        assert_eq!(Where::<u8, 2>::Arr([1, 2]).trace(), "Where::Arr([1, 2])");
    }
}

mod fields {
    use foible::Trace;

    #[derive(Debug, Trace)]
    enum Baz {
        C,
    }

    #[derive(Debug, Trace)]
    enum Bar {
        Plain(u8, &'static str),
        Named { id: u8, name: &'static str },
        Wraps(Baz),
        Empty(),
        EmptyNamed {},
    }

    #[derive(Debug, Trace)]
    enum Foo {
        DataThenCause(Bar, &'static str),
        CauseThenData(&'static str, Bar),
        NamedCause {
            err: Bar,
            id: u8,
        },
        #[from]
        NamedSingle {
            err: Baz,
        },
    }

    #[test]
    fn data_fields_inline() {
        assert_eq!(Bar::Plain(0, "Baz").trace(), "Bar::Plain(0; Baz)");
        assert_eq!(
            Bar::Named { id: 0, name: "Baz" }.trace(),
            "Bar::Named(id: 0; name: Baz)"
        );
        assert_eq!(Bar::Empty().trace(), "Bar::Empty");
        assert_eq!(Bar::EmptyNamed {}.trace(), "Bar::EmptyNamed");
    }

    #[test]
    fn cause_on_following_line() {
        let cause = || Bar::Wraps(Baz::C);
        assert_eq!(
            Foo::DataThenCause(cause(), "x").trace(),
            "Foo::DataThenCause(x)\n  Bar::Wraps\n  Baz::C"
        );
        assert_eq!(
            Foo::CauseThenData("x", cause()).trace(),
            "Foo::CauseThenData(x)\n  Bar::Wraps\n  Baz::C"
        );
        assert_eq!(
            Foo::NamedCause {
                err: cause(),
                id: 7
            }
            .trace(),
            "Foo::NamedCause(id: 7)\n  Bar::Wraps\n  Baz::C"
        );
    }

    #[test]
    fn from_on_named_single_field() {
        assert_eq!(Foo::from(Baz::C).trace(), "Foo::NamedSingle\n  Baz::C");
    }
}

mod error_interop {
    use foible::Trace;
    use std::error::Error;

    #[derive(Debug, Trace)]
    enum Leaf {
        Gone,
    }

    #[derive(Debug, Trace)]
    #[trace(error)]
    enum Failure {
        #[from]
        Cause(Leaf),
        Data(u8),
    }

    #[derive(Debug, Trace)]
    #[trace(error)]
    enum Generic<E: Trace> {
        Wrapped(E),
    }

    fn fails() -> Result<(), Box<dyn Error + Send + Sync>> {
        Err(Failure::from(Leaf::Gone))?;
        Ok(())
    }

    #[test]
    fn display_is_the_trace_and_source_is_none() {
        let e = Failure::from(Leaf::Gone);
        assert_eq!(e.to_string(), e.trace());
        assert_eq!(e.to_string(), "Failure::Cause\n  Leaf::Gone");
        assert!(e.source().is_none());
        assert_eq!(Failure::Data(1).to_string(), "Failure::Data(1)");
    }

    #[test]
    fn boxes_and_downcasts() {
        let err = fails().unwrap_err();
        assert_eq!(err.to_string(), "Failure::Cause\n  Leaf::Gone");
        assert!(matches!(
            err.downcast_ref::<Failure>(),
            Some(Failure::Cause(Leaf::Gone))
        ));
    }

    #[test]
    fn generic() {
        assert_eq!(
            Generic::Wrapped(Leaf::Gone).to_string(),
            "Generic::Wrapped\n  Leaf::Gone"
        );
    }
}

mod tier_priority {
    use foible::Trace;
    use std::fmt;

    #[derive(Debug, Trace)]
    enum Leaf {
        Gone,
    }

    // Implements `Trace` and `Display`: `Trace` wins, so it is the cause.
    #[derive(Debug, Trace)]
    enum Both {
        Gone,
    }
    impl fmt::Display for Both {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("display")
        }
    }

    // Implements `Display` and `Debug`: `Display` wins.
    #[derive(Debug)]
    struct Shown;
    impl fmt::Display for Shown {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("shown")
        }
    }

    #[derive(Debug, Trace)]
    enum Outer {
        Trace(Both),
        Display(Shown),
        Debug(Option<u8>),
        Leaf(Leaf),
    }

    #[test]
    fn trace_then_display_then_debug() {
        assert_eq!(
            Outer::Trace(Both::Gone).trace(),
            "Outer::Trace\n  Both::Gone"
        );
        assert_eq!(Outer::Display(Shown).trace(), "Outer::Display(shown)");
        assert_eq!(Outer::Debug(Some(1)).trace(), "Outer::Debug(Some(1))");
        assert_eq!(Outer::Leaf(Leaf::Gone).trace(), "Outer::Leaf\n  Leaf::Gone");
    }
}
