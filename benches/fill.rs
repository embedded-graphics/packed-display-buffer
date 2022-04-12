use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::Point, pixelcolor::BinaryColor, primitives::Rectangle,
};
use packed_display_buffer::PackedBuffer;

fn mask(c: &mut Criterion) {
    let cases = [
        // Full display fill
        Rectangle::with_corners(Point::new(0, 0), Point::new(127, 63)),
        // Pathalogical case; half filled top/bottom block, all other blocks filled
        Rectangle::with_corners(Point::new(4, 4), Point::new(120, 60)),
        // Tiny rectangle near center of display
        Rectangle::with_corners(Point::new(50, 33), Point::new(60, 38)),
        // Zero sized because why not
        Rectangle::zero(),
        // Single pixel
        Rectangle::with_corners(Point::new(50, 33), Point::new(51, 34)),
    ];

    let mut group = c.benchmark_group("create mask");

    for case in cases.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?} - {:?}", case.top_left, case.bottom_right())),
            case,
            |b, fill_area| {
                let mut buffer = PackedBuffer::<128, 64, { 128 * 64 / u8::BITS as usize }>::new();

                b.iter(|| buffer.fill_solid(fill_area, BinaryColor::On));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, mask);
criterion_main!(benches);
