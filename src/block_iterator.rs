static EMPTY: &[u8] = &[];

/// An iterator over horizontal blocks in the buffer.
///
/// A block is a horizontal section of the screen 8 bits tall.
#[derive(Debug)]
pub struct BlockIterator<'a> {
    /// Complete display buffer.
    pub buffer: &'a [u8],

    /// Width of the display.
    pub display_width: usize,

    /// Current block start index into the display buffer.
    pub buffer_idx: usize,

    /// Block width, must be less than or equal to `display_width`.
    ///
    /// Used to return a subsection of an entire display-wide block.
    pub block_width: usize,

    /// Current block counter.
    pub current_block: u32,

    /// Block counter limit.
    pub num_blocks: u32,
}

impl<'a> BlockIterator<'a> {
    pub fn empty() -> Self {
        Self {
            buffer: EMPTY,
            display_width: 0,
            buffer_idx: 0,
            block_width: 0,
            current_block: 0,
            num_blocks: 0,
        }
    }
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_block == self.num_blocks {
            return None;
        }

        let block = &self.buffer[self.buffer_idx..][..self.block_width];

        self.buffer_idx += self.display_width;
        self.current_block += 1;

        Some(block)
    }
}
