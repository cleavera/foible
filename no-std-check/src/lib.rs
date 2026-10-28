//! Compiled for a bare-metal target in CI. Nothing here may use `std` or
//! `alloc`.
#![cfg_attr(not(test), no_std)]

use core::fmt::{self, Write};
use foible::Trace;

#[derive(Debug, Trace)]
pub enum Leaf {
    Gone,
}

#[derive(Debug, Trace)]
pub enum Wrapped<E: Trace> {
    Cause(E),
    Data(u8, &'static str),
    Named { id: u8 },
}

#[derive(Debug, Trace)]
pub enum Outer {
    #[from]
    Wrapped(Wrapped<Leaf>),
}

/// A fixed-size buffer standing in for a UART or log sink.
pub struct Buf {
    bytes: [u8; 128],
    len: usize,
}

impl Buf {
    pub fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl Default for Buf {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for Buf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        let slot = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        slot.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Renders a trace without allocating.
pub fn render(err: &impl Trace, buf: &mut Buf) -> fmt::Result {
    write!(buf, "{}", err.display_trace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_without_alloc() {
        let mut buf = Buf::new();
        render(&Outer::from(Wrapped::Cause(Leaf::Gone)), &mut buf).unwrap();
        assert_eq!(
            buf.as_str(),
            "Outer::Wrapped\n  Wrapped::Cause\n  Leaf::Gone"
        );

        let mut buf = Buf::new();
        render(&Wrapped::<Leaf>::Data(1, "x"), &mut buf).unwrap();
        assert_eq!(buf.as_str(), "Wrapped::Data(1; x)");
    }
}
