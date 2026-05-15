use otterbrix_bench::csv;
use otterbrix_bench::scenarios::{N_OPEN, N_OPEN_WARMUP};
use otterbrix_bench::{bench_config, fresh_workdir};
use sqlx_core::connection::ConnectOptions;
use sqlx_otterbrix::OtterbrixConnectOptions;
use std::time::Instant;
use tokio::runtime::Builder;

fn one_run(rt: &tokio::runtime::Runtime, prefix: &str, i: usize) -> u128 {
    let workdir = fresh_workdir(&format!("{prefix}{i}_"));
    let opts = OtterbrixConnectOptions::from_config(
        bench_config(workdir.path()),
        workdir.path(),
    );
    let t0 = Instant::now();
    let _conn = rt.block_on(async { opts.connect().await }).expect("connect");
    t0.elapsed().as_nanos()
}

fn main() {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    for i in 0..N_OPEN_WARMUP {
        let _ = one_run(&rt, "s6_sqlx_warm_", i);
    }

    let mut samples = Vec::with_capacity(N_OPEN);
    for i in 0..N_OPEN {
        samples.push(one_run(&rt, "s6_sqlx_", i));
    }

    csv::write_samples("s6_open_sqlx", &samples);
}
