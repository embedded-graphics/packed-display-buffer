fn fill_bits(start: usize, end: usize, slice: &mut [u32]) {
    let bits_per_item = u32::BITS as usize;
    let len = end - start;

    dbg!(bits_per_item, len);

    // Partial fill at start

    let start_idx = start / bits_per_item;
    // Number of bits in partial fill
    let start_bits = bits_per_item - (start % bits_per_item);

    dbg!(start_bits);

    // Full fill in middle

    let fill_bits = len.saturating_sub(start_bits);
    // Start index to fill full of ones
    let fill_idx = start_idx + 1;

    let items_filled = fill_bits / bits_per_item;

    // Partial final fill

    let num_final_bits = len
        .saturating_sub(start_bits)
        .saturating_sub(items_filled * bits_per_item);

    dbg!(items_filled, num_final_bits);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full() {
        let mut input = [0u32; 4];

        dbg!(33 % 32);

        // fill_bits(0, 32, &mut input);
        fill_bits(1, 2, &mut input);
    }
}
