use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::fresh_workdir;
use otterbrix_bench::scenarios::{
    DB, N_DELETE, N_UPDATE, N_WARMUP, SEED_MUTATE, SEED_SMALL, TBL,
};
use otterbrix_sys::*;
use std::time::Instant;

fn sv(s: &str) -> string_view_t {
    string_view_t {
        data: s.as_ptr() as *const i8,
        size: s.len(),
    }
}

fn parse_mode() -> &'static str {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: s10_mutate_sys <update|delete>");
        std::process::exit(2);
    });
    match arg.as_str() {
        "update" => "update",
        "delete" => "delete",
        other => {
            eprintln!("invalid mode '{other}', expected update|delete");
            std::process::exit(2);
        }
    }
}

unsafe fn seed(db: otterbrix_ptr, n: usize) {
    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    let insert_sv = sv(&insert_sql);
    for r in bench_rows(n) {
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
        let cur = execute_sql_params(db, insert_sv, p.as_ptr(), p.len());
        release_cursor(cur);
    }
}

unsafe fn run_update(db: otterbrix_ptr) {
    let update_sql = format!("UPDATE {DB}.{TBL} SET x = $1 WHERE id = $2;");
    let update_sv = sv(&update_sql);

    for i in 0..N_WARMUP {
        let id = (i % SEED_SMALL) as i64 + 1;
        let p = [
            sql_param_t {
                index: 1,
                kind: sql_param_kind_t_SQL_PARAM_DOUBLE,
                bool_value: 0,
                int64_value: 0,
                uint64_value: 0,
                double_value: i as f64 * 0.5,
                string_value: sv(""),
            },
            sql_param_t {
                index: 2,
                kind: sql_param_kind_t_SQL_PARAM_INT64,
                bool_value: 0,
                int64_value: id,
                uint64_value: 0,
                double_value: 0.0,
                string_value: sv(""),
            },
        ];
        let cur = execute_sql_params(db, update_sv, p.as_ptr(), p.len());
        release_cursor(cur);
    }

    let mut samples = Vec::with_capacity(N_UPDATE);
    for i in 0..N_UPDATE {
        let id = ((i + N_WARMUP) % SEED_SMALL) as i64 + 1;
        let p = [
            sql_param_t {
                index: 1,
                kind: sql_param_kind_t_SQL_PARAM_DOUBLE,
                bool_value: 0,
                int64_value: 0,
                uint64_value: 0,
                double_value: i as f64 * 0.5,
                string_value: sv(""),
            },
            sql_param_t {
                index: 2,
                kind: sql_param_kind_t_SQL_PARAM_INT64,
                bool_value: 0,
                int64_value: id,
                uint64_value: 0,
                double_value: 0.0,
                string_value: sv(""),
            },
        ];
        let t0 = Instant::now();
        let cur = execute_sql_params(db, update_sv, p.as_ptr(), p.len());
        release_cursor(cur);
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_sys_update", &samples);
}

unsafe fn run_delete(db: otterbrix_ptr) {
    let delete_sql = format!("DELETE FROM {DB}.{TBL} WHERE id = $1;");
    let delete_sv = sv(&delete_sql);

    for i in 0..N_WARMUP {
        let id = (SEED_SMALL + i + 1) as i64;
        let p = [sql_param_t {
            index: 1,
            kind: sql_param_kind_t_SQL_PARAM_INT64,
            bool_value: 0,
            int64_value: id,
            uint64_value: 0,
            double_value: 0.0,
            string_value: sv(""),
        }];
        let cur = execute_sql_params(db, delete_sv, p.as_ptr(), p.len());
        release_cursor(cur);
    }

    let mut samples = Vec::with_capacity(N_DELETE);
    for i in 0..N_DELETE {
        let id = (SEED_SMALL + N_WARMUP + i + 1) as i64;
        let p = [sql_param_t {
            index: 1,
            kind: sql_param_kind_t_SQL_PARAM_INT64,
            bool_value: 0,
            int64_value: id,
            uint64_value: 0,
            double_value: 0.0,
            string_value: sv(""),
        }];
        let t0 = Instant::now();
        let cur = execute_sql_params(db, delete_sv, p.as_ptr(), p.len());
        release_cursor(cur);
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_sys_delete", &samples);
}

fn main() {
    let mode = parse_mode();
    let workdir = fresh_workdir(&format!("s10_sys_{mode}_"));
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
        assert!(!db.is_null());
        let cur = create_database(db, sv(DB));
        release_cursor(cur);
        let cur = create_collection(db, sv(DB), sv(TBL));
        release_cursor(cur);

        match mode {
            "update" => {
                seed(db, SEED_SMALL);
                run_update(db);
            }
            "delete" => {
                seed(db, SEED_MUTATE);
                run_delete(db);
            }
            _ => unreachable!(),
        }

        otterbrix_destroy(db);
    }
}
