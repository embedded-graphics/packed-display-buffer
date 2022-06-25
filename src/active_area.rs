use embedded_graphics_core::{geometry::Point, primitives::Rectangle};

#[derive(Debug, PartialEq)]
pub struct ActiveArea<const W: u32, const H: u32> {
    min: Point,
    max: Point,
    touched: bool,
}

impl<const W: u32, const H: u32> ActiveArea<W, H> {
    pub const fn new() -> Self {
        Self {
            min: Point::new(W.saturating_sub(1) as i32, H.saturating_sub(1) as i32),
            max: Point::zero(),
            touched: false,
        }
    }

    pub fn update_from_point(&mut self, point: Point) {
        self.touched = true;

        self.min = self.min.component_min(point);
        self.max = self.max.component_max(point);
    }

    // Will not update if rectangle is zero sized
    pub fn update_from_rect(&mut self, rect: Rectangle) {
        if let Some(br) = rect.bottom_right() {
            self.update_from_point(rect.top_left);
            self.update_from_point(br);
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }

    /// Return the rectangle containing the active area.
    pub fn rectangle(&self) -> Rectangle {
        if self.touched {
            Rectangle::with_corners(self.min, self.max)
        } else {
            Rectangle::zero()
        }
    }
}
