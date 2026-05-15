use otterbrix::{Database, SqlParam, SqlParamValue};
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::scenarios::{
    DB, N_DELETE, N_UPDATE, N_WARMUP, SEED_MUTATE, SEED_SMALL, TBL,
};
use otterbrix_bench::{bench_config, fresh_workdir};
use std::time::Instant;

fn parse_mode() -> &'static str {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: s10_mutate_otterbrix <update|delete>");
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

fn seed(db: &Database, n: usize) {
    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    for r in bench_rows(n) {
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
}

fn run_update(db: &Database) {
    let update_sql = format!("UPDATE {DB}.{TBL} SET x = $1 WHERE id = $2;");

    for i in 0..N_WARMUP {
        let id = (i % SEED_SMALL) as i64 + 1;
        db.execute_with_params(
            &update_sql,
            &[
                SqlParam {
                    index: 1,
                    value: SqlParamValue::Double(i as f64 * 0.5),
                },
                SqlParam {
                    index: 2,
                    value: SqlParamValue::Int64(id),
                },
            ],
        )
        .expect("warmup update");
    }

    let mut samples = Vec::with_capacity(N_UPDATE);
    for i in 0..N_UPDATE {
        let id = ((i + N_WARMUP) % SEED_SMALL) as i64 + 1;
        let t0 = Instant::now();
        db.execute_with_params(
            &update_sql,
            &[
                SqlParam {
                    index: 1,
                    value: SqlParamValue::Double(i as f64 * 0.5),
                },
                SqlParam {
                    index: 2,
                    value: SqlParamValue::Int64(id),
                },
            ],
        )
        .expect("update");
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_otterbrix_update", &samples);
}

fn run_delete(db: &Database) {
    let delete_sql = format!("DELETE FROM {DB}.{TBL} WHERE id = $1;");

    for i in 0..N_WARMUP {
        let id = (SEED_SMALL + i + 1) as i64;
        db.execute_with_params(
            &delete_sql,
            &[SqlParam {
                index: 1,
                value: SqlParamValue::Int64(id),
            }],
        )
        .expect("warmup delete");
    }

    let mut samples = Vec::with_capacity(N_DELETE);
    for i in 0..N_DELETE {
        let id = (SEED_SMALL + N_WARMUP + i + 1) as i64;
        let t0 = Instant::now();
        db.execute_with_params(
            &delete_sql,
            &[SqlParam {
                index: 1,
                value: SqlParamValue::Int64(id),
            }],
        )
        .expect("delete");
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_otterbrix_delete", &samples);
}

fn main() {
    let mode = parse_mode();
    let workdir = fresh_workdir(&format!("s10_ob_{mode}_"));
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create collection");

    match mode {
        "update" => {
            seed(&db, SEED_SMALL);
            run_update(&db);
        }
        "delete" => {
            seed(&db, SEED_MUTATE);
            run_delete(&db);
        }
        _ => unreachable!(),
    }
}
