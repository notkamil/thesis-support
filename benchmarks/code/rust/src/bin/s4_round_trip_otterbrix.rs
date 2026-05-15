use otterbrix::{Database, SqlParam, SqlParamValue};
use otterbrix_bench::csv;
use otterbrix_bench::data::{bench_rows, lookup_ids};
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{DB, N_ROUND_TRIP, N_WARMUP, SEED_SMALL, TBL};
use std::time::Instant;

fn main() {
    let workdir = fresh_workdir("s4_ob_");
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create collection");

    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    for r in bench_rows(SEED_SMALL) {
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

    let select_sql = format!("SELECT id FROM {DB}.{TBL} WHERE id = $1;");
    let ids = lookup_ids(N_ROUND_TRIP + N_WARMUP, SEED_SMALL as i64);

    for &id in &ids[..N_WARMUP] {
        let cur = db
            .execute_with_params(
                &select_sql,
                &[SqlParam {
                    index: 1,
                    value: SqlParamValue::Int64(id),
                }],
            )
            .expect("warmup select");
        let _id_out: i64 = cur.rows().next().unwrap().get(0).get().unwrap();
    }

    let mut samples = Vec::with_capacity(N_ROUND_TRIP);
    for &id in &ids[N_WARMUP..] {
        let t0 = Instant::now();
        let cur = db
            .execute_with_params(
                &select_sql,
                &[SqlParam {
                    index: 1,
                    value: SqlParamValue::Int64(id),
                }],
            )
            .expect("select");
        let _id_out: i64 = cur.rows().next().unwrap().get(0).get().unwrap();
        samples.push(t0.elapsed().as_nanos());
    }

    csv::write_samples("s4_round_trip_otterbrix", &samples);
}
