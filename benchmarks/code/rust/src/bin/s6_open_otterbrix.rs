use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::scenarios::{N_OPEN, N_OPEN_WARMUP};
use otterbrix_bench::{bench_config, fresh_workdir};
use std::time::Instant;

fn one_run(prefix: &str, i: usize) -> u128 {
    let workdir = fresh_workdir(&format!("{prefix}{i}_"));
    let cfg = bench_config(workdir.path());
    let t0 = Instant::now();
    let _db = Database::open(cfg).expect("open");
    t0.elapsed().as_nanos()
}

fn main() {
    for i in 0..N_OPEN_WARMUP {
        let _ = one_run("s6_ob_warm_", i);
    }

    let mut samples = Vec::with_capacity(N_OPEN);
    for i in 0..N_OPEN {
        samples.push(one_run("s6_ob_", i));
    }

    csv::write_samples("s6_open_otterbrix", &samples);
}
