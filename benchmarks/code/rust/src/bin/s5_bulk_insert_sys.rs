use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::fresh_workdir;
use otterbrix_bench::scenarios::{DB, N_BULK, N_BULK_RUNS, N_BULK_WARMUP, TBL};
use otterbrix_sys::*;
use std::fmt::Write as _;
use std::time::Instant;

fn sv(s: &str) -> string_view_t {
    string_view_t {
        data: s.as_ptr() as *const i8,
        size: s.len(),
    }
}

fn build_bulk_sql() -> String {
    let mut s = String::with_capacity(N_BULK * 60);
    write!(s, "INSERT INTO {DB}.{TBL} (id, name, x) VALUES ").unwrap();
    for (i, r) in bench_rows(N_BULK).into_iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        write!(s, "({}, '{}', {})", r.id, r.name, r.x).unwrap();
    }
    s.push(';');
    s
}

fn one_run(bulk_sql: &str, prefix: &str, run: usize) -> u128 {
    let workdir = fresh_workdir(&format!("{prefix}{run}_"));
    let log = workdir.path().join("log");
    let wal = workdir.path().join("wal");
    let disk = workdir.path().join("disk");
    let main = workdir.path().join("main");
    unsafe {
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
        let db = otterbrix_create(cfg);
        let cur = create_database(db, sv(DB));
        release_cursor(cur);
        let cur = create_collection(db, sv(DB), sv(TBL));
        release_cursor(cur);

        let bulk_sv = sv(bulk_sql);
        let t0 = Instant::now();
        let cur = execute_sql(db, bulk_sv);
        release_cursor(cur);
        let elapsed = t0.elapsed().as_nanos();

        otterbrix_destroy(db);
        elapsed
    }
}

fn main() {
    let bulk_sql = build_bulk_sql();

    for run in 0..N_BULK_WARMUP {
        let _ = one_run(&bulk_sql, "s5_sys_warm_", run);
    }

    let mut samples = Vec::with_capacity(N_BULK_RUNS);
    for run in 0..N_BULK_RUNS {
        samples.push(one_run(&bulk_sql, "s5_sys_", run));
    }

    csv::write_samples("s5_bulk_insert_sys", &samples);
}
