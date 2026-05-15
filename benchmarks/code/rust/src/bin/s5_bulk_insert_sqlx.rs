use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    CREATE_TABLE_SQL, DB, N_BULK, N_BULK_RUNS, N_BULK_WARMUP, TBL,
};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_otterbrix::OtterbrixConnectOptions;
use std::fmt::Write as _;
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
    let mut conn = rt
        .block_on(async {
            OtterbrixConnectOptions::from_config(
                bench_config(workdir.path()),
                workdir.path(),
            )
            .connect()
            .await
        })
        .expect("connect");
    rt.block_on(async {
        conn.execute(format!("CREATE DATABASE {DB};").as_str())
            .await
            .expect("create db");
        conn.execute(CREATE_TABLE_SQL).await.expect("create table");
    });

    let t0 = Instant::now();
    rt.block_on(async {
        conn.execute(bulk_sql).await.expect("bulk insert");
    });
    t0.elapsed().as_nanos()
}

fn main() {
    let bulk_sql = build_bulk_sql();
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    for run in 0..N_BULK_WARMUP {
        let _ = one_run(&rt, &bulk_sql, "s5_sqlx_warm_", run);
    }

    let mut samples = Vec::with_capacity(N_BULK_RUNS);
    for run in 0..N_BULK_RUNS {
        samples.push(one_run(&rt, &bulk_sql, "s5_sqlx_", run));
    }

    csv::write_samples("s5_bulk_insert_sqlx", &samples);
}
