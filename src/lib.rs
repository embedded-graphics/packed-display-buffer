use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
    Pixel,
};
use mask::StartChunk;
use std::convert::Infallible;

mod mask;

// TODO: Remove `N` and calculate from W * H when const features allow us to do so.
#[derive(Debug, PartialEq)]
pub struct PackedBuffer<const W: u32, const H: u32, const N: usize> {
    buf: [u8; N],
    area: Rectangle,
}

impl<const W: u32, const H: u32, const N: usize> PackedBuffer<W, H, N> {
    pub const fn new() -> Self {
        // TODO: Remove this when we can do maths in const generics
        // FIXME: What if H is not a multiple of 8 high?
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

    /// Create a range representing the indices corresponding to the section of a block in the given
    /// area.
    fn block_range(&mut self, block_idx: usize, rect: &Rectangle) -> &mut [u8] {
        let rect_width = rect.size.width as usize;
        let start_x = rect.top_left.x as usize;
        let start_idx = block_idx * self.area.size.width as usize + start_x;

        let range = start_idx..(start_idx + rect_width);

        &mut self.buf[range]
    }

    /// Fill a packed buffer with the given color in the given area.
    ///
    /// The area is clipped to the display dimensions. In conjunction with the `W * H = N` assertion
    /// in [`new`] guarantees that no out of bounds writes can occur.
    fn fill_rect(&mut self, rect: &Rectangle, color: BinaryColor) {
        let rect = rect.intersection(&self.area);

        let y_start = rect.top_left.y as u32;

        let y_end = if let Some(br) = rect.bottom_right() {
            br.y
        } else {
            // Rectangle is zero sized, so don't fill any of the buffer
            return;
        } as u32;

        let mut block = rect.top_left.y as usize / u8::BITS as usize;

        let color = if color.is_on() { 0xff } else { 0x00 };

        let StartChunk {
            mask: first_mask,
            mut remaining,
        } = mask::start_chunk(y_start, y_end);

        // If the area covers part of a block, merge the top row with existing data in the block
        self.block_range(block, &rect)
            .iter_mut()
            .for_each(|byte| *byte = (*byte & !first_mask) | (color & first_mask));

        // If fill rectangle fits entirely within first block, there's nothing more to do
        if remaining == 0 {
            return;
        }

        // Start filling blocks below the starting partial block
        block += 1;

        // Completely fill middle blocks in the area. We don't need to do any bit twiddling here so
        // it can be optimised by just filling the slice
        while remaining >= u8::BITS {
            // Completely overwrite any existing value
            self.block_range(block, &rect).fill(u8::MAX);

            block += 1;
            remaining -= u8::BITS;
        }

        // Partially fill end chunk if there are any remaining bits
        if remaining > 0 {
            // Fill block underneath last fully filled block
            self.block_range(block, &rect).iter_mut().for_each(|byte| {
                let mask = !(i8::MAX << remaining) as u8;

                // Merge with existing data
                *byte = (*byte & !mask) | (color & mask)
            });
        }
    }

    fn fill_rect_iter<I>(&mut self, rect: &Rectangle, colors: I)
    where
        I: IntoIterator<Item = BinaryColor>,
    {
        let mut colors = colors.into_iter();

        let rect = rect.intersection(&self.area);

        let y_start = rect.top_left.y as u32;

        let y_end = if let Some(br) = rect.bottom_right() {
            br.y
        } else {
            // Rectangle is zero sized, so don't fill any of the buffer
            return;
        } as u32;

        // For partial block
        // ---
        let start_block = y_start / u8::BITS;
        let start_block_bit_idx = y_start % u8::BITS;
        // Count how many starting rows
        let starting_rows = u8::BITS - start_block_bit_idx;
        // Number of bits to use in the starting partial block
        let starting_bits = starting_rows * rect.size.width;

        let start_block = self.block_range(start_block as usize, &rect);

        let starting_pixels = colors.take(starting_bits as usize);

        // Create repeating iterator over starting block bytes
        for (idx, color) in starting_pixels.enumerate() {
            let byte = idx % rect.size.width as usize;

            let bit = idx / rect.size.width as usize;
            let bit = start_block_bit_idx as usize + bit;

            let color = color as u8;

            start_block[byte] = start_block[byte] & !(1 << bit) | (color << bit);
        }

        colors.for_each(|c| {
            //
        });

        // Take W * starting rows pixels from `colors`
        // Zip the two together with an index
        // Calculate offset from start of byte
        // Calculate bit index based on start offset + (index / W)
        // Merge bits
        // ---
        // For full blocks
        // ---
        // For each block
        // Create repeating iterator over block bytes
        // Take W * u8::BITS pixels
        // Zip the two with an index
        // Calculate bit index based on (index / W)
        // This should work for the partial last block too

        // todo!();
    }
}

impl<const W: u32, const H: u32, const N: usize> OriginDimensions for PackedBuffer<W, H, N> {
    fn size(&self) -> Size {
        self.area.size
    }
}

impl<const W: u32, const H: u32, const N: usize> DrawTarget for PackedBuffer<W, H, N> {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        // NOTE: Don't need to filter here as `set_pixel` already does bounds checking
        pixels
            .into_iter()
            .for_each(|Pixel(pos, color)| self.set_pixel(pos, color));

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_rect(area, color);

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.fill_rect_iter(area, colors);

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
    use embedded_graphics_core::{geometry::Point, pixelcolor::Rgb565, primitives::PointsIter};
    use rand::{thread_rng, Rng};
    use tinybmp::Bmp;

