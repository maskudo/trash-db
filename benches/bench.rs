use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use tempfile::TempDir;
use trash_db::engines::{kvstore::KvStore, sled::SledKvsEngine, KvsEngine};

fn set_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_bench");
    group.bench_function("kvs", |b| {
        b.iter_batched(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                let store = KvStore::open(temp_dir.path()).expect("unable to open kvstore");
                store
            },
            |mut store| {
                for i in 1..(1 << 12) {
                    store.set(format!("key{i}"), format!("value{i}")).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function("sled", |b| {
        b.iter_batched(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                let store = SledKvsEngine::open(temp_dir.path()).expect("unable to open kvstore");
                store
            },
            |mut store| {
                for i in 1..(1 << 12) {
                    store.set(format!("key{i}"), format!("value{i}")).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn get_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_bench");
    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("kvs_{i}"), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = KvStore::open(temp_dir.path()).unwrap();
            for key_i in 1..(1 << i) {
                store
                    .set(format!("key{key_i}"), format!("value{key_i}"))
                    .unwrap();
            }
            let mut rng = SmallRng::from_seed([0; 16]);
            b.iter(|| {
                store
                    .get(format!("key{}", rng.gen_range(1, 1 << i)))
                    .unwrap();
            })
        });
    }

    for i in &vec![8, 12, 16, 20] {
        group.bench_with_input(format!("sled_{i}"), i, |b, i| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = SledKvsEngine::open(temp_dir.path()).unwrap();
            for key_i in 1..(1 << i) {
                store
                    .set(format!("key{key_i}"), format!("value{key_i}"))
                    .unwrap();
            }
            let mut rng = SmallRng::from_seed([0; 16]);
            b.iter(|| {
                store
                    .get(format!("key{}", rng.gen_range(1, 1 << i)))
                    .unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, set_bench, get_bench);
criterion_main!(benches);
