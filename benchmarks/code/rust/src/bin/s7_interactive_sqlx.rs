use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::{bench_config, fresh_workdir};
use otterbrix_bench::scenarios::{
    CREATE_TABLE_SQL, DB, N_INTERACTIVE, N_WARMUP, SEED_SMALL, TBL,
};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_core::row::Row;
use sqlx_otterbrix::{Otterbrix, OtterbrixConnectOptions};
use std::time::Instant;
use tokio::runtime::Builder;

fn main() {
    let workdir = fresh_workdir("s7_sqlx_");
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

    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES (?, ?, ?);");
    let select_sql = format!("SELECT name FROM {DB}.{TBL} WHERE id = ?;");
    let total = N_INTERACTIVE + N_WARMUP;
    let extra = bench_rows(SEED_SMALL + total);

    rt.block_on(async {
        for r in &extra[SEED_SMALL..SEED_SMALL + N_WARMUP] {
            sqlx_core::query::query::<Otterbrix>(&insert_sql)
                .bind(r.id)
                .bind(&r.name)
                .bind(r.x)
                .execute(&mut conn)
                .await
                .expect("warmup insert");
            let row = sqlx_core::query::query::<Otterbrix>(&select_sql)
                .bind(r.id)
                .fetch_one(&mut conn)
                .await
                .expect("warmup select");
            let _name: String = row.try_get("name").unwrap();
        }
    });

    let mut samples = Vec::with_capacity(N_INTERACTIVE);
    for r in &extra[SEED_SMALL + N_WARMUP..] {
        let id = r.id;
        let name = r.name.clone();
        let x = r.x;
        let t0 = Instant::now();
        rt.block_on(async {
            sqlx_core::query::query::<Otterbrix>(&insert_sql)
                .bind(id)
                .bind(&name)
                .bind(x)
                .execute(&mut conn)
                .await
                .expect("insert");
            let row = sqlx_core::query::query::<Otterbrix>(&select_sql)
                .bind(id)
                .fetch_one(&mut conn)
                .await
                .expect("select");
            let _name: String = row.try_get("name").unwrap();
        });
        samples.push(t0.elapsed().as_nanos());
    }

    csv::write_samples("s7_interactive_sqlx", &samples);
}
