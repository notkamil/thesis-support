use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::data::{bench_rows, lookup_ids};
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{DB, N_ROUND_TRIP, N_WARMUP, SEED_SMALL, TBL};
use sea_orm::sea_query::Value as SeaValue;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, ProxyDatabaseTrait, Statement,
};
use seaorm_otterbrix::OtterbrixProxy;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s4_seaorm_");
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
        for r in bench_rows(SEED_SMALL) {
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

    let select_sql = format!("SELECT id FROM {DB}.{TBL} WHERE id = $1;");
    let ids = lookup_ids(N_ROUND_TRIP + N_WARMUP, SEED_SMALL as i64);

    rt.block_on(async {
        for &id in &ids[..N_WARMUP] {
            let raw = conn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    &select_sql,
                    vec![SeaValue::BigInt(Some(id))],
                ))
                .await
                .expect("warmup select")
                .expect("row");
            let _id_out: i64 = raw.try_get("", "id").unwrap();
        }
    });

    let mut samples = Vec::with_capacity(N_ROUND_TRIP);
    for &id in &ids[N_WARMUP..] {
        let t0 = Instant::now();
        rt.block_on(async {
            let raw = conn
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    &select_sql,
                    vec![SeaValue::BigInt(Some(id))],
                ))
                .await
                .expect("select")
                .expect("row");
            let _id_out: i64 = raw.try_get("", "id").unwrap();
        });
        samples.push(t0.elapsed().as_nanos());
    }

    csv::write_samples("s4_round_trip_seaorm", &samples);
}
