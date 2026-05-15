use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::scenarios::{
    CREATE_TABLE_SQL, DB, N_DELETE, N_UPDATE, N_WARMUP, SEED_MUTATE, SEED_SMALL, TBL,
};
use otterbrix_bench::{bench_config, fresh_workdir};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_otterbrix::{Otterbrix, OtterbrixConnection, OtterbrixConnectOptions};
use std::time::Instant;
use tokio::runtime::Runtime;

fn parse_mode() -> &'static str {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: s10_mutate_sqlx <update|delete>");
        std::process::exit(2);
    });
    match arg.as_str() {
        "update" => "update",
        "delete" => "delete",
        other => {
            eprintln!("invalid mode '{other}', expected update|delete");
            std::process::exit(2);
        }
    }
}

fn seed(rt: &Runtime, conn: &mut OtterbrixConnection, n: usize) {
    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES (?, ?, ?);");
    rt.block_on(async {
        for r in bench_rows(n) {
            sqlx_core::query::query::<Otterbrix>(&insert_sql)
                .bind(r.id)
                .bind(r.name)
                .bind(r.x)
                .execute(&mut *conn)
                .await
                .expect("seed insert");
        }
    });
}

fn run_update(rt: &Runtime, conn: &mut OtterbrixConnection) {
    let update_sql = format!("UPDATE {DB}.{TBL} SET x = ? WHERE id = ?;");

    rt.block_on(async {
        for i in 0..N_WARMUP {
            let id = (i % SEED_SMALL) as i64 + 1;
            sqlx_core::query::query::<Otterbrix>(&update_sql)
                .bind(i as f64 * 0.5)
                .bind(id)
                .execute(&mut *conn)
                .await
                .expect("warmup update");
        }
    });

    let mut samples = Vec::with_capacity(N_UPDATE);
    for i in 0..N_UPDATE {
        let id = ((i + N_WARMUP) % SEED_SMALL) as i64 + 1;
        let t0 = Instant::now();
        rt.block_on(async {
            sqlx_core::query::query::<Otterbrix>(&update_sql)
                .bind(i as f64 * 0.5)
                .bind(id)
                .execute(&mut *conn)
                .await
                .expect("update");
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_sqlx_update", &samples);
}

fn run_delete(rt: &Runtime, conn: &mut OtterbrixConnection) {
    let delete_sql = format!("DELETE FROM {DB}.{TBL} WHERE id = ?;");

    rt.block_on(async {
        for i in 0..N_WARMUP {
            let id = (SEED_SMALL + i + 1) as i64;
            sqlx_core::query::query::<Otterbrix>(&delete_sql)
                .bind(id)
                .execute(&mut *conn)
                .await
                .expect("warmup delete");
        }
    });

    let mut samples = Vec::with_capacity(N_DELETE);
    for i in 0..N_DELETE {
        let id = (SEED_SMALL + N_WARMUP + i + 1) as i64;
        let t0 = Instant::now();
        rt.block_on(async {
            sqlx_core::query::query::<Otterbrix>(&delete_sql)
                .bind(id)
                .execute(&mut *conn)
                .await
                .expect("delete");
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_sqlx_delete", &samples);
}

fn main() {
    let mode = parse_mode();
    let workdir = fresh_workdir(&format!("s10_sqlx_{mode}_"));
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
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

    match mode {
        "update" => {
            seed(&rt, &mut conn, SEED_SMALL);
            run_update(&rt, &mut conn);
        }
        "delete" => {
            seed(&rt, &mut conn, SEED_MUTATE);
            run_delete(&rt, &mut conn);
        }
        _ => unreachable!(),
    }
}
