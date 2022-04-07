#![allow(dead_code)]
#![allow(unused)]

use embedded_graphics_core::{pixelcolor::BinaryColor, primitives::Rectangle};
use mask::build_mask;

mod mask;

pub fn fill_rect(buf: &mut [u8], rect: &Rectangle, color: BinaryColor) {
    // TODO: Intersect rect with display so coords are always positive, or do that somewhere else

    let y_start = rect.top_left.y as u32;

    let y_end = if let Some(br) = rect.bottom_right() {
        br.y
    } else {
        // Rectangle is zero sized, so don't fill any of the buffer
        return;
    } as u32;

    // Test constants, TODO: Make const generic
    let w = 128;
    let h = 64;

    // 64px high display
    let mut mask_buf = [0u8; 8];

    let (chunk_start_idx, mask) = build_mask(&mut mask_buf, y_start, y_end);

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

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::prelude::{Point, PointsIter, Size};
    use rand::{thread_rng, Rng};

    /// Fixed 128 px wide display.
    fn set_pixel(buf: &mut [u8], x: u32, y: u32, value: bool) {
        let value = value as u8;

        let idx = ((y as usize) / 8 * 128 as usize) + (x as usize);
        let bit = y % 8;

        if let Some(byte) = buf.as_mut().get_mut(idx) {
            // Set pixel value in byte
            // Ref this comment https://stackoverflow.com/questions/47981/how-do-you-set-clear-and-toggle-a-single-bit#comment46654671_47990
            *byte = *byte & !(1 << bit) | (value << bit)
        }
    }

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

        for i in 0..10_000 {
            let mut display = [0x00u8; 128 * 8];
            let mut display2 = [0x00u8; 128 * 8];

            let tl = random_point();
            let br = random_point();

            let area = Rectangle::with_corners(tl, br);

            let area = area.intersection(&disp_size);

            fill_rect(&mut display, &area, BinaryColor::On);

            for point in area.points() {
                set_pixel(&mut display2, point.x as u32, point.y as u32, true);
            }

            assert_eq!(display, display2, "{i}: {:?}", area);
        }
    }
}
