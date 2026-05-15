use otterbrix::{Database, SqlParam, SqlParamValue};
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::scenarios::{
    DB, N_AGG_RUNS, N_WARMUP, SEED_LARGE, TBL,
};
use otterbrix_bench::{bench_config, fresh_workdir};
use std::time::Instant;

fn measure(db: &Database, sql: &str, name: &str) {
    for _ in 0..N_WARMUP {
        let _ = db.execute(sql).expect("warmup agg");
    }

    let mut samples = Vec::with_capacity(N_AGG_RUNS);
    for _ in 0..N_AGG_RUNS {
        let t0 = Instant::now();
        let cur = db.execute(sql).expect("agg");
        let _: f64 = cur.rows().next().unwrap().get(0).get().unwrap();
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples(name, &samples);
}

fn main() {
    let workdir = fresh_workdir("s8_ob_");
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

    measure(
        &db,
        &format!("SELECT SUM(x) FROM {DB}.{TBL};"),
        "s8_aggregation_otterbrix_sum",
    );
    measure(
        &db,
        &format!("SELECT MAX(x) FROM {DB}.{TBL};"),
        "s8_aggregation_otterbrix_max",
    );
}
