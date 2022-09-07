use crate::PackedBuffer;
use embedded_graphics_core::Pixel;
use embedded_graphics_core::{geometry::Point, pixelcolor::BinaryColor};

pub struct Pixels<'a> {
    buf: &'a [u8],
    point: Point,
    width: usize,
}

impl<'a> Pixels<'a> {
    pub fn new<const W: u32, const H: u32, const N: usize>(fb: &'a PackedBuffer<W, H, N>) -> Self {
        Self {
            buf: &fb.buf,
            point: Point::zero(),
            width: W as usize,
        }
    }
}

impl<'a> Iterator for Pixels<'a> {
    type Item = Pixel<BinaryColor>;

    fn next(&mut self) -> Option<Self::Item> {
        let Point { x, y } = self.point;

        let idx = ((y as usize) / u8::BITS as usize * self.width) + (x as usize);
        let bit = y % u8::BITS as i32;

        let byte = self.buf.get(idx)?;

        let bit = (*byte >> bit) & 1;

        let c = if bit == 1 {
            BinaryColor::On
        } else {
            BinaryColor::Off
        };

        let out = Some(Pixel(self.point, c));

        self.point.x += 1;

        if self.point.x as usize >= self.width {
            self.point.x = 0;
            self.point.y += 1;
        }

        out
    }
}
