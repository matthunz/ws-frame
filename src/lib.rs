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
        Status::Partial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        const BYTES: &[u8] = &[];
        let mut f = Frame::empty();
        assert_eq!(Status::Partial, f.decode(BYTES));
    }
}
