//! Lightweight typed error chaining.
//!
//! `#[derive(Trace)]` on an error enum gives it a [`trace()`](Trace::trace)
//! that renders the chain of wrapped errors, one link per line, outermost
//! first, with no `Display` or `std::error::Error` boilerplate and no loss of
//! the typed enum.
//!
//! ```
//! use foible::Trace;
//!
//! #[derive(Debug, Trace)]
//! enum DbError {
//!     Timeout { after_ms: u32 },
//! }
//!
//! #[derive(Debug, Trace)]
//! enum MyError {
//!     // `#[from]` also generates `impl From<DbError> for MyError`.
//!     #[from]
//!     Db(DbError),
//!     NotFound(&'static str),
//! }
//!
//! let err = MyError::from(DbError::Timeout { after_ms: 30 });
//! assert_eq!(err.trace(), "MyError::Db\n  DbError::Timeout(after_ms: 30)");
//! assert_eq!(MyError::NotFound("users").trace(), "MyError::NotFound(users)");
//! ```
//!
//! Each field is either the cause or data. A field whose type implements
//! [`Trace`] is the cause and is rendered on the next line; at most one is
//! allowed per variant. Every other field is data, written inline using
//! `Display`, falling back to `Debug`.
//!
//! Add `#[trace(error)]` to the enum to also derive `Display` (the full trace)
//! and `std::error::Error`, for `?` into `Box<dyn Error>` and similar
//! boundaries.
//!
//! See the [README](https://github.com/cleavera/foible#readme) for the full
//! rules, including generics and `#[from]`.

use std::fmt::{self, Debug, Display};

/// A typed error that can render its chain of wrapped errors as a stack-trace
/// style listing, one link per line, outermost first.
///
/// Prefer `#[derive(Trace)]`. Implementing the trait by hand is an escape hatch
/// for shapes the derive doesn't cover, such as an error with several causes
/// (the derive allows at most one `Trace` field per variant). The trait
/// requires `Debug`, and [`write_trace`](Trace::write_trace) should follow the
/// same conventions as the derive: write this link's line, then `"\n  "`
/// followed by the cause's trace.
///
/// A hand-written type is picked up as the cause by derived enums that wrap it.
/// The format has no way to tell siblings from nesting, so what the output
/// looks like for several causes is up to the implementation, for example
/// numbering them:
///
/// ```
/// use std::fmt;
/// use foible::Trace;
///
/// #[derive(Debug, Trace)]
/// enum Failure {
///     Timeout { after_ms: u32 },
///     Refused,
/// }
///
/// #[derive(Debug)]
/// struct Many(Vec<Failure>);
///
/// impl Trace for Many {
///     fn write_trace(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "Many({})", self.0.len())?;
///         for (i, failure) in self.0.iter().enumerate() {
///             write!(f, "\n  [{i}] ")?;
///             failure.write_trace(f)?;
///         }
///         Ok(())
///     }
/// }
///
/// #[derive(Debug, Trace)]
/// enum Request {
///     Failed(Many),
/// }
///
/// let err = Request::Failed(Many(vec![
///     Failure::Timeout { after_ms: 30 },
///     Failure::Refused,
/// ]));
/// assert_eq!(
///     err.trace(),
///     "Request::Failed\n  Many(2)\n  [0] Failure::Timeout(after_ms: 30)\n  [1] Failure::Refused"
/// );
/// ```
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

// Compiles and runs the README's examples as doctests. `cfg(doctest)` keeps
// the path out of normal builds, so the published crate doesn't need it.
#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
struct ReadmeDoctests;

/// Used by the derive macro. Not part of the public API.
///
/// Autoref specialization: for every field, method lookup tries `Trace`, then
/// `Display`, then `Debug`, so the first one the concrete type implements wins.
/// A `Trace` field is the variant's cause and is written on the following
/// line; any other field is data and is written inline.
#[doc(hidden)]
pub mod __private {
    use super::*;

    pub struct Wrap<'a, T: ?Sized>(pub &'a T);

    /// Writes the inline `(a; name: b)` list, opening the parenthesis lazily so
    /// variants without data fields print no parentheses.
    #[derive(Default)]
    pub struct DataSep {
        opened: bool,
    }

    impl DataSep {
        pub fn begin(&mut self, f: &mut fmt::Formatter<'_>, name: Option<&str>) -> fmt::Result {
            f.write_str(if self.opened { "; " } else { "(" })?;
            self.opened = true;
            match name {
                Some(name) => write!(f, "{name}: "),
                None => Ok(()),
            }
        }

        pub fn finish(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.opened {
                f.write_str(")")
            } else {
                Ok(())
            }
        }
    }

    // Typestate that turns a second `Trace` field in one variant into a type
    // error: only the first can consume the `CauseSlot`.
    pub struct CauseSlot;
    pub struct CauseAlreadyTaken;

    pub trait ViaTrace {
        fn write_data(
            &self,
            f: &mut fmt::Formatter<'_>,
            sep: &mut DataSep,
            name: Option<&str>,
        ) -> fmt::Result;
        fn write_child(
            &self,
            f: &mut fmt::Formatter<'_>,
            slot: CauseSlot,
        ) -> Result<CauseAlreadyTaken, fmt::Error>;
    }
    impl<T: Trace + ?Sized> ViaTrace for &&Wrap<'_, T> {
        fn write_data(
            &self,
            _: &mut fmt::Formatter<'_>,
            _: &mut DataSep,
            _: Option<&str>,
        ) -> fmt::Result {
            Ok(())
        }
        fn write_child(
            &self,
            f: &mut fmt::Formatter<'_>,
            _: CauseSlot,
        ) -> Result<CauseAlreadyTaken, fmt::Error> {
            f.write_str("\n  ")?;
            self.0.write_trace(f)?;
            Ok(CauseAlreadyTaken)
        }
    }

    pub trait ViaDisplay {
        fn write_data(
            &self,
            f: &mut fmt::Formatter<'_>,
            sep: &mut DataSep,
            name: Option<&str>,
        ) -> fmt::Result;
        fn write_child<S>(&self, f: &mut fmt::Formatter<'_>, slot: S) -> Result<S, fmt::Error>;
    }
    impl<T: Display + ?Sized> ViaDisplay for &Wrap<'_, T> {
        fn write_data(
            &self,
            f: &mut fmt::Formatter<'_>,
            sep: &mut DataSep,
            name: Option<&str>,
        ) -> fmt::Result {
            sep.begin(f, name)?;
            Display::fmt(self.0, f)
        }
        fn write_child<S>(&self, _: &mut fmt::Formatter<'_>, slot: S) -> Result<S, fmt::Error> {
            Ok(slot)
        }
    }

    pub trait ViaDebug {
        fn write_data(
            &self,
            f: &mut fmt::Formatter<'_>,
            sep: &mut DataSep,
            name: Option<&str>,
        ) -> fmt::Result;
        fn write_child<S>(&self, f: &mut fmt::Formatter<'_>, slot: S) -> Result<S, fmt::Error>;
    }
    impl<T: Debug + ?Sized> ViaDebug for Wrap<'_, T> {
        fn write_data(
            &self,
            f: &mut fmt::Formatter<'_>,
            sep: &mut DataSep,
            name: Option<&str>,
        ) -> fmt::Result {
            sep.begin(f, name)?;
            Debug::fmt(self.0, f)
        }
        fn write_child<S>(&self, _: &mut fmt::Formatter<'_>, slot: S) -> Result<S, fmt::Error> {
            Ok(slot)
        }
    }
}
