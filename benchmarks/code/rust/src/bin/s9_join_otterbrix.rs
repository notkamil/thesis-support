use otterbrix::{Database, SqlParam, SqlParamValue};
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    DB, N_JOIN_RUNS, N_JOIN_WARMUP, SEED_JOIN, TBL, TBL2,
};
use std::time::Instant;

fn main() {
    let workdir = fresh_workdir("s12_ob_");
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create t");
    db.create_collection(DB, TBL2).expect("create u");

    let insert_t =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    let insert_u =
        format!("INSERT INTO {DB}.{TBL2} (id, name, x) VALUES ($1, $2, $3);");
    for r in bench_rows(SEED_JOIN) {
        let p = [
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
        ];
        db.execute_with_params(&insert_t, &p).expect("seed t");
        db.execute_with_params(&insert_u, &p).expect("seed u");
    }

    let join_sql = format!(
        "SELECT t.id, t.name, u.x FROM {DB}.{TBL} AS t \
         INNER JOIN {DB}.{TBL2} AS u ON t.id = u.id;"
    );

    for _ in 0..N_JOIN_WARMUP {
        let _ = db.execute(&join_sql).expect("warmup join");
    }

    let mut samples = Vec::with_capacity(N_JOIN_RUNS);
    for _ in 0..N_JOIN_RUNS {
        let t0 = Instant::now();
        let cur = db.execute(&join_sql).expect("join");
        let mut sink: i64 = 0;
        for row in cur.rows() {
            let id: i64 = row.get(0).get().unwrap();
            let name: String = row.get(1).get().unwrap();
            let x: f64 = row.get(2).get().unwrap();
            sink = sink
                .wrapping_add(id)
                .wrapping_add(name.len() as i64)
                .wrapping_add(x as i64);
        }
        std::hint::black_box(sink);
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s9_join_otterbrix", &samples);
}
