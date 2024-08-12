use std::hash::BuildHasher;

use rand::prelude::*;

fn compute_u64_avalanche<H: BuildHasher, F: FnMut() -> H>(
    num_hashers: usize,
    iters_per_hasher: usize,
    mut new_hasher: F,
) -> Vec<f64> {
    let mut rng = thread_rng();
    let mut worst_bias = vec![0.5f64; 64 * 64];
    for _ in 0..num_hashers {
        let h = new_hasher();
        let mut bit_flips = vec![0; 64 * 64];
        for _ in 0..iters_per_hasher {
            let base_val: u64 = rng.gen();
            let base_hash = h.hash_one(base_val);
            for flip_pos in 0..64 {
                let delta_val = base_val ^ (1 << flip_pos);
                let delta_hash = h.hash_one(delta_val);

                for test_pos in 0..64 {
                    let flipped = ((base_hash ^ delta_hash) >> test_pos) & 1;
                    bit_flips[test_pos * 64 + flip_pos] += flipped as usize;
                }
            }
        }

        for i in 0..64 * 64 {
            let flip_frac = bit_flips[i] as f64 / iters_per_hasher as f64;
            if (flip_frac - 0.5).abs() > (worst_bias[i] - 0.5).abs() {
                worst_bias[i] = flip_frac;
            }
        }
    }

    worst_bias
}

fn write_avalanche_csv<H: BuildHasher, F: FnMut() -> H>(name: &str, new_hasher: F) {
    println!("calculating avalanche properties of {name}");
    let strings: Vec<String> = compute_u64_avalanche(10000, 1000, new_hasher)
        .into_iter()
        .map(|b| format!("{b}"))
        .collect();
    std::fs::create_dir_all("out").unwrap();
    std::fs::write(format!("out/avalanche-{name}.csv"), strings.join(",")).unwrap();
}

fn main() {
    write_avalanche_csv("foldhash-fast", || foldhash::fast::RandomState::default());
    write_avalanche_csv("foldhash-quality", || {
        foldhash::quality::RandomState::default()
    });
    write_avalanche_csv("siphash", || std::hash::RandomState::default());
    write_avalanche_csv("ahash", || ahash::RandomState::default());
    write_avalanche_csv("fxhash", || fxhash::FxBuildHasher::default());
}
