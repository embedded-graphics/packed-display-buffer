use embedded_graphics_core::{
    geometry::Point,
    pixelcolor::{raw::RawData, IntoStorage, PixelColor},
};

pub trait ByteDirection {
    fn set_pixel<C>(p: Point, color: C, buf: &mut [u8])
    where
        C: PixelColor + IntoStorage<Storage = u8>;
}

pub struct VerticalByte<const W: usize>;

impl<const W: usize> ByteDirection for VerticalByte<W> {
    #[inline]
    fn set_pixel<C>(p: Point, color: C, buf: &mut [u8])
    where
        C: PixelColor + IntoStorage<Storage = u8>,
    {
        let color: u8 = color.into_storage();

        let x = p.x.unsigned_abs();
        let y = p.y.unsigned_abs();

        let bpp = C::Raw::BITS_PER_PIXEL as u32;

        let bit_y = y * bpp;
        let byte_y = bit_y as usize / 8;
        let index_in_byte = bit_y % 8 / bpp;

        let pixel_mask = 2u8.pow(bpp) - 1;
        let shift = index_in_byte * bpp;
        let mask = pixel_mask << shift;
        let color = color << shift;

        let byte_index = (byte_y * W) + x as usize;

        buf[byte_index] = buf[byte_index] & !mask | color;
    }
}

pub struct HorizontalByte<const W: usize>;

impl<const W: usize> ByteDirection for HorizontalByte<W> {
    fn set_pixel<C>(p: Point, color: C, buf: &mut [u8])
    where
        C: PixelColor + IntoStorage<Storage = u8>,
    {
        let color: u8 = color.into_storage();

        let x = p.x.unsigned_abs();
        let y = p.y.unsigned_abs();

        let bpp = C::Raw::BITS_PER_PIXEL as u32;
        let bytes_per_row = W as u32 * bpp / 8;

        let bit_x = x * bpp;
        let byte_x = bit_x / 8;
        let index_in_byte = (7 - bit_x) % 8 / bpp;

        let pixel_mask = 2u8.pow(bpp) - 1;
        let shift = index_in_byte * bpp;
        let mask = pixel_mask << shift;
        let color = color << shift;

        let byte_index = (byte_x + y * bytes_per_row) as usize;

        buf[byte_index] = buf[byte_index] & !mask | color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::pixelcolor::Gray2;

    #[test]
    fn set_vertical() {
        let mut buf = [0u8; 8];

        VerticalByte::<4>::set_pixel::<Gray2>(Point::new(3, 0), Gray2::new(3), &mut buf);

        // for i in buf {
        //     println!("{:08b}", i);
        // }

        //

        let mut buf = [0u8; 8];

        HorizontalByte::<4>::set_pixel::<Gray2>(Point::new(3, 0), Gray2::new(3), &mut buf);

        for i in buf {
            println!("{:08b}", i);
        }
    }
}
