use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::scenarios::{N_OPEN, N_OPEN_WARMUP};
use otterbrix_bench::{bench_config, fresh_workdir};
use sea_orm::{DbBackend, ProxyDatabaseTrait};
use seaorm_otterbrix::OtterbrixProxy;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Builder;

fn one_run(rt: &tokio::runtime::Runtime, prefix: &str, i: usize) -> u128 {
    let workdir = fresh_workdir(&format!("{prefix}{i}_"));
    let t0 = Instant::now();
    let db = Database::open(bench_config(workdir.path())).expect("open");
    let proxy: Arc<Box<dyn ProxyDatabaseTrait>> =
        Arc::new(Box::new(OtterbrixProxy::new(db)));
    let _conn = rt
        .block_on(async {
            sea_orm::Database::connect_proxy(DbBackend::Sqlite, proxy).await
        })
        .expect("connect_proxy");
    t0.elapsed().as_nanos()
}

fn main() {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    for i in 0..N_OPEN_WARMUP {
        let _ = one_run(&rt, "s6_seaorm_warm_", i);
    }

    let mut samples = Vec::with_capacity(N_OPEN);
    for i in 0..N_OPEN {
        samples.push(one_run(&rt, "s6_seaorm_", i));
    }

    csv::write_samples("s6_open_seaorm", &samples);
}
