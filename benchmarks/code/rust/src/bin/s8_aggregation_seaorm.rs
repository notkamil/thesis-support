use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    DB, N_AGG_RUNS, N_WARMUP, SEED_LARGE, TBL,
};
use sea_orm::sea_query::Value as SeaValue;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, ProxyDatabaseTrait, Statement,
};
use seaorm_otterbrix::{positional_proxy_column_key, OtterbrixProxy};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::{Builder, Runtime};

fn measure(rt: &Runtime, conn: &DatabaseConnection, sql: &str, name: &str) {
    rt.block_on(async {
        for _ in 0..N_WARMUP {
            let _ = conn
                .query_one(Statement::from_string(
                    DbBackend::Postgres,
                    sql.to_string(),
                ))
                .await
                .expect("warmup agg");
        }
    });

    let key = positional_proxy_column_key(0);
    let mut samples = Vec::with_capacity(N_AGG_RUNS);
    for _ in 0..N_AGG_RUNS {
        let t0 = Instant::now();
        rt.block_on(async {
            let raw = conn
                .query_one(Statement::from_string(
                    DbBackend::Postgres,
                    sql.to_string(),
                ))
                .await
                .expect("agg")
                .expect("row");
            let v: f64 = raw.try_get("", &key).unwrap_or(0.0);
            std::hint::black_box(v);
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples(name, &samples);
}

fn main() {
    let workdir = fresh_workdir("s8_seaorm_");
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

    measure(
        &rt,
        &conn,
        &format!("SELECT SUM(x) FROM {DB}.{TBL};"),
        "s8_aggregation_seaorm_sum",
    );
    measure(
        &rt,
        &conn,
        &format!("SELECT MAX(x) FROM {DB}.{TBL};"),
        "s8_aggregation_seaorm_max",
    );
}
