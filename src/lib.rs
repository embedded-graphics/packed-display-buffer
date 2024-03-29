// #![no_std]

use active_area::ActiveArea;
use block_iterator::BlockIterator;
use core::{convert::Infallible, marker::PhantomData};
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{raw::RawData, BinaryColor, IntoStorage, PixelColor},
    primitives::Rectangle,
    Pixel,
};
use mask::StartChunk;
// use mask::StartChunk;
// use pixels::Pixels;

mod active_area;
mod block_iterator;
mod byte_direction;
mod mask;
// mod pixels;

// TODO: Remove `N` and calculate from W * H when const features allow us to do so.
#[derive(Debug, PartialEq)]
pub struct PackedBuffer<const W: u32, const H: u32, const N: usize, C> {
    buf: [u8; N],
    area: Rectangle,
    active_area: ActiveArea<W, H>,
    _color: PhantomData<C>,
}

impl<const W: u32, const H: u32, const N: usize, C> PackedBuffer<W, H, N, C>
where
    C: PixelColor + IntoStorage<Storage = u8>,
{
    pub const fn new() -> Self {
        // TODO: Remove this when we can do maths in const generics
        // FIXME: What if H is not a multiple of 8 high?
        if N != (W * H / u8::BITS) as usize {
            panic!("Invariant error: W * H != N")
        }

        assert!(
            C::Raw::BITS_PER_PIXEL <= 8,
            "Only pixel formats with <=8bpp are currently supported"
        );

        Self {
            buf: [0x00u8; N],
            area: Rectangle::new(Point::zero(), Size::new(W, H)),
            active_area: ActiveArea::new(),
            _color: PhantomData,
        }
    }

    /// Set an individual pixel.
    ///
    /// Any given pixels that are outside the display area will be ignored.
    pub fn set_pixel(&mut self, point: Point, color: BinaryColor) {
        // Invariant: requires W * H == N / 8
        if !self.area.contains(point) {
            return;
        }

        self.set_pixel_unchecked(point, color);

        self.active_area.update_from_point(point);
    }

    fn set_pixel_unchecked(&mut self, point: Point, color: BinaryColor) {
        let color = color as u8;
        let Point { x, y } = point;

        let idx = ((y as usize) / u8::BITS as usize * W as usize) + (x as usize);
        let bit = y % u8::BITS as i32;

        let byte = &mut self.buf[idx];

        *byte = *byte & !(1 << bit) | (color << bit)
    }

    // TODO
    // pub fn pixels<'a>(&'a self) -> Pixels<'a> {
    //     Pixels::new(self)
    // }

    /// Create a range representing the indices corresponding to the section of a block in the given
    /// area.
    ///
    /// I.e., the `◀═════════════▶` section in the diagram below:
    ///
    /// ```text
    /// Block
    ///      │ Display════════════════════════════╗
    ///    1 │ ║       Area────────────┐          ║
    ///    2 │ ║       │               │          ║
    ///  * 3 │ ║───────┼◀═════════════▶┼──────────║
    ///    4 │ ║       │               │          ║
    ///    5 │ ║       │               │          ║
    ///    6 │ ║       └───────────────┘          ║
    ///    7 │ ║                                  ║
    ///      │ ╚══════════════════════════════════╝
    /// ```
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
    fn fill_rect(&mut self, rect: &Rectangle, color: C) {
        let color: u8 = color.into_storage();

        // Repeat color throughout the byte
        let color = match C::Raw::BITS_PER_PIXEL {
            1 => {
                if color > 0 {
                    0xff
                } else {
                    0x00
                }
            }
            2 => color | color << 2 | color << 4 | color << 6,
            4 => color | color << 4,
            8 => color,
        };

        let rect = rect.intersection(&self.area);

        let y_start = rect.top_left.y as u32;

        let br = if let Some(br) = rect.bottom_right() {
            br
        } else {
            // Rectangle is zero sized, so don't fill any of the buffer
            return;
        };

        self.active_area.update_from_rect(rect);

        let y_end = br.y as u32;

        let mut block = rect.top_left.y as usize / u8::BITS as usize;

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

        // Partially fill end block if there are any remaining bits
        if remaining > 0 {
            // Merge block underneath last fully filled block with current data
            self.block_range(block, &rect).iter_mut().for_each(|byte| {
                let mask = !(i8::MAX << remaining) as u8;

                // Merge with existing data
                *byte = (*byte & !mask) | (color & mask)
            });
        }
    }

    /// Contiguous fill.
    fn fill_rect_iter<I>(&mut self, rect: &Rectangle, colors: I)
    where
        I: IntoIterator<Item = BinaryColor>,
    {
        // let intersection = rect.intersection(&self.area);

        // // Don't draw anything if the entire rect lies outside the visible area
        // if intersection.is_zero_sized() {
        //     return;
        // }

        // self.active_area.update_from_rect(intersection);

        // // Number of lines above the visible area
        // let row_pre_skip = rect.top_left.y.min(0).abs() as u32;

        // // Number of pixels above the visible area
        // let skip = row_pre_skip * rect.size.width;

        // // Take only the whole rows within the visible area. This filters out rows below the visible
        // // area
        // let take = intersection.size.height * rect.size.width;

        // let colors = colors
        //     .into_iter()
        //     .skip(skip as usize)
        //     .take(take as usize)
        //     .enumerate();

        // let x_range = 0..W as i32;

        // for (idx, color) in colors {
        //     let idx = idx as u32;
        //     let x = rect.top_left.x + (idx % rect.size.width) as i32;
        //     let y = intersection.top_left.y + (idx / rect.size.width) as i32;

        //     // We checked Y range before with .skip().take() on the iterator. We only need to check
        //     // whether the X coordinate is within the visible area here.
        //     if !x_range.contains(&x) {
        //         continue;
        //     }

        //     let pos = Point::new(x, y);

        //     self.set_pixel_unchecked(pos, color);
        // }
        todo!()
    }

    pub fn clear_active_area(&mut self) {
        self.active_area.clear();
    }

    pub fn active_area(&self) -> Rectangle {
        self.active_area.rectangle()
    }

    /// Get an iterator over blocks in the active area of the display buffer.
    ///
    /// Example:
    ///
    /// ```text
    /// Display buffer════════════════════════════════╗
    /// ║                                             ║
    /// ║                                             ║
    /// ║         Active area══════════════╗          ║
    /// ║ .next() ║◀────────&[u8]─────────▶║          ║
    /// ║ .next() ║◀────────&[u8]─────────▶║          ║
    /// ║ .next() ║◀────────&[u8]─────────▶║          ║
    /// ║ .next() ║◀────────&[u8]─────────▶║          ║
    /// ║         ╚════════════════════════╝          ║
    /// ║                                             ║
    /// ╚═════════════════════════════════════════════╝
    /// ```
    ///
    /// This method can be useful for efficiently transferring contiguous parts of the display
    /// buffer to the display hardware.
    pub fn active_blocks<'a>(&'a self) -> BlockIterator<'a> {
        let active_area = self.active_area.rectangle();

        let br = if let Some(br) = active_area.bottom_right() {
            br
        } else {
            return BlockIterator::empty();
        };

        let start_block = active_area.top_left.y as u32 / 8;
        let end_block = br.y as u32 / 8 + 1;

        let start_idx = (start_block * W) + active_area.top_left.x as u32;
        let block_width = active_area.size.width;

        BlockIterator {
            buffer: &self.buf,
            display_width: W as usize,
            buffer_idx: start_idx as usize,
            block_width: block_width as usize,
            num_blocks: end_block - start_block,
            current_block: 0,
        }
    }
}