    fn random_point() -> Point {
        let mut rng = thread_rng();

        let x: i32 = rng.gen_range(-256..256);
        let y: i32 = rng.gen_range(-256..256);

        Point::new(x, y)
    }

    #[test]
    fn fuzz_fill() {
        for i in 0..100_000 {
            let mut disp_fill = PackedBuffer::<128, 64, 1024>::new();
            let mut disp_pixels = PackedBuffer::<128, 64, 1024>::new();

            let tl = random_point();
            let br = random_point();

            let area = Rectangle::with_corners(tl, br);

            // Fill pixel by pixel
            for point in area.points() {
                disp_pixels.set_pixel(point, BinaryColor::On);
            }

            disp_fill.fill_solid(&area, BinaryColor::On).ok();

            assert_eq!(disp_fill, disp_pixels, "{i}: {:?}", area);
        }
    }

    #[test]
    fn contiguous_zero() {
        let mut disp_fill = PackedBuffer::<128, 64, 1024>::new();
        let mut disp_pixels = PackedBuffer::<128, 64, 1024>::new();

        let tl = Point::zero();

        let bmp: Bmp<Rgb565> = Bmp::from_slice(include_bytes!("../benches/dvd.bmp"))
            .expect("Failed to load BMP image");

        let pixels = bmp.pixels().map(|p| (p.0, p.1.into()));

        let area = Rectangle::new(tl, bmp.size());

        // Fill pixel by pixel
        for (point, color) in pixels.clone() {
            disp_pixels.set_pixel(point + area.top_left, color);
        }

        disp_fill.fill_contiguous(&area, pixels.map(|p| p.1)).ok();

        assert_eq!(disp_fill, disp_pixels, "{:?}", area);
    }

    #[test]
    fn fuzz_contiguous() {
        for i in 0..10_000 {
            let mut disp_fill = PackedBuffer::<128, 64, 1024>::new();
            let mut disp_pixels = PackedBuffer::<128, 64, 1024>::new();

            let tl = {
                let mut rng = thread_rng();

                let x: i32 = rng.gen_range(-60..130);
                let y: i32 = rng.gen_range(-30..70);

                Point::new(x, y)
            };

            let bmp: Bmp<Rgb565> = Bmp::from_slice(include_bytes!("../benches/dvd.bmp"))
                .expect("Failed to load BMP image");

            let pixels = bmp.pixels().map(|p| (p.0, p.1.into()));

            let area = Rectangle::new(tl, bmp.size());

            // Fill pixel by pixel
            for (point, color) in pixels.clone() {
                disp_pixels.set_pixel(point + area.top_left, color);
            }

            disp_fill.fill_contiguous(&area, pixels.map(|p| p.1)).ok();

            assert_eq!(disp_fill, disp_pixels, "{i}: {:?}", area);
        }
    }
}
