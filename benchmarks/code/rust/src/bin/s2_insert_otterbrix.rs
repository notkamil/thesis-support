use otterbrix::{Database, SqlParam, SqlParamValue};
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::scenarios::{DB, N_INSERT, N_WARMUP, TBL};
use otterbrix_bench::{bench_config, fresh_workdir};
use std::time::Instant;

fn main() {
    let workdir = fresh_workdir("s2_ob_");
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create collection");

    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    let rows = bench_rows(N_INSERT + N_WARMUP);

    for r in &rows[..N_WARMUP] {
        db.execute_with_params(
            &insert_sql,
            &[
                SqlParam { index: 1, value: SqlParamValue::Int64(r.id) },
                SqlParam { index: 2, value: SqlParamValue::Str(&r.name) },
                SqlParam { index: 3, value: SqlParamValue::Double(r.x) },
            ],
        )
        .expect("warmup insert");
    }

    let mut samples = Vec::with_capacity(N_INSERT);
    for r in &rows[N_WARMUP..] {
        let t0 = Instant::now();
        db.execute_with_params(
            &insert_sql,
            &[
                SqlParam { index: 1, value: SqlParamValue::Int64(r.id) },
                SqlParam { index: 2, value: SqlParamValue::Str(&r.name) },
                SqlParam { index: 3, value: SqlParamValue::Double(r.x) },
            ],
        )
        .expect("insert");
        samples.push(t0.elapsed().as_nanos());
    }

    csv::write_samples("s2_insert_otterbrix", &samples);
}
