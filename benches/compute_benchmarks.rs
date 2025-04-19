use criterion::{black_box, criterion_group, criterion_main, Criterion};
use epirust::compute::simd::SimdOperations;
use epirust::compute::parallel::NumaAwareThreadPool;
use epirust::datasets::test_data::create_test_dataset;

fn bench_simd_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("SIMD Operations");
    let data = vec![1.0f64; 1_000_000];
    let simd_ops = SimdOperations::new();

    group.bench_function("vector_sum_simd", |b| {
        b.iter(|| simd_ops.vector_sum(black_box(&data)))
    });

    group.bench_function("vector_sum_scalar", |b| {
        b.iter(|| data.iter().sum::<f64>())
    });

    group.finish();
}

fn bench_numa_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("NUMA Operations");
    let data = create_test_dataset();
    let pool = NumaAwareThreadPool::new(Default::default()).unwrap();

    group.bench_function("parallel_sum_numa", |b| {
        b.iter(|| {
            pool.run(&data.concat(), |slice| {
                slice.iter().sum::<f64>()
            })
        })
    });

    group.finish();
}

criterion_group!(benches, bench_simd_operations, bench_numa_operations);
criterion_main!(benches); 