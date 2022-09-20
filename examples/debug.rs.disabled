//! Debugging example; click anywhere on the screen to draw an item.

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay, SimulatorEvent, Window};
use packed_display_buffer::PackedBuffer;

fn draw_stuff(
    display: &mut PackedBuffer<256, 256, { 256 * (256 / 8) }>,
    _old_center: Point,
    new_center: Point,
) -> Result<(), core::convert::Infallible> {
    display.clear(BinaryColor::Off)?;

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("./rust.raw"), 64);
    let im = Image::new(&raw, new_center - raw.size() / 2);

    im.draw(display)?;

    Ok(())
}

fn main() -> Result<(), std::convert::Infallible> {
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(256, 256));
    let mut window = Window::new("Click to move image", &OutputSettings::default());

    let mut framebuffer = PackedBuffer::<256, 256, { 256 * (256 / 8) }>::new();

    let mut position = Point::new(100, 100);

    draw_stuff(&mut framebuffer, position, position)?;

    'running: loop {
        display.draw_iter(framebuffer.pixels())?;

        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::MouseButtonUp { point, .. } => {
                    println!("{point:?}");
                    draw_stuff(&mut framebuffer, position, point)?;
                    position = point;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
