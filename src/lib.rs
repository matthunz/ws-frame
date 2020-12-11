#![cfg_attr(not(feature = "std"), no_std)]

//! # ws-frame
//!
//! A library for decoding WebSocket
//! ([RFC6455](https://tools.ietf.org/html/rfc6455)) frames.
//!
//! # Example
//! ```
//! use ws_frame::{Frame, Opcode};
//!
//! let buf = [0b10100010, 0b00000001, 0b00000010];
//! let mut f = Frame::empty();
//!
//! if f.decode(&buf).is_complete() {
//!     if Opcode::Ping == f.head.unwrap().op {
//!         println!("Pong!")
//!     }
//! }
//! ```

#[cfg(feature = "std")]
extern crate std as core;

use byteorder::{BigEndian, ByteOrder};

mod iter;
use iter::Bytes;

macro_rules! unwrap {
    ($e:expr) => {
        match $e {
            Some(t) => t,
            None => return Status::Partial,
        }
    };
}

/// The result of a successful decode pass.
///
/// `Complete` is used when the buffer
/// contained the complete value. `Partial` is used when decoding did not reach
/// the end of the expected value, but no invalid data was found.
#[derive(Debug, PartialEq)]
pub enum Status {
    /// The completed result.
    ///
    /// Contains the amount of bytes decoded.
    Complete(usize),
    /// A partial result.
    Partial,
}

impl Status {
    /// Convenience method to check if status is complete.
    #[inline]
    pub fn is_complete(&self) -> bool {
        match *self {
            Status::Complete(..) => true,
            Status::Partial => false,
        }
    }

    /// Convenience method to check if status is partial.
    #[inline]
    pub fn is_partial(&self) -> bool {
        match *self {
            Status::Complete(..) => false,
            Status::Partial => true,
        }
    }

    /// Convenience method to unwrap a Complete value. Panics if the status is
    /// partial.
    #[inline]
    pub fn unwrap(self) -> usize {
        match self {
            Status::Complete(len) => len,
            Status::Partial => panic!("Tried to unwrap Status::Partial"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    Continue,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
    Reserved,
}

impl From<u8> for Opcode {
    fn from(opcode: u8) -> Opcode {
        match opcode {
            0 => Opcode::Continue,
            1 => Opcode::Text,
            2 => Opcode::Binary,
            8 => Opcode::Close,
            9 => Opcode::Ping,
            10 => Opcode::Pong,
            _ => Opcode::Reserved,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Head {
    pub op: Opcode,
    pub finished: bool,
    pub rsv: [bool; 3],
}

/// A decoded Frame.
///
/// The optional values will be `None` if a decode was not complete, and did
/// not decode the associated property. This allows you to inspect the parts
/// that could be decoded, before reading more.
///
/// # Example
/// ```
/// use ws_frame::Frame;
///
/// let buf = &[0b10000010, 0b00000001];
/// let mut f = Frame::empty();
///
/// if f.decode(buf).is_partial() {
///     match f.head {
///         Some(head) => assert_eq!([false; 3], head.rsv),
///         None => {
///             // read more and decode again
///         }
///     }
/// }
/// ```
#[derive(Debug, PartialEq)]
pub struct Frame {
    /// The head section of a frame.
    pub head: Option<Head>,
    /// An optional mask key to apply over the payload.
    pub mask: Option<[u8; 4]>,
    /// The payload section of a frame.
    ///
    /// An empty payload is represented as `Some(&[])`.
    pub payload_len: Option<u64>,
}

impl<'buf> Frame {
    /// Creates a new `Frame`.
    pub const fn empty() -> Self {
        Self {
            head: None,
            mask: None,
            payload_len: None,
        }
    }
    /// Try to decode a buffer of bytes into this `Frame`.
    pub fn decode(&mut self, buf: &'buf [u8]) -> Status {
        let mut bytes = Bytes::new(buf);

        let first = unwrap!(bytes.next());
        let rsv_bits = first >> 4 & 0x7u8;

        let mut rsv = [false; 3];
        for i in 0..3 {
            rsv[2 - i] = rsv_bits >> i & 0x1u8 == 1u8;
        }

        self.head = Some(Head {
            op: Opcode::from(first & 0xF),
            finished: first_bit(first),
            rsv,
        });

        let second = unwrap!(bytes.next());
        self.payload_len = Some(match second & 0x7F {
            126 => unwrap!(bytes.slice_to(4).map(BigEndian::read_u64)),
            // TODO validate most-sig bit == 0
            127 => unwrap!(bytes.slice_to(8).map(BigEndian::read_u64)),
            l => l as u64,
        });

        if first_bit(second) {
            let mut mask = [0; 4];
            mask.copy_from_slice(unwrap!(bytes.slice_to(4)));
            self.mask = Some(mask);
        }

        Status::Complete(bytes.pos())
    }
}

#[inline]
fn first_bit(byte: u8) -> bool {
    byte >> 7 == 1u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        const BYTES: &[u8] = &[0b10100010, 0b00000011, 0b00000001, 0b00000010, 0b00000011];
        let mut f = Frame::empty();
        let used = f.decode(BYTES);

        let head = f.head.unwrap();
        assert!(head.finished);
        assert_eq!([false, true, false], head.rsv);
        assert_eq!(3, f.payload_len.unwrap());

        assert_eq!(
            Status::Complete(BYTES.len() - f.payload_len.unwrap() as usize),
            used
        );
    }

    #[test]
    fn payload_length() {
        const BYTES: &[u8] = &[0b10100010, 0b01100100];
        let mut f = Frame::empty();
        f.decode(BYTES);

        assert_eq!(f.payload_len, Some(100));
    }
}