impl<const W: u32, const H: u32, const N: usize, C> AsRef<[u8]> for PackedBuffer<W, H, N, C> {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}

impl<const W: u32, const H: u32, const N: usize, C> OriginDimensions for PackedBuffer<W, H, N, C> {
    fn size(&self) -> Size {
        self.area.size
    }
}

impl<const W: u32, const H: u32, const N: usize, C> DrawTarget for PackedBuffer<W, H, N, C>
where
    C: PixelColor,
    <<C as PixelColor>::Raw as RawData>::Storage: Into<u8>,
{
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

    // fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
    //     self.fill_rect(area, color);

    //     Ok(())
    // }

    // fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    // where
    //     I: IntoIterator<Item = Self::Color>,
    // {
    //     self.fill_rect_iter(area, colors);

    //     Ok(())
    // }

    // fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
    //     match color {
    //         BinaryColor::Off => self.buf.fill(0x00),
    //         BinaryColor::On => self.buf.fill(0xff),
    //     }

    //     Ok(())
    // }
}

// #[cfg(test)]
#[cfg(feature = "DELETEME")]
mod tests {
    use super::*;
    use embedded_graphics::{
        primitives::{Circle, Primitive, PrimitiveStyle},
        Drawable,
    };
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

    #[test]
    fn active_area_fuzz_contiguous() {
        for _ in 0..10_000 {
            let mut disp_fill = PackedBuffer::<128, 64, 1024>::new();

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

            let visible_image = Rectangle::new(Point::zero(), disp_fill.size()).intersection(&area);

            disp_fill.fill_contiguous(&area, pixels.map(|p| p.1)).ok();

            assert_eq!(disp_fill.active_area.rectangle(), visible_image);
        }
    }

    #[test]
    fn bmp() {
        let mut disp_fill = PackedBuffer::<32, 16, { 32 * 16 / 8 }>::new();
        let mut disp_pixels = PackedBuffer::<32, 16, { 32 * 16 / 8 }>::new();

        let tl = Point::new(-5, -5);

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
    fn active_area_blocks() {
        let mut disp_fill = PackedBuffer::<128, 64, { 128 * 64 / 8 }>::new();

        let tl = Point::new(2, 2);

        let bmp: Bmp<Rgb565> = Bmp::from_slice(include_bytes!("../benches/dvd.bmp"))
            .expect("Failed to load BMP image");

        let pixels = bmp.pixels().map(|p| (p.0, p.1.into()));

        let area = Rectangle::new(tl, bmp.size());

        disp_fill.fill_contiguous(&area, pixels.map(|p| p.1)).ok();

        assert_eq!(disp_fill.active_blocks().count(), 4);

        for block in disp_fill.active_blocks() {
            assert_eq!(block.len(), area.size.width as usize);
        }
    }

    #[test]
    fn block_offset() {
        let mut disp = PackedBuffer::<132, 64, { 132 * 64 / 8 }>::new();

        disp.set_pixel(Point::new(0, 0), BinaryColor::On);
        disp.set_pixel(Point::new(0, 10), BinaryColor::On);
        disp.set_pixel(Point::new(0, 16), BinaryColor::On);
        disp.set_pixel(Point::new(0, 20), BinaryColor::On);

        let mut blocks = disp.active_blocks();

        let block1 = blocks.next().unwrap();
        let block2 = blocks.next().unwrap();
        let block3 = blocks.next().unwrap();

        assert_eq!(blocks.next(), None);

        assert_eq!(block1[0], 0b0000_0001);
        assert_eq!(block2[0], 0b0000_0100);
        assert_eq!(block3[0], 0b0001_0001);
    }
}
