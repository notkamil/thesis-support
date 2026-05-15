use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{DB, N_BULK, N_BULK_RUNS, N_BULK_WARMUP, TBL};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, ProxyDatabaseTrait, Statement,
};
use seaorm_otterbrix::OtterbrixProxy;
use std::fmt::Write as _;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::{Builder, Runtime};

fn build_bulk_sql() -> String {
    let mut s = String::with_capacity(N_BULK * 60);
    write!(s, "INSERT INTO {DB}.{TBL} (id, name, x) VALUES ").unwrap();
    for (i, r) in bench_rows(N_BULK).into_iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        write!(s, "({}, '{}', {})", r.id, r.name, r.x).unwrap();
    }
    s.push(';');
    s
}

fn one_run(rt: &Runtime, bulk_sql: &str, prefix: &str, run: usize) -> u128 {
    let workdir = fresh_workdir(&format!("{prefix}{run}_"));
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create collection");
    let proxy: Arc<Box<dyn ProxyDatabaseTrait>> =
        Arc::new(Box::new(OtterbrixProxy::new(db)));
    let conn: DatabaseConnection = rt
        .block_on(async {
            sea_orm::Database::connect_proxy(DbBackend::Sqlite, proxy).await
        })
        .expect("connect_proxy");

    let t0 = Instant::now();
    rt.block_on(async {
        conn.execute(Statement::from_string(
            DbBackend::Postgres,
            bulk_sql.to_string(),
        ))
        .await
        .expect("bulk insert");
    });
    t0.elapsed().as_nanos()
}

fn main() {
    let bulk_sql = build_bulk_sql();
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    for run in 0..N_BULK_WARMUP {
        let _ = one_run(&rt, &bulk_sql, "s5_seaorm_warm_", run);
    }

    let mut samples = Vec::with_capacity(N_BULK_RUNS);
    for run in 0..N_BULK_RUNS {
        samples.push(one_run(&rt, &bulk_sql, "s5_seaorm_", run));
    }

    csv::write_samples("s5_bulk_insert_seaorm", &samples);
}
