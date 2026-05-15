use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::fresh_workdir;
use otterbrix_bench::scenarios::{DB, N_AGG_RUNS, N_WARMUP, SEED_LARGE, TBL};
use otterbrix_sys::*;
use std::time::Instant;

fn sv(s: &str) -> string_view_t {
    string_view_t {
        data: s.as_ptr() as *const i8,
        size: s.len(),
    }
}

fn measure(db: otterbrix_ptr, sql: &str, name: &str) {
    let sql_sv = sv(sql);

    unsafe {
        for _ in 0..N_WARMUP {
            let cur = execute_sql(db, sql_sv);
            release_cursor(cur);
        }
    }

    let mut samples = Vec::with_capacity(N_AGG_RUNS);
    for _ in 0..N_AGG_RUNS {
        let t0 = Instant::now();
        unsafe {
            let cur = execute_sql(db, sql_sv);
            let v = cursor_get_value(cur, 0, 0);
            let _ = value_get_double(v);
            release_value(v);
            release_cursor(cur);
        }
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples(name, &samples);
}

fn main() {
    let workdir = fresh_workdir("s8_sys_");
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

        let insert_sql =
            format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
        for r in bench_rows(SEED_LARGE) {
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
            let cur = execute_sql_params(db, sv(&insert_sql), p.as_ptr(), p.len());
            release_cursor(cur);
        }

        let sum_sql = format!("SELECT SUM(x) FROM {DB}.{TBL};");
        let max_sql = format!("SELECT MAX(x) FROM {DB}.{TBL};");
        measure(db, &sum_sql, "s8_aggregation_sys_sum");
        measure(db, &max_sql, "s8_aggregation_sys_max");

        otterbrix_destroy(db);
    }
}
