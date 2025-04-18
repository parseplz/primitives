use criterion::{Criterion, criterion_group, criterion_main};
use mime::from_extension::EXTENSION_MAP;

pub fn criterion_benchmark_hashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_type");
    group.bench_function("hashmap", |b| {
        b.iter(|| EXTENSION_MAP.get("wvx").copied());
    });
}

criterion_group!(benches, criterion_benchmark_hashmap,);
criterion_main!(benches);
