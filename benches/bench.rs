use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::hint::black_box;
use std::time::Duration;

const NUM_PRECOMPUTED_KEYS: usize = 1024;

mod distribution;
use distribution::Distribution;

fn profile_hashonly<S: BuildHasher + Default, D: Distribution>(
    hash_name: &str,
    mut distr: D,
    c: &mut BenchmarkGroup<'_, WallTime>,
) {
    let name = format!("hashonly-{}-{hash_name}", distr.name().to_lowercase());
    let mut rng = StdRng::seed_from_u64(0x123456789abcdef);

    let hasher = S::default();

    c.bench_function(&name, |b| {
        b.iter_custom(|iters| {
            let to_hash: Vec<_> = black_box(
                (0..NUM_PRECOMPUTED_KEYS)
                    .map(|_| distr.sample(&mut rng))
                    .collect(),
            );
            let start = std::time::Instant::now();
            for i in 0..iters as usize {
                black_box(hasher.hash_one(&to_hash[i % NUM_PRECOMPUTED_KEYS]));
            }
            start.elapsed()
        });
    });
}

fn profile_lookup_hit<S: BuildHasher + Default, D: Distribution>(
    hash_name: &str,
    mut distr: D,
    map_size: usize,
    c: &mut BenchmarkGroup<'_, WallTime>,
) {
    let name = format!("lookuphit-{}-{hash_name}", distr.name().to_lowercase());
    let mut rng = StdRng::seed_from_u64(0x123456789abcdef);

    c.bench_function(&name, |b| {
        b.iter_custom(|iters| {
            let mut hm: HashMap<D::Value, u64, S> = HashMap::with_hasher(S::default());
            for i in 0..map_size {
                hm.insert(distr.sample(&mut rng), i as u64);
            }

            let keys: Vec<_> = hm.keys().cloned().collect();
            let lookup: Vec<_> = black_box(
                (0..NUM_PRECOMPUTED_KEYS)
                    .map(|_| keys.choose(&mut rng).unwrap().clone())
                    .collect(),
            );

            let start = std::time::Instant::now();
            let mut sum = 0u64;
            for i in 0..iters as usize {
                if let Some(x) = hm.get(&lookup[i % NUM_PRECOMPUTED_KEYS]) {
                    sum = sum.wrapping_add(*x);
                }
            }
            black_box(sum);
            start.elapsed()
        });
    });
}

fn profile_lookup_miss<S: BuildHasher + Default, D: Distribution>(
    hash_name: &str,
    mut distr: D,
    map_size: usize,
    c: &mut BenchmarkGroup<'_, WallTime>,
) {
    let name = format!("lookupmiss-{}-{hash_name}", distr.name().to_lowercase());
    let mut rng = StdRng::seed_from_u64(0x123456789abcdef);

    c.bench_function(&name, |b| {
        b.iter_custom(|iters| {
            let mut hm: HashMap<D::Value, u64, S> = HashMap::with_hasher(S::default());
            for i in 0..map_size {
                hm.insert(distr.sample(&mut rng), i as u64);
            }

            let lookup: Vec<_> = black_box(
                (0..NUM_PRECOMPUTED_KEYS)
                    .map(|_| distr.sample_missing(&mut rng))
                    .collect(),
            );

            let start = std::time::Instant::now();
            let mut sum = 0u64;
            for i in 0..iters as usize {
                if let Some(x) = hm.get(&lookup[i % NUM_PRECOMPUTED_KEYS]) {
                    sum = sum.wrapping_add(*x);
                }
            }
            black_box(sum);
            start.elapsed()
        });
    });
}

fn profile_set_build<S: BuildHasher + Default, D: Distribution>(
    hash_name: &str,
    mut distr: D,
    map_size: usize,
    c: &mut BenchmarkGroup<'_, WallTime>,
) {
    let name = format!("setbuild-{}-{hash_name}", distr.name().to_lowercase());
    let mut rng = StdRng::seed_from_u64(0x123456789abcdef);

    c.bench_function(&name, |b| {
        b.iter_custom(|iters| {
            // Repeat each key 10 times.
            let keys: Vec<_> = (0..map_size).map(|_| distr.sample(&mut rng)).collect();
            let mut keys: Vec<_> = keys.iter().cycle().cloned().take(10 * map_size).collect();
            keys.shuffle(&mut rng);
            let keys = black_box(keys);

            let start = std::time::Instant::now();
            for _ in 0..iters as usize {
                // We intentionally do not pre-reserve so we observe re-hash
                // behavior.
                let mut set = HashSet::with_hasher(S::default());
                for key in &keys {
                    set.insert(key);
                }
                black_box(set);
            }
            start.elapsed()
        });
    });
}

