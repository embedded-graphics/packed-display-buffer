use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::OriginDimensions, geometry::Point, pixelcolor::Rgb565,
    primitives::Rectangle,
};
use packed_display_buffer::PackedBuffer;
use tinybmp::Bmp;

fn mask(c: &mut Criterion) {
    let cases = [
        // Top left
        Point::new(0, 0),
        // Completely off screen
        Point::new(-100, -100),
        // Positive coordinates, Y is byte aligned
        Point::new(11, 16),
        // Positive coordinates, Y is NOT byte aligned
        Point::new(11, 11),
        // Large(ish) coords, partially off bottom right corner
        Point::new(100, 48),
    ];

    let mut group = c.benchmark_group("draw image");

    for tl in cases.iter() {
        let bmp: Bmp<Rgb565> =
            Bmp::from_slice(include_bytes!("./dvd.bmp")).expect("Failed to load BMP image");

        let pixels = bmp.pixels().map(|p| p.1.into());

        let bb = Rectangle::new(*tl, bmp.size());

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("Top left {:?}", tl)),
            &pixels,
            |b, pixels| {
                let mut buffer = PackedBuffer::<128, 64, { 128 * 64 / u8::BITS as usize }>::new();

                b.iter(|| buffer.fill_contiguous(&bb, pixels.clone()));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, mask);
criterion_main!(benches);
