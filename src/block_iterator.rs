static EMPTY: &[u8] = &[];

#[derive(Debug)]
pub struct BlockIterator<'a> {
    pub buffer: &'a [u8],
    pub step_by: usize,
    pub idx: usize,
    pub block_width: usize,
    pub block: u32,
    pub num_blocks: u32,
}

impl<'a> BlockIterator<'a> {
    pub fn empty() -> Self {
        Self {
            buffer: EMPTY,
            step_by: 0,
            idx: 0,
            block_width: 0,
            block: 0,
            num_blocks: 0,
        }
    }
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.block == self.num_blocks {
            return None;
        }

        let block = &self.buffer[self.idx..][..self.block_width];

        self.idx += self.step_by;
        self.block += 1;

        Some(block)
    }
}
