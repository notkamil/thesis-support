use otterbrix_bench::csv;
use otterbrix_bench::fresh_workdir;
use otterbrix_bench::scenarios::{N_OPEN, N_OPEN_WARMUP};
use otterbrix_sys::*;
use std::time::Instant;

fn sv(s: &str) -> string_view_t {
    string_view_t {
        data: s.as_ptr() as *const i8,
        size: s.len(),
    }
}

fn one_run(prefix: &str, i: usize) -> u128 {
    let workdir = fresh_workdir(&format!("{prefix}{i}_"));
    let log = workdir.path().join("log");
    let wal = workdir.path().join("wal");
    let disk = workdir.path().join("disk");
    let main = workdir.path().join("main");

    let cfg = config_t {
        level: 6,
        log_path: sv(log.to_str().unwrap()),
        wal_path: sv(wal.to_str().unwrap()),
        disk_path: sv(disk.to_str().unwrap()),
        main_path: sv(main.to_str().unwrap()),
        wal_on: false,
        disk_on: false,
        sync_to_disk: false,
    };

    let t0 = Instant::now();
    let db = unsafe { otterbrix_create(cfg) };
    let elapsed = t0.elapsed().as_nanos();
    unsafe { otterbrix_destroy(db) };
    elapsed
}

fn main() {
    for i in 0..N_OPEN_WARMUP {
        let _ = one_run("s6_sys_warm_", i);
    }

    let mut samples = Vec::with_capacity(N_OPEN);
    for i in 0..N_OPEN {
        samples.push(one_run("s6_sys_", i));
    }

    csv::write_samples("s6_open_sys", &samples);
}
