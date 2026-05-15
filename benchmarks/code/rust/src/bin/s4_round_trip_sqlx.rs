use otterbrix_bench::csv;
use otterbrix_bench::data::{bench_rows, lookup_ids};
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    CREATE_TABLE_SQL, DB, N_ROUND_TRIP, N_WARMUP, SEED_SMALL, TBL,
};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_core::row::Row;
use sqlx_otterbrix::{Otterbrix, OtterbrixConnectOptions};
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s4_sqlx_");
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
        for r in bench_rows(SEED_SMALL) {
            sqlx_core::query::query::<Otterbrix>(&insert_sql)
                .bind(r.id)
                .bind(r.name)
                .bind(r.x)
                .execute(&mut conn)
                .await
                .expect("seed insert");
        }
    });

    let select_sql = format!("SELECT id FROM {DB}.{TBL} WHERE id = ?;");
    let ids = lookup_ids(N_ROUND_TRIP + N_WARMUP, SEED_SMALL as i64);

    rt.block_on(async {
        for &id in &ids[..N_WARMUP] {
            let row = sqlx_core::query::query::<Otterbrix>(&select_sql)
                .bind(id)
                .fetch_one(&mut conn)
                .await
                .expect("warmup select");
            let _id_out: i64 = row.try_get("id").unwrap();
        }
    });

    let mut samples = Vec::with_capacity(N_ROUND_TRIP);
    for &id in &ids[N_WARMUP..] {
        let t0 = Instant::now();
        rt.block_on(async {
            let row = sqlx_core::query::query::<Otterbrix>(&select_sql)
                .bind(id)
                .fetch_one(&mut conn)
                .await
                .expect("select");
            let _id_out: i64 = row.try_get("id").unwrap();
        });
        samples.push(t0.elapsed().as_nanos());
    }

    csv::write_samples("s4_round_trip_sqlx", &samples);
}
