use otterbrix::{Database, SqlParam, SqlParamValue};
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    DB, N_RANGE_K, N_RANGE_RUNS, N_RANGE_WARMUP, SEED_LARGE, TBL,
};
use std::time::Instant;

fn main() {
    let workdir = fresh_workdir("s3_ob_");
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create collection");

    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    for r in bench_rows(SEED_LARGE) {
        db.execute_with_params(
            &insert_sql,
            &[
                SqlParam {
                    index: 1,
                    value: SqlParamValue::Int64(r.id),
                },
                SqlParam {
                    index: 2,
                    value: SqlParamValue::Str(&r.name),
                },
                SqlParam {
                    index: 3,
                    value: SqlParamValue::Double(r.x),
                },
            ],
        )
        .expect("seed insert");
    }

    let select_sql = format!(
        "SELECT id, name, x FROM {DB}.{TBL} WHERE id BETWEEN 1 AND $1;"
    );

    for &k in N_RANGE_K {
        for _ in 0..N_RANGE_WARMUP {
            let cur = db
                .execute_with_params(
                    &select_sql,
                    &[SqlParam {
                        index: 1,
                        value: SqlParamValue::Int64(k),
                    }],
                )
                .expect("warmup select");
            for row in cur.rows() {
                let _id: i64 = row.get_by_name("id").get().unwrap();
            }
        }

        let mut samples = Vec::with_capacity(N_RANGE_RUNS);
        for _ in 0..N_RANGE_RUNS {
            let t0 = Instant::now();
            let cur = db
                .execute_with_params(
                    &select_sql,
                    &[SqlParam {
                        index: 1,
                        value: SqlParamValue::Int64(k),
                    }],
                )
                .expect("select");
            let mut sink: i64 = 0;
            for row in cur.rows() {
                let id: i64 = row.get_by_name("id").get().unwrap();
                let name: String = row.get_by_name("name").get().unwrap();
                let x: f64 = row.get_by_name("x").get().unwrap();
                sink = sink.wrapping_add(id).wrapping_add(name.len() as i64);
                std::hint::black_box(x);
            }
            std::hint::black_box(sink);
            samples.push(t0.elapsed().as_nanos());
        }
        csv::write_samples(&format!("s3_range_scan_otterbrix_k{k}"), &samples);
    }
}
