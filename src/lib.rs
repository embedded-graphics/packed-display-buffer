#![allow(dead_code)]
#![allow(unused)]

// /// Returns (remaining number of bits to pack, next index)
// fn start_chunk(buf: &mut [u32], start: u32, end: u32) -> (u32, usize) {
//     // let mut start = 3;
//     // let mut end = 66;
//     let len = end - start;
//     let num_bits = u32::BITS;

//     // Array element to start and end at
//     let start_idx = (start / num_bits) as usize;
//     let end_idx = (end / num_bits) as usize;

//     dbg!(start_idx, end_idx);

//     let start = start % num_bits;

//     // Case 1.
//     // Start and end are in the same chunk (i.e. start / num_bits == end / num_bits)
//     // --> Compute chunk bit positions with start = start % num_bits and end = end % num_bits
//     let end = if start_idx == end_idx {
//         end % num_bits
//     } else {
//         // Case 2.
//         // Start and end are in different chunks
//         // --> Compute start bit position as normal
//         // --> Compute end bit position as end = num_bits
//         num_bits - 1
//     };

//     dbg!(start, end);

//     let num_set_bits = end - start;
//     let shift_places = (num_bits - 1) - end;

//     let remaining = len - num_set_bits;

//     dbg!(num_set_bits, shift_places, remaining);

//     let shifted = i32::MIN >> num_set_bits;

//     // println!("{:032b}", shifted);

//     let shifted = (shifted as u32) >> shift_places;

//     buf[start_idx] = shifted;

//     (remaining, start_idx + 1)
// }

// pub fn fill(buf: &mut [u32], start: u32, end: u32) {
//     let (remaining, fill_start_idx) = start_chunk(buf, start, end);

//     if remaining == 0 {
//         return;
//     }

//     // Fully fill chunks
//     let (remaining, final_idx) = {
//         let num_fill = remaining / u32::BITS;

//         buf[fill_start_idx..][..num_fill as usize].fill(u32::MAX);

//         let remaining = remaining - (num_fill * u32::BITS);

//         (remaining, fill_start_idx + num_fill as usize)
//     };

//     dbg!(remaining);

//     // Partially fill end chunk
//     if remaining > 0 {
//         buf[final_idx] = !(i32::MAX << remaining) as u32;
//     }
// }

// fn main() {
//     let mut buf = [0u32; 8];

//     fill(&mut buf, 1, 127);

//     for (i, word) in buf.iter().enumerate() {
//         println!("{i}: {word:032b}");
//     }
// }

use embedded_graphics_core::{pixelcolor::BinaryColor, primitives::Rectangle};
use mask::build_mask;

mod mask;

pub fn fill_rect(buf: &mut [u8], rect: &Rectangle, color: BinaryColor) {
    // TODO: Intersect rect with display so coords are always positive

    let y_start = rect.top_left.y as u32;

    let y_end = if let Some(br) = rect.bottom_right() {
        br.y
    } else {
        // Rectangle is zero sized, so don't fill any of the buffer
        return;
    } as u32;

    // dbg!(y_start, y_end);

    // Test crap
    let w = 128;
    let h = 64;

    // 64px high display
    let mut mask_buf = [0u8; 8];

    let (chunk_start_idx, mask) = build_mask(&mut mask_buf, y_start, y_end);

    // dbg!(chunk_start_idx, mask);

    let start_x = rect.top_left.x as u32;

    let color = if color.is_on() { 0xff } else { 0x00 };

    for x in start_x..(start_x + rect.size.width) {
        for (chunk, mask) in mask.iter().enumerate() {
            let offset = x as usize + ((chunk + chunk_start_idx) * w);

            // dbg!(offset, chunk, mask);

            let current = buf[offset];

            buf[offset] = (current & !mask) | (color & mask)
        }
    }
}

/// Test only. Fixed 128 px wide display.
pub fn set_pixel(buf: &mut [u8], x: u32, y: u32, value: bool) {
    let value = value as u8;

    let idx = ((y as usize) / 8 * 128 as usize) + (x as usize);
    let bit = y % 8;

    if let Some(byte) = buf.as_mut().get_mut(idx) {
        // Set pixel value in byte
        // Ref this comment https://stackoverflow.com/questions/47981/how-do-you-set-clear-and-toggle-a-single-bit#comment46654671_47990
        *byte = *byte & !(1 << bit) | (value << bit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::prelude::{Point, PointsIter, Size};
    use rand::{thread_rng, Rng};

    fn random_point() -> Point {
        let mut rng = thread_rng();

        let x: i32 = rng.gen_range(-256..256);
        let y: i32 = rng.gen_range(-256..256);

        Point::new(x, y)
    }

    #[test]
    fn fuzz() {
        let disp_size = Rectangle::new(Point::zero(), Size::new(128, 64));

        // let mut bads = Vec::new();

        for i in 0..100_000 {
            let mut display = [0x00u8; 128 * 8];
            let mut display2 = [0x00u8; 128 * 8];

            let tl = random_point();
            let br = random_point();

            let area = Rectangle::with_corners(tl, br);

            let area = area.intersection(&disp_size);

            // println!("{i}: {area:?}");

            fill_rect(&mut display, &area, BinaryColor::On);

            for point in area.points() {
                set_pixel(&mut display2, point.x as u32, point.y as u32, true);
            }

            assert_eq!(display, display2, "{i}: {:?}", area);

            // if display != display2 {
            //     panic!("{i}: {:?}", area);

            //     bads.push((i, area));
            // }

            // if !bads.is_empty() {
            //     panic!("{:#?}", bads);
            // }
        }
    }
}
