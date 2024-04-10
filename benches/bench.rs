use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlite_compressions::{BrotliEncoder, Bzip2Encoder, Encoder as _, GzipEncoder};

macro_rules! enc_test {
    ($func_name:ident, $enc_type:ident, $func:literal) => {
        fn $func_name(c: &mut Criterion) {
            let mut group = c.benchmark_group("test");
            group.sample_size(60);
            for size in [10, 10 * 1024, 1024 * 1024] {
                let data = $enc_type::encode(gen_data(size).as_slice(), None).unwrap();
                group.bench_function(BenchmarkId::new($func, size), |b| {
                    b.iter(|| $enc_type::test(&data));
                });
            }
            group.finish();
        }
    };
}

enc_test!(gzip_test, GzipEncoder, "gzip");
enc_test!(brotli_test, BrotliEncoder, "brotli");
enc_test!(bzip2_test, Bzip2Encoder, "bzip2");

fn gen_data(size: usize) -> Vec<u8> {
    let mut byte_data: Vec<u8> = Vec::with_capacity(size);
    for i in 0..size {
        #[allow(clippy::cast_possible_truncation)]
        byte_data.push((i % 256) as u8);
    }
    byte_data
}

criterion_group!(benches, gzip_test, brotli_test, bzip2_test);
criterion_main!(benches);
