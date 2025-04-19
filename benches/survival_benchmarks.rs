use criterion::{black_box, criterion_group, criterion_main, Criterion};
use epirust::compute::survival::kaplan_meier::KaplanMeier;
use rand::Rng;

fn generate_survival_data(size: usize) -> (Vec<f64>, Vec<bool>) {
    let mut rng = rand::thread_rng();
    let time: Vec<f64> = (0..size).map(|i| i as f64 + rng.gen::<f64>()).collect();
    let event: Vec<bool> = (0..size).map(|_| rng.gen_bool(0.3)).collect();
    (time, event)
}

fn bench_kaplan_meier(c: &mut Criterion) {
    let mut group = c.benchmark_group("Survival Analysis");
    let sizes = [100, 1000, 10000];

    for size in sizes {
        let (time, event) = generate_survival_data(size);
        let km = KaplanMeier::new().unwrap();

        group.bench_function(format!("kaplan_meier_n_{}", size), |b| {
            b.iter(|| km.fit(black_box(&time), black_box(&event)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_kaplan_meier);
criterion_main!(benches); 