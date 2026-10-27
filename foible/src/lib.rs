use std::fmt::{Debug, Display};

/// A typed error that can render its chain of wrapped errors as a stack-trace
/// style listing, one link per line, outermost first.
pub trait Trace: Debug {
    /// Appends one line per link in the chain, outermost first.
    fn write_trace(&self, out: &mut Vec<String>);

    /// Renders the chain as a multi-line trace. Every line after the first is
    /// indented, like the frames of a stack trace.
    fn trace(&self) -> String {
        let mut lines = Vec::new();
        self.write_trace(&mut lines);
        let mut iter = lines.into_iter();
        let mut out = iter.next().unwrap_or_default();
        for line in iter {
            out.push_str("\n  ");
            out.push_str(&line);
        }
        out
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
        fn write_chain(&self, out: &mut Vec<String>);
    }
    impl<T: Trace + ?Sized> ViaTrace for &&Wrap<'_, T> {
        fn write_chain(&self, out: &mut Vec<String>) {
            self.0.write_trace(out);
        }
    }

    pub trait ViaDisplay {
        fn write_chain(&self, out: &mut Vec<String>);
    }
    impl<T: Display + ?Sized> ViaDisplay for &Wrap<'_, T> {
        fn write_chain(&self, out: &mut Vec<String>) {
            out.push(self.0.to_string());
        }
    }

    pub trait ViaDebug {
        fn write_chain(&self, out: &mut Vec<String>);
    }
    impl<T: Debug + ?Sized> ViaDebug for Wrap<'_, T> {
        fn write_chain(&self, out: &mut Vec<String>) {
            out.push(format!("{:?}", self.0));
        }
    }
}
