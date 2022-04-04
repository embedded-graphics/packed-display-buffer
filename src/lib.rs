fn fill_bits(start: usize, end: usize, slice: &mut [u32]) {
    let bits_per_item = u32::BITS as usize;
    let len = end - start;

    dbg!(bits_per_item, len);

    // Partial fill at start

    let start_idx = start / bits_per_item;
    let start_shift = start % bits_per_item;
    // Number of bits in partial fill
    let start_bits = (bits_per_item - start_shift).wrapping_sub(1);

    dbg!(start_bits);

    let start_mask = (2u32 << start_bits).wrapping_sub(1);
    let start_mask = start_mask << start_shift;

    println!("Start: {:032b}", start_mask);

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

        // fill_bits(0, 32, &mut input);
        fill_bits(0, 33, &mut input);
    }
}
