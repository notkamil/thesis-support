use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    DB, N_RANGE_K, N_RANGE_RUNS, N_RANGE_WARMUP, SEED_LARGE, TBL,
};
use sea_orm::sea_query::Value as SeaValue;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, ProxyDatabaseTrait, Statement,
};
use seaorm_otterbrix::OtterbrixProxy;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s3_seaorm_");
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create collection");
    let proxy: Arc<Box<dyn ProxyDatabaseTrait>> =
        Arc::new(Box::new(OtterbrixProxy::new(db)));
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let conn: DatabaseConnection = rt
        .block_on(async {
            sea_orm::Database::connect_proxy(DbBackend::Sqlite, proxy).await
        })
        .expect("connect_proxy");

    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    rt.block_on(async {
        for r in bench_rows(SEED_LARGE) {
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &insert_sql,
                vec![
                    SeaValue::BigInt(Some(r.id)),
                    SeaValue::String(Some(Box::new(r.name))),
                    SeaValue::Double(Some(r.x)),
                ],
            ))
            .await
            .expect("seed insert");
        }
    });

    let select_sql = format!(
        "SELECT id, name, x FROM {DB}.{TBL} WHERE id BETWEEN 1 AND $1;"
    );

    for &k in N_RANGE_K {
        rt.block_on(async {
            for _ in 0..N_RANGE_WARMUP {
                let rows = conn
                    .query_all(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        &select_sql,
                        vec![SeaValue::BigInt(Some(k))],
                    ))
                    .await
                    .expect("warmup select");
                for raw in &rows {
                    let _id: i64 = raw.try_get("", "id").unwrap();
                }
            }
        });

        let mut samples = Vec::with_capacity(N_RANGE_RUNS);
        for _ in 0..N_RANGE_RUNS {
            let t0 = Instant::now();
            rt.block_on(async {
                let rows = conn
                    .query_all(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        &select_sql,
                        vec![SeaValue::BigInt(Some(k))],
                    ))
                    .await
                    .expect("select");
                let mut sink: i64 = 0;
                for raw in &rows {
                    let id: i64 = raw.try_get("", "id").unwrap();
                    let name: String = raw.try_get("", "name").unwrap();
                    let x: f64 = raw.try_get("", "x").unwrap();
                    sink = sink.wrapping_add(id).wrapping_add(name.len() as i64);
                    std::hint::black_box(x);
                }
                std::hint::black_box(sink);
            });
            samples.push(t0.elapsed().as_nanos());
        }
        csv::write_samples(&format!("s3_range_scan_seaorm_k{k}"), &samples);
    }
}
