use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub const SEED: u64 = 0xCAFE_BABE_DEAD_BEEF;

#[derive(Debug, Clone)]
pub struct BenchRow {
    pub id: i64,
    pub name: String,
    pub x: f64,
}

#[must_use]
pub fn bench_rows(n: usize) -> Vec<BenchRow> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut rows = Vec::with_capacity(n);
    for i in 1..=n as i64 {
        rows.push(BenchRow {
            id: i,
            name: random_name(&mut rng, 16),
            x: rng.gen_range(-1.0e6..1.0e6),
        });
    }
    rows
}

#[must_use]
pub fn lookup_ids(n: usize, max_id: i64) -> Vec<i64> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ 1);
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(rng.gen_range(1..=max_id));
    }
    out
}

fn random_name(rng: &mut ChaCha8Rng, len: usize) -> String {
    const ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let i = rng.gen_range(0..ALPHA.len());
        s.push(ALPHA[i] as char);
    }
    s
}
