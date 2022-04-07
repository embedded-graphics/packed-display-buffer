use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use packed_display_buffer::mask::build_mask;

fn mask(c: &mut Criterion) {
    let cases = [(3, 60), (0, 16), (35, 63)];

    let mut group = c.benchmark_group("create mask");

    for case in cases.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{} - {}", case.0, case.1)),
            case,
            |b, &(start, end)| {
                b.iter(|| {
                    let mut mask_buf = [0x00u8; 8];

                    build_mask(&mut mask_buf, start, end);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, mask);
criterion_main!(benches);
