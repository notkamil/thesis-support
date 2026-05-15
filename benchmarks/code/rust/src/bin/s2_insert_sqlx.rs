use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::scenarios::{CREATE_TABLE_SQL, DB, N_INSERT, N_WARMUP, TBL};
use otterbrix_bench::{bench_config, fresh_workdir};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_otterbrix::{Otterbrix, OtterbrixConnectOptions};
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s2_sqlx_");
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
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

    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES (?, ?, ?);");
    let rows = bench_rows(N_INSERT + N_WARMUP);

    rt.block_on(async {
        for r in &rows[..N_WARMUP] {
            sqlx_core::query::query::<Otterbrix>(&insert_sql)
                .bind(r.id)
                .bind(r.name.clone())
                .bind(r.x)
                .execute(&mut conn)
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
            sqlx_core::query::query::<Otterbrix>(&insert_sql)
                .bind(id)
                .bind(name)
                .bind(x)
                .execute(&mut conn)
                .await
                .expect("insert");
        });
        samples.push(t0.elapsed().as_nanos());
    }

    csv::write_samples("s2_insert_sqlx", &samples);
}
