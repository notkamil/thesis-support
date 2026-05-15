use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    CREATE_TABLE_SQL, DB, N_RANGE_K, N_RANGE_RUNS, N_RANGE_WARMUP, SEED_LARGE, TBL,
};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_core::row::Row;
use sqlx_otterbrix::{Otterbrix, OtterbrixConnectOptions};
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s3_sqlx_");
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

    let select_sql = format!(
        "SELECT id, name, x FROM {DB}.{TBL} WHERE id BETWEEN 1 AND ?;"
    );

    for &k in N_RANGE_K {
        rt.block_on(async {
            for _ in 0..N_RANGE_WARMUP {
                let rows = sqlx_core::query::query::<Otterbrix>(&select_sql)
                    .bind(k)
                    .fetch_all(&mut conn)
                    .await
                    .expect("warmup select");
                for row in &rows {
                    let _id: i64 = row.try_get("id").unwrap();
                }
            }
        });

        let mut samples = Vec::with_capacity(N_RANGE_RUNS);
        for _ in 0..N_RANGE_RUNS {
            let t0 = Instant::now();
            rt.block_on(async {
                let rows = sqlx_core::query::query::<Otterbrix>(&select_sql)
                    .bind(k)
                    .fetch_all(&mut conn)
                    .await
                    .expect("select");
                let mut sink: i64 = 0;
                for row in &rows {
                    let id: i64 = row.try_get("id").unwrap();
                    let name: String = row.try_get("name").unwrap();
                    let x: f64 = row.try_get("x").unwrap();
                    sink = sink.wrapping_add(id).wrapping_add(name.len() as i64);
                    std::hint::black_box(x);
                }
                std::hint::black_box(sink);
            });
            samples.push(t0.elapsed().as_nanos());
        }
        csv::write_samples(&format!("s3_range_scan_sqlx_k{k}"), &samples);
    }
}
