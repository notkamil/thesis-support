use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    CREATE_TABLE_SQL, DB, N_AGG_RUNS, N_WARMUP, SEED_LARGE, TBL,
};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_core::row::Row;
use sqlx_otterbrix::{Otterbrix, OtterbrixConnection, OtterbrixConnectOptions};
use std::time::Instant;
use tokio::runtime::{Builder, Runtime};

fn measure(rt: &Runtime, conn: &mut OtterbrixConnection, sql: &str, name: &str) {
    rt.block_on(async {
        for _ in 0..N_WARMUP {
            let _ = sqlx_core::query::query::<Otterbrix>(sql)
                .fetch_one(&mut *conn)
                .await
                .expect("warmup agg");
        }
    });

    let mut samples = Vec::with_capacity(N_AGG_RUNS);
    for _ in 0..N_AGG_RUNS {
        let t0 = Instant::now();
        rt.block_on(async {
            let row = sqlx_core::query::query::<Otterbrix>(sql)
                .fetch_one(&mut *conn)
                .await
                .expect("agg");

            let v: f64 = row.try_get(0).unwrap_or(0.0);
            std::hint::black_box(v);
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples(name, &samples);
}

fn main() {
    let workdir = fresh_workdir("s8_sqlx_");
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

        let insert_sql =
            format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES (?, ?, ?);");
        for r in bench_rows(SEED_LARGE) {
            sqlx_core::query::query::<Otterbrix>(&insert_sql)
                .bind(r.id)
                .bind(r.name)
                .bind(r.x)
                .execute(&mut conn)
                .await
                .expect("seed insert");
        }
    });

    measure(
        &rt,
        &mut conn,
        &format!("SELECT SUM(x) FROM {DB}.{TBL};"),
        "s8_aggregation_sqlx_sum",
    );
    measure(
        &rt,
        &mut conn,
        &format!("SELECT MAX(x) FROM {DB}.{TBL};"),
        "s8_aggregation_sqlx_max",
    );
}
