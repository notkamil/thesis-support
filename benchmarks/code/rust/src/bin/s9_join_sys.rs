use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::fresh_workdir;
use otterbrix_bench::scenarios::{
    DB, N_JOIN_RUNS, N_JOIN_WARMUP, SEED_JOIN, TBL, TBL2,
};
use otterbrix_sys::*;
use std::ffi::CStr;
use std::time::Instant;

fn sv(s: &str) -> string_view_t {
    string_view_t {
        data: s.as_ptr() as *const i8,
        size: s.len(),
    }
}

fn main() {
    let workdir = fresh_workdir("s9_sys_");
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
        let cur = create_collection(db, sv(DB), sv(TBL2));
        release_cursor(cur);

        let insert_t = format!(
            "INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);"
        );
        let insert_u = format!(
            "INSERT INTO {DB}.{TBL2} (id, name, x) VALUES ($1, $2, $3);"
        );
        for r in bench_rows(SEED_JOIN) {
            let p = [
                sql_param_t {
                    index: 1,
                    kind: sql_param_kind_t_SQL_PARAM_INT64,
                    bool_value: 0,
                    int64_value: r.id,
                    uint64_value: 0,
                    double_value: 0.0,
                    string_value: sv(""),
                },
                sql_param_t {
                    index: 2,
                    kind: sql_param_kind_t_SQL_PARAM_STRING,
                    bool_value: 0,
                    int64_value: 0,
                    uint64_value: 0,
                    double_value: 0.0,
                    string_value: sv(&r.name),
                },
                sql_param_t {
                    index: 3,
                    kind: sql_param_kind_t_SQL_PARAM_DOUBLE,
                    bool_value: 0,
                    int64_value: 0,
                    uint64_value: 0,
                    double_value: r.x,
                    string_value: sv(""),
                },
            ];
            let cur = execute_sql_params(db, sv(&insert_t), p.as_ptr(), p.len());
            release_cursor(cur);
            let cur = execute_sql_params(db, sv(&insert_u), p.as_ptr(), p.len());
            release_cursor(cur);
        }

        let join_sql = format!(
            "SELECT t.id, t.name, u.x FROM {DB}.{TBL} AS t \
             INNER JOIN {DB}.{TBL2} AS u ON t.id = u.id;"
        );
        let join_sv = sv(&join_sql);

        for _ in 0..N_JOIN_WARMUP {
            let cur = execute_sql(db, join_sv);
            release_cursor(cur);
        }

        let mut samples = Vec::with_capacity(N_JOIN_RUNS);
        for _ in 0..N_JOIN_RUNS {
            let t0 = Instant::now();
            let cur = execute_sql(db, join_sv);
            let n = cursor_size(cur);
            let mut sink: i64 = 0;
            for r in 0..n {
                let v = cursor_get_value(cur, r, 0);
                sink = sink.wrapping_add(value_get_int(v));
                release_value(v);
                let v = cursor_get_value(cur, r, 1);
                let raw = value_get_string(v);
                let s = CStr::from_ptr(raw).to_string_lossy();
                sink = sink.wrapping_add(s.len() as i64);
                release_value(v);
                let v = cursor_get_value(cur, r, 2);
                sink = sink.wrapping_add(value_get_double(v) as i64);
                release_value(v);
            }
            release_cursor(cur);
            std::hint::black_box(sink);
            samples.push(t0.elapsed().as_nanos());
        }
        csv::write_samples("s9_join_sys", &samples);

        otterbrix_destroy(db);
    }
}
