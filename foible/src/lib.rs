use std::fmt::{self, Debug, Display};

/// A typed error that can render its chain of wrapped errors as a stack-trace
/// style listing, one link per line, outermost first.
pub trait Trace: Debug {
    /// Writes this link, then (indented on a new line) the trace of the error
    /// it wraps, if any.
    fn write_trace(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Renders the chain as a multi-line trace. Every line after the first is
    /// indented, like the frames of a stack trace.
    fn trace(&self) -> String {
        struct Adapter<'a, T: ?Sized>(&'a T);
        impl<T: Trace + ?Sized> Display for Adapter<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.write_trace(f)
            }
        }
        Adapter(self).to_string()
    }
}

pub use foible_derive::Trace;

/// Used by the derive macro. Not part of the public API.
///
/// Autoref specialization: method lookup tries `Trace`, then `Display`,
/// then `Debug`, so the first one the concrete type implements wins.
#[doc(hidden)]
pub mod __private {
    use super::*;

    pub struct Wrap<'a, T: ?Sized>(pub &'a T);

    pub trait ViaTrace {
        fn write_chain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    }
    impl<T: Trace + ?Sized> ViaTrace for &&Wrap<'_, T> {
        fn write_chain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.write_trace(f)
        }
    }

    pub trait ViaDisplay {
        fn write_chain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    }
    impl<T: Display + ?Sized> ViaDisplay for &Wrap<'_, T> {
        fn write_chain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Display::fmt(self.0, f)
        }
    }

    pub trait ViaDebug {
        fn write_chain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    }
    impl<T: Debug + ?Sized> ViaDebug for Wrap<'_, T> {
        fn write_chain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Debug::fmt(self.0, f)
        }
    }
}
