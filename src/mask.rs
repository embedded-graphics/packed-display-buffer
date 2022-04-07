type Chunk = u8;
type ShiftSource = i8;

/// Returns (remaining number of bits to pack, next index)
pub fn start_chunk(buf: &mut [Chunk], start: u32, end: u32) -> (u32, usize) {
    let len = end - start;
    let num_bits = Chunk::BITS;

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
    let shifted = ShiftSource::MIN >> num_set_bits;
    let shifted = (shifted as Chunk) >> shift_places;

    // buf[start_idx] = shifted;
    buf.get_mut(start_idx).map(|b| *b = shifted);

    (remaining, start_idx)
}

/// Returns section of buffer that was actually modified
pub fn build_mask(buf: &mut [Chunk], start: u32, end: u32) -> (usize, &[Chunk]) {
    let (remaining, fill_start_idx) = start_chunk(buf, start, end);

    if remaining == 0 {
        return (fill_start_idx, &buf[fill_start_idx..=fill_start_idx]);
    }

    let fill_start_idx = fill_start_idx + 1;

    // Fully fill chunks
    let (remaining, mut final_idx) = {
        let num_fill = remaining / Chunk::BITS;

        buf[fill_start_idx..][..num_fill as usize].fill(Chunk::MAX);

        let remaining = remaining - (num_fill * Chunk::BITS);

        (remaining, fill_start_idx + num_fill as usize)
    };

    // Partially fill end chunk if there are any remaining bits
    if remaining > 0 {
        // MSRV: Consider unsafe unchecked_shl when stabilised
        // buf[final_idx] = !(ShiftSource::MAX << remaining) as Chunk;
        buf.get_mut(final_idx)
            .map(|b| *b = !(ShiftSource::MAX << remaining) as Chunk);
    } else {
        final_idx -= 1
    }

    ((fill_start_idx - 1), &buf[(fill_start_idx - 1)..=final_idx])
}
