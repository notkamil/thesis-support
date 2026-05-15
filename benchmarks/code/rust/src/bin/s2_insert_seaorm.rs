use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::scenarios::{DB, N_INSERT, N_WARMUP, TBL};
use otterbrix_bench::{bench_config, fresh_workdir};
use sea_orm::sea_query::Value as SeaValue;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, ProxyDatabaseTrait, Statement,
};
use seaorm_otterbrix::OtterbrixProxy;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s2_seaorm_");
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
    let rows = bench_rows(N_INSERT + N_WARMUP);

    rt.block_on(async {
        for r in &rows[..N_WARMUP] {
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &insert_sql,
                vec![
                    SeaValue::BigInt(Some(r.id)),
                    SeaValue::String(Some(Box::new(r.name.clone()))),
                    SeaValue::Double(Some(r.x)),
                ],
            ))
            .await
            .expect("warmup insert");
        }
    });

    let mut samples = Vec::with_capacity(N_INSERT);
    for r in &rows[N_WARMUP..] {
        let id = r.id;
        let name = r.name.clone();
        let x = r.x;
        let t0 = Instant::now();
        rt.block_on(async {
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &insert_sql,
                vec![
                    SeaValue::BigInt(Some(id)),
                    SeaValue::String(Some(Box::new(name))),
                    SeaValue::Double(Some(x)),
                ],
            ))
            .await
            .expect("insert");
        });
        samples.push(t0.elapsed().as_nanos());
    }

    csv::write_samples("s2_insert_seaorm", &samples);
}
