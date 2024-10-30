use criterion::{black_box, criterion_group, criterion_main, Criterion};
use router::utils::bitmap::Bitmap;

fn benchmark_is_empty(c: &mut Criterion) {
    let bitmap = Bitmap::new();
    c.bench_function("is_empty", |b| b.iter(|| black_box(bitmap).is_empty()));
}

fn benchmark_as_value(c: &mut Criterion) {
    let bitmap = Bitmap::new();
    c.bench_function("as_value == 0", |b| {
        b.iter(|| black_box(bitmap).as_value() == 0)
    });
}

criterion_group!(benches, benchmark_is_empty, benchmark_as_value);
criterion_main!(benches);
