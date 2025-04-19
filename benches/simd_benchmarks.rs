use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::Rng;
use epirust::compute::simd::SimdOperations;

fn generate_test_data(size: usize) -> (Vec<usize>, Vec<usize>) {
    let mut rng = rand::thread_rng();
    let n_risk: Vec<usize> = (0..size).map(|_| rng.gen_range(10..100)).collect();
    let n_event: Vec<usize> = n_risk.iter()
        .map(|&r| rng.gen_range(0..r))
        .collect();
    (n_risk, n_event)
}

fn generate_vector_data(size: usize) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen::<f64>()).collect()
}

fn generate_aligned_data(size: usize, alignment: usize) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let padding = alignment - (size % alignment);
    let padded_size = size + padding;
    (0..padded_size).map(|_| rng.gen::<f64>()).collect()
}

fn generate_sparse_data(size: usize, sparsity: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| {
        if rng.gen::<f64>() > sparsity {
            rng.gen::<f64>()
        } else {
            0.0
        }
    }).collect()
}

fn generate_survival_edge_cases(size: usize) -> (Vec<usize>, Vec<usize>) {
    let mut rng = rand::thread_rng();
    let n_risk: Vec<usize> = (0..size).map(|i| {
        match i % 4 {
            0 => 100, // constant risk
            1 => rng.gen_range(90..100), // high risk
            2 => rng.gen_range(10..20),  // low risk
            _ => 10,  // minimum risk
        }
    }).collect();
    
    let n_event: Vec<usize> = n_risk.iter()
        .enumerate()
        .map(|(i, &r)| {
            match i % 4 {
                0 => r/2,     // 50% event rate
                1 => r - 1,   // high event rate
                2 => 1,       // low event rate
                _ => 0,       // no events
            }
        })
        .collect();
    
    (n_risk, n_event)
}

fn benchmark_survival_probabilities(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];
    let simd_ops = SimdOperations::new();

    let mut group = c.benchmark_group("survival_probabilities");
    for size in sizes {
        let (n_risk, n_event) = generate_test_data(size);
        
        group.bench_function(format!("size_{}", size), |b| {
            b.iter(|| {
                simd_ops.compute_survival_probabilities(
                    black_box(&n_risk),
                    black_box(&n_event)
                )
            })
        });
    }
    group.finish();
}

fn benchmark_vector_sum(c: &mut Criterion) {
    let sizes = [128, 1024, 8192, 65536];
    let simd_ops = SimdOperations::new();

    let mut group = c.benchmark_group("vector_sum");
    for size in sizes {
        let data = generate_vector_data(size);
        
        group.bench_function(format!("size_{}", size), |b| {
            b.iter(|| {
                simd_ops.vector_sum(black_box(&data))
            })
        });
    }
    group.finish();
}

fn benchmark_vector_sum_alignment(c: &mut Criterion) {
    let sizes = [32, 64, 128];
    let alignments = [8, 16, 32];
    let simd_ops = SimdOperations::new();

    let mut group = c.benchmark_group("vector_sum_alignment");
    for size in sizes {
        for &alignment in &alignments {
            let data = generate_aligned_data(size, alignment);
            group.bench_with_input(
                BenchmarkId::new(format!("size_{}", size), alignment),
                &data,
                |b, data| b.iter(|| simd_ops.vector_sum(black_box(data)))
            );
        }
    }
    group.finish();
}

fn benchmark_vector_sum_sparsity(c: &mut Criterion) {
    let size = 1000;
    let sparsities = [0.0, 0.25, 0.5, 0.75, 0.9];
    let simd_ops = SimdOperations::new();

    let mut group = c.benchmark_group("vector_sum_sparsity");
    for sparsity in sparsities {
        let data = generate_sparse_data(size, sparsity);
        group.bench_function(format!("sparsity_{}", sparsity), |b| {
            b.iter(|| simd_ops.vector_sum(black_box(&data)))
        });
    }
    group.finish();
}

fn benchmark_survival_edge_cases(c: &mut Criterion) {
    let sizes = [100, 1000];
    let simd_ops = SimdOperations::new();

    let mut group = c.benchmark_group("survival_edge_cases");
    for size in sizes {
        let (n_risk, n_event) = generate_survival_edge_cases(size);
        
        group.bench_function(format!("mixed_patterns_{}", size), |b| {
            b.iter(|| {
                simd_ops.compute_survival_probabilities(
                    black_box(&n_risk),
                    black_box(&n_event)
                )
            })
        });
    }
    group.finish();
}

fn benchmark_small_vectors(c: &mut Criterion) {
    let sizes = [2, 4, 8, 16];  // Sizes around SIMD register boundaries
    let simd_ops = SimdOperations::new();

    let mut group = c.benchmark_group("small_vectors");
    for size in sizes {
        let data = generate_vector_data(size);
        group.bench_function(format!("size_{}", size), |b| {
            b.iter(|| simd_ops.vector_sum(black_box(&data)))
        });
    }
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_survival_probabilities,
    benchmark_vector_sum,
    benchmark_vector_sum_alignment,
    benchmark_vector_sum_sparsity,
    benchmark_survival_edge_cases,
    benchmark_small_vectors
);
criterion_main!(benches); 