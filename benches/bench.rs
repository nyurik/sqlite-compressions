use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlite_compressions::{BrotliEncoder, Encoder as _, GzipEncoder};

// criterion_group!(benches, gzip_test);
// criterion_group!(benches, brotli_test);
criterion_group!(benches, gzip_test, brotli_test);
criterion_main!(benches);

fn gzip_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("test");
    for size in [10, 10 * 1024, 1024 * 1024] {
        let data = GzipEncoder::encode(gen_data(size).as_slice(), None).unwrap();
        group.bench_function(BenchmarkId::new("gzip", size), |b| {
            b.iter(|| GzipEncoder::test(&data));
        });
    }
    group.finish();
}

fn brotli_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("test");
    group.sample_size(60);
    for size in [10, 10 * 1024, 1024 * 1024] {
        let data = BrotliEncoder::encode(gen_data(size).as_slice(), None).unwrap();
        group.bench_function(BenchmarkId::new("brotli", size), |b| {
            b.iter(|| BrotliEncoder::test(&data));
        });
    }
    group.finish();
}

fn gen_data(size: usize) -> Vec<u8> {
    let mut byte_data: Vec<u8> = Vec::with_capacity(size);
    for i in 0..size {
        #[allow(clippy::cast_possible_truncation)]
        byte_data.push((i % 256) as u8);
    }
    byte_data
}
