pub(crate) struct StartChunk {
    /// Generated bit mask.
    pub mask: u8,

    /// Bits remaining after those in the start chunk.
    pub remaining: u32,
}

/// Create the bit mask for the partially filled starting block.
///
/// This is required if the top Y coordinate of the block doesn't start at a multiple of
/// `u8::BITS`.
pub(crate) fn start_chunk(start: u32, end: u32) -> StartChunk {
    let len = end - start;
    let num_bits = u8::BITS;

    // Array element to start and end at
    let start_idx = (start / num_bits) as usize;
    let end_idx = (end / num_bits) as usize;

    let start = start % num_bits;

    // Start and end are in the same chunk: compute intra-chunk bit positions
    let end = if start_idx == end_idx {
        end % num_bits
    }
    // Start and end are in different chunks:
    // - Compute start bit position as normal
    // - Clamp end bit position to MSB
    else {
        num_bits - 1
    };

    let num_set_bits = end - start;
    let shift_places = (num_bits - 1) - end;

    let remaining = len - num_set_bits;

    // MSRV: Consider unsafe unchecked_shr when stabilised
    let shifted = i8::MIN >> num_set_bits;
    let shifted = (shifted as u8) >> shift_places;

    StartChunk {
        mask: shifted,
        remaining,
    }
}
