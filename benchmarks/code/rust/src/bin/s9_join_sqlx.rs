use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    DB, N_JOIN_RUNS, N_JOIN_WARMUP, SEED_JOIN, TBL, TBL2,
};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_core::row::Row;
use sqlx_otterbrix::{Otterbrix, OtterbrixConnectOptions};
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s12_sqlx_");
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
        conn.execute(
            format!("CREATE TABLE {DB}.{TBL} (id bigint, name string, x double);")
                .as_str(),
        )
        .await
        .expect("create t");
        conn.execute(
            format!("CREATE TABLE {DB}.{TBL2} (id bigint, name string, x double);")
                .as_str(),
        )
        .await
        .expect("create u");

        let insert_t =
            format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES (?, ?, ?);");
        let insert_u =
            format!("INSERT INTO {DB}.{TBL2} (id, name, x) VALUES (?, ?, ?);");
        for r in bench_rows(SEED_JOIN) {
            sqlx_core::query::query::<Otterbrix>(&insert_t)
                .bind(r.id)
                .bind(r.name.clone())
                .bind(r.x)
                .execute(&mut conn)
                .await
                .expect("seed t");
            sqlx_core::query::query::<Otterbrix>(&insert_u)
                .bind(r.id)
                .bind(r.name)
                .bind(r.x)
                .execute(&mut conn)
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
            let _ = sqlx_core::query::query::<Otterbrix>(&join_sql)
                .fetch_all(&mut conn)
                .await
                .expect("warmup join");
        }
    });

    let mut samples = Vec::with_capacity(N_JOIN_RUNS);
    for _ in 0..N_JOIN_RUNS {
        let t0 = Instant::now();
        rt.block_on(async {
            let rows = sqlx_core::query::query::<Otterbrix>(&join_sql)
                .fetch_all(&mut conn)
                .await
                .expect("join");
            let mut sink: i64 = 0;
            for row in &rows {
                let id: i64 = row.try_get(0).unwrap();
                let name: String = row.try_get(1).unwrap();
                let x: f64 = row.try_get(2).unwrap();
                sink = sink
                    .wrapping_add(id)
                    .wrapping_add(name.len() as i64)
                    .wrapping_add(x as i64);
            }
            std::hint::black_box(sink);
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s9_join_sqlx", &samples);
}
