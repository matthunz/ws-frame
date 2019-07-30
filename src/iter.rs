pub struct Bytes<'a> {
    slice: &'a [u8],
    pos: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice, pos: 0 }
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn slice_to(&mut self, end: usize) -> Option<&'a [u8]> {
        let start = self.pos;
        self.pos += end;
        self.slice.get(start..self.pos)
    }
}

impl<'a> Iterator for Bytes<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.slice.len() > self.pos {
            let b = unsafe { *self.slice.get_unchecked(self.pos) };
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }
}
