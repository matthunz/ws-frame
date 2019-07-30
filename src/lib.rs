#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std as core;

use byteorder::{BigEndian, ByteOrder};
use core::convert::TryInto;

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

#[derive(Debug, PartialEq)]
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
/// let buf = &[0b10100010, 0b00000011, 0b00000001, 0b00000010, 0b00000011];
/// let mut f = Frame::empty();
/// f.decode(buf);
/// ```
#[derive(Debug, PartialEq)]
pub struct Frame<'buf> {
    /// The head section of a frame.
    pub head: Option<Head>,
    /// An optional mask key to apply over the payload.
    pub mask: Option<[u8; 4]>,
    /// The payload section of a frame.
    ///
    /// An empty payload is represented as `Some(&[])`.
    pub payload: Option<&'buf [u8]>,
}

impl<'buf> Frame<'buf> {
    /// Creates a new `Frame`.
    pub const fn empty() -> Self {
        Self {
            head: None,
            mask: None,
            payload: None,
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
        let len = match second & 0x3F {
            126 => unwrap!(bytes.slice_to(4).map(BigEndian::read_u64)),
            // TODO validate most-sig bit == 0
            127 => unwrap!(bytes.slice_to(8).map(BigEndian::read_u64)),
            l => l as u64,
        };

        if first_bit(second) {
            let mut mask = [0; 4];
            mask.copy_from_slice(unwrap!(bytes.slice_to(4)));
            self.mask = Some(mask);
        }

        self.payload = Some(unwrap!(bytes.slice_to(len.try_into().unwrap())));
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
        assert_eq!(Status::Complete(BYTES.len()), f.decode(BYTES));

        let head = f.head.unwrap();
        assert!(head.finished);
        assert_eq!([false, true, false], head.rsv);
        assert_eq!(3, f.payload.unwrap().len());
    }
}
