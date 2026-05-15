use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    DB, N_JOIN_RUNS, N_JOIN_WARMUP, SEED_JOIN, TBL, TBL2,
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
    let workdir = fresh_workdir("s12_seaorm_");
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create t");
    db.create_collection(DB, TBL2).expect("create u");
    let proxy: Arc<Box<dyn ProxyDatabaseTrait>> =
        Arc::new(Box::new(OtterbrixProxy::new(db)));
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let conn: DatabaseConnection = rt
        .block_on(async {
            sea_orm::Database::connect_proxy(DbBackend::Sqlite, proxy).await
        })
        .expect("connect_proxy");

    let insert_t =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    let insert_u =
        format!("INSERT INTO {DB}.{TBL2} (id, name, x) VALUES ($1, $2, $3);");
    rt.block_on(async {
        for r in bench_rows(SEED_JOIN) {
            let vals = vec![
                SeaValue::BigInt(Some(r.id)),
                SeaValue::String(Some(Box::new(r.name.clone()))),
                SeaValue::Double(Some(r.x)),
            ];
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &insert_t,
                vals.clone(),
            ))
            .await
            .expect("seed t");
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &insert_u,
                vals,
            ))
            .await
            .expect("seed u");
        }
    });

    let join_sql = format!(
        "SELECT t.id, t.name, u.x FROM {DB}.{TBL} AS t \
         INNER JOIN {DB}.{TBL2} AS u ON t.id = u.id;"
    );

    rt.block_on(async {
        for _ in 0..N_JOIN_WARMUP {
            let _ = conn
                .query_all(Statement::from_string(
                    DbBackend::Postgres,
                    join_sql.clone(),
                ))
                .await
                .expect("warmup join");
        }
    });

    let mut samples = Vec::with_capacity(N_JOIN_RUNS);
    for _ in 0..N_JOIN_RUNS {
        let t0 = Instant::now();
        rt.block_on(async {
            let rows = conn
                .query_all(Statement::from_string(
                    DbBackend::Postgres,
                    join_sql.clone(),
                ))
                .await
                .expect("join");
            let mut sink: i64 = 0;
            for raw in &rows {
                let id: i64 = raw.try_get("", "id").unwrap();
                let name: String = raw.try_get("", "name").unwrap();
                let x: f64 = raw.try_get("", "x").unwrap();
                sink = sink
                    .wrapping_add(id)
                    .wrapping_add(name.len() as i64)
                    .wrapping_add(x as i64);
            }
            std::hint::black_box(sink);
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s9_join_seaorm", &samples);
}
