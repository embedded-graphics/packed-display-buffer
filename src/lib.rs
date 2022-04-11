#![allow(dead_code)]
#![allow(unused)]

use std::convert::Infallible;

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{Dimensions, OriginDimensions, Point, Size},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
    Pixel,
};
use mask::{Chunk, ShiftSource, StartChunk};

mod mask;

// TODO: Remove `N` and calculate from W * H when const features allow us to do so.
#[derive(Debug, PartialEq)]
pub struct PackedBuffer<T, const W: u32, const H: u32, const N: usize> {
    buf: [T; N],
    area: Rectangle,
}

impl<const W: u32, const H: u32, const N: usize> PackedBuffer<u8, W, H, N> {
    pub const fn new() -> Self {
        // FIXME: Remove this when we can do maths in const generics
        if N != (W * H / u8::BITS) as usize {
            panic!("Invariant error: W * H != N")
        }

        Self {
            buf: [0x00u8; N],
            area: Rectangle::new(Point::zero(), Size::new(W, H)),
        }
    }

    /// Set an individual pixel.
    ///
    /// Any given pixels that are outside the display area will be ignored.
    pub fn set_pixel(&mut self, point: Point, color: BinaryColor) {
        let color = color as u8;

        if !self.area.contains(point) {
            return;
        }

        let Point { x, y } = point;

        let idx = ((y as usize) / u8::BITS as usize * W as usize) + (x as usize);
        let bit = y % u8::BITS as i32;

        if let Some(byte) = self.buf.as_mut().get_mut(idx) {
            // Set pixel value in byte
            // Ref this comment https://stackoverflow.com/questions/47981/how-do-you-set-clear-and-toggle-a-single-bit#comment46654671_47990
            *byte = *byte & !(1 << bit) | (color << bit)
        }
    }

    /// Fill a packed buffer with the given color in the given area.
    ///
    /// The area is clipped to the display dimensions. In conjunction with the `W * H = N` assertion
    /// in [`new`] guarantees that no out of bounds writes can occur.
    fn fill_rect(&mut self, rect: &Rectangle, color: BinaryColor) {
        let rect = rect.intersection(&self.area);

        let Size {
            width: display_width,
            ..
        } = self.area.size;

        let display_width = display_width as usize;

        let y_start = rect.top_left.y as u32;

        let y_end = if let Some(br) = rect.bottom_right() {
            br.y
        } else {
            // Rectangle is zero sized, so don't fill any of the buffer
            return;
        } as u32;

        // X start coordinate
        let x_start = rect.top_left.x as usize;

        let rect_width = rect.size.width as usize;
        let start_block = rect.top_left.y as usize / Chunk::BITS as usize;

        let color = if color.is_on() { 0xff } else { 0x00 };

        let StartChunk {
            mask: first_mask,
            mut remaining,
        } = mask::start_chunk(y_start, y_end);

        let first_block_start_idx = start_block * display_width + x_start;
        let first_block_end_idx = first_block_start_idx + rect_width;

        // Partial fill at top of area; need to merge with existing data
        self.buf
            .get_mut(first_block_start_idx..first_block_end_idx)
            .map(|chunk| {
                chunk
                    .iter_mut()
                    .for_each(|byte| *byte = (*byte & !first_mask) | (color & first_mask));
            });

        // If fill rectangle fits entirely within first block, there's nothing more to do
        if remaining == 0 {
            return;
        }

        // Number of full blocks to fill
        let num_fill = (remaining / Chunk::BITS) as usize;
        // Block underneath start block
        let fill_block_start_idx = first_block_start_idx + display_width;
        let fill_block_end_idx = fill_block_start_idx + (num_fill * display_width);

        // Completely fill middle blocks in the area. We don't need to do any bit twiddling here so
        // it can be optimised by just filling the slice
        for start_x in (fill_block_start_idx..fill_block_end_idx).step_by(display_width) {
            let end_x = start_x + rect_width;

            // Complete overwrite
            self.buf
                .get_mut(start_x..end_x)
                .map(|chunk| chunk.fill(Chunk::MAX));

            remaining -= Chunk::BITS;
        }

        // Partially fill end chunk if there are any remaining bits
        if remaining > 0 {
            let final_block_start_idx = first_block_start_idx + (num_fill + 1) * display_width;
            let final_block_end_idx = final_block_start_idx + rect_width;

            self.buf
                .get_mut(final_block_start_idx..final_block_end_idx)
                .map(|chunk| {
                    chunk.iter_mut().for_each(|byte| {
                        let mask = !(ShiftSource::MAX << remaining) as Chunk;

                        // Merge with existing data
                        *byte = (*byte & !mask) | (color & mask)
                    });
                });
        }
    }
}

impl<T, const W: u32, const H: u32, const N: usize> OriginDimensions for PackedBuffer<T, W, H, N> {
    fn size(&self) -> Size {
        self.area.size
    }
}

impl<const W: u32, const H: u32, const N: usize> DrawTarget for PackedBuffer<u8, W, H, N> {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| self.set_pixel(pos, color));

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_rect(area, color);

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        match color {
            BinaryColor::Off => self.buf.fill(0x00),
            BinaryColor::On => self.buf.fill(0xff),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::{
        geometry::{Point, Size},
        primitives::PointsIter,
    };
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

        for i in 0..10_000 {
            let mut disp_fill = PackedBuffer::<u8, 128, 64, 1024>::new();
            let mut disp_pixels = PackedBuffer::<u8, 128, 64, 1024>::new();

            let tl = random_point();
            let br = random_point();

            let area = Rectangle::with_corners(tl, br);

            // Fill pixel by pixel
            for point in area.points() {
                disp_pixels.set_pixel(point, BinaryColor::On);
            }

            disp_fill.fill_solid(&area, BinaryColor::On);

            assert_eq!(disp_fill, disp_pixels, "{i}: {:?}", area);
        }
    }
}
