#[derive(Debug, PartialEq)]
pub enum Status {
    Complete(usize),
    Partial,
}

pub struct Head {
    pub finished: bool,
    pub rsv: [bool; 3],
}

#[derive(Default)]
pub struct Frame {
    pub head: Option<Head>,
}

impl Frame {
    pub const fn empty() -> Self {
        Self { head: None }
    }
    pub fn decode(&mut self, buf: &[u8]) -> Status {
        let mut bytes = buf.iter();
        if let Some(byte) = bytes.next() {
            let finished = byte >> 7 == 1u8;
            let rsv_bits = byte >> 4 & 0x7u8;

            let mut rsv = [false; 3];
            for i in 0..3 {
                rsv[2 - i] = rsv_bits >> i & 0x1u8 == 1u8;
            }

            self.head = Some(Head { finished, rsv })
        }
        Status::Partial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        const BYTES: &[u8] = &[0b10100010];
        let mut f = Frame::empty();
        assert_eq!(Status::Partial, f.decode(BYTES));

        let head = f.head.unwrap();
        assert!(head.finished);
        assert_eq!([false, true, false], head.rsv);
    }
}
