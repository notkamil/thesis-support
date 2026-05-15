use otterbrix_bench::csv;
use otterbrix_bench::data::{bench_rows, lookup_ids};
use otterbrix_bench::fresh_workdir;
use otterbrix_bench::scenarios::{DB, N_ROUND_TRIP, N_WARMUP, SEED_SMALL, TBL};
use otterbrix_sys::*;
use std::time::Instant;

fn sv(s: &str) -> string_view_t {
    string_view_t {
        data: s.as_ptr() as *const i8,
        size: s.len(),
    }
}

fn main() {
    let workdir = fresh_workdir("s4_sys_");
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

        let insert_sql =
            format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
        for r in bench_rows(SEED_SMALL) {
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

        let select_sql = format!("SELECT id FROM {DB}.{TBL} WHERE id = $1;");
        let select_sv = sv(&select_sql);
        let ids = lookup_ids(N_ROUND_TRIP + N_WARMUP, SEED_SMALL as i64);

        for &id in &ids[..N_WARMUP] {
            let p = [sql_param_t {
                index: 1,
                kind: sql_param_kind_t_SQL_PARAM_INT64,
                bool_value: 0,
                int64_value: id,
                uint64_value: 0,
                double_value: 0.0,
                string_value: sv(""),
            }];
            let cur = execute_sql_params(db, select_sv, p.as_ptr(), p.len());
            release_cursor(cur);
        }

        let mut samples = Vec::with_capacity(N_ROUND_TRIP);
        for &id in &ids[N_WARMUP..] {
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
            let cur = execute_sql_params(db, select_sv, p.as_ptr(), p.len());
            let v = cursor_get_value(cur, 0, 0);
            let _id_out = value_get_int(v);
            release_value(v);
            release_cursor(cur);
            samples.push(t0.elapsed().as_nanos());
        }

        otterbrix_destroy(db);
        csv::write_samples("s4_round_trip_sys", &samples);
    }
}