#[rustfmt::skip]
fn profile_distr<D: Distribution>(distr: D, map_size: usize, c: &mut Criterion) {
    let c = &mut c.benchmark_group(distr.name());
    c.sampling_mode(criterion::SamplingMode::Flat);

    profile_hashonly::<foldhash::fast::RandomState, _>("foldhash-fast", distr.clone(), c);
    profile_hashonly::<foldhash::quality::RandomState, _>("foldhash-quality", distr.clone(), c);
    profile_hashonly::<fxhash::FxBuildHasher, _>("fxhash", distr.clone(), c);
    profile_hashonly::<ahash::RandomState, _>("ahash", distr.clone(), c);
    profile_hashonly::<std::hash::RandomState, _>("siphash", distr.clone(), c);

    profile_lookup_miss::<foldhash::fast::RandomState, _>("foldhash-fast", distr.clone(), map_size, c);
    profile_lookup_miss::<foldhash::quality::RandomState, _>("foldhash-quality", distr.clone(), map_size, c);
    profile_lookup_miss::<fxhash::FxBuildHasher, _>("fxhash", distr.clone(), map_size, c);
    profile_lookup_miss::<ahash::RandomState, _>("ahash", distr.clone(), map_size, c);
    profile_lookup_miss::<std::hash::RandomState, _>("siphash", distr.clone(), map_size, c);

    profile_lookup_hit::<foldhash::fast::RandomState, _>("foldhash-fast", distr.clone(), map_size, c);
    profile_lookup_hit::<foldhash::quality::RandomState, _>("foldhash-quality", distr.clone(), map_size, c);
    profile_lookup_hit::<fxhash::FxBuildHasher, _>("fxhash", distr.clone(), map_size, c);
    profile_lookup_hit::<ahash::RandomState, _>("ahash", distr.clone(), map_size, c);
    profile_lookup_hit::<std::hash::RandomState, _>("siphash", distr.clone(), map_size, c);

    profile_set_build::<foldhash::fast::RandomState, _>("foldhash-fast", distr.clone(), map_size, c);
    profile_set_build::<foldhash::quality::RandomState, _>("foldhash-quality", distr.clone(), map_size, c);
    profile_set_build::<fxhash::FxBuildHasher, _>("fxhash", distr.clone(), map_size, c);
    profile_set_build::<ahash::RandomState, _>("ahash", distr.clone(), map_size, c);
    profile_set_build::<std::hash::RandomState, _>("siphash", distr.clone(), map_size, c);
}

fn bench_hashes(c: &mut Criterion) {
    let map_size = 1000;
    profile_distr(distribution::U32, map_size, c);
    profile_distr(distribution::U64, map_size, c);
    profile_distr(distribution::U64LoBits, map_size, c);
    profile_distr(distribution::U64HiBits, map_size, c);
    profile_distr(distribution::U32Pair, map_size, c);
    profile_distr(distribution::U64Pair, map_size, c);
    profile_distr(distribution::Rgba, map_size, c);
    profile_distr(distribution::Ipv4, map_size, c);
    profile_distr(distribution::Ipv6, map_size, c);
    profile_distr(distribution::StrUuid, map_size, c);
    profile_distr(distribution::StrDate, map_size, c);
    profile_distr(distribution::AccessLog, map_size, c);
    profile_distr(distribution::StrWordList::english(), map_size, c);
    profile_distr(distribution::StrWordList::urls(), map_size, c);
    profile_distr(distribution::Kilobyte, map_size, c);
    profile_distr(distribution::TenKilobyte, map_size, c);
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(5));
    targets = bench_hashes
);
criterion_main!(benches);
