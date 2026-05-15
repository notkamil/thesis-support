use otterbrix::Database;
use otterbrix_bench::csv;
use otterbrix_bench::data::bench_rows;
use otterbrix_bench::scenarios::{
    DB, N_DELETE, N_UPDATE, N_WARMUP, SEED_MUTATE, SEED_SMALL, TBL,
};
use otterbrix_bench::{bench_config, fresh_workdir};
use sea_orm::sea_query::Value as SeaValue;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, ProxyDatabaseTrait, Statement,
};
use seaorm_otterbrix::OtterbrixProxy;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

fn parse_mode() -> &'static str {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: s10_mutate_seaorm <update|delete>");
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

fn seed(rt: &Runtime, conn: &DatabaseConnection, n: usize) {
    let insert_sql =
        format!("INSERT INTO {DB}.{TBL} (id, name, x) VALUES ($1, $2, $3);");
    rt.block_on(async {
        for r in bench_rows(n) {
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &insert_sql,
                vec![
                    SeaValue::BigInt(Some(r.id)),
                    SeaValue::String(Some(Box::new(r.name))),
                    SeaValue::Double(Some(r.x)),
                ],
            ))
            .await
            .expect("seed insert");
        }
    });
}

fn run_update(rt: &Runtime, conn: &DatabaseConnection) {
    let update_sql = format!("UPDATE {DB}.{TBL} SET x = $1 WHERE id = $2;");

    rt.block_on(async {
        for i in 0..N_WARMUP {
            let id = (i % SEED_SMALL) as i64 + 1;
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &update_sql,
                vec![
                    SeaValue::Double(Some(i as f64 * 0.5)),
                    SeaValue::BigInt(Some(id)),
                ],
            ))
            .await
            .expect("warmup update");
        }
    });

    let mut samples = Vec::with_capacity(N_UPDATE);
    for i in 0..N_UPDATE {
        let id = ((i + N_WARMUP) % SEED_SMALL) as i64 + 1;
        let t0 = Instant::now();
        rt.block_on(async {
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &update_sql,
                vec![
                    SeaValue::Double(Some(i as f64 * 0.5)),
                    SeaValue::BigInt(Some(id)),
                ],
            ))
            .await
            .expect("update");
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_seaorm_update", &samples);
}

fn run_delete(rt: &Runtime, conn: &DatabaseConnection) {
    let delete_sql = format!("DELETE FROM {DB}.{TBL} WHERE id = $1;");

    rt.block_on(async {
        for i in 0..N_WARMUP {
            let id = (SEED_SMALL + i + 1) as i64;
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &delete_sql,
                vec![SeaValue::BigInt(Some(id))],
            ))
            .await
            .expect("warmup delete");
        }
    });

    let mut samples = Vec::with_capacity(N_DELETE);
    for i in 0..N_DELETE {
        let id = (SEED_SMALL + N_WARMUP + i + 1) as i64;
        let t0 = Instant::now();
        rt.block_on(async {
            conn.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &delete_sql,
                vec![SeaValue::BigInt(Some(id))],
            ))
            .await
            .expect("delete");
        });
        samples.push(t0.elapsed().as_nanos());
    }
    csv::write_samples("s10_mutate_seaorm_delete", &samples);
}

fn main() {
    let mode = parse_mode();
    let workdir = fresh_workdir(&format!("s10_seaorm_{mode}_"));
    let db = Database::open(bench_config(workdir.path())).expect("open");
    db.create_database(DB).expect("create db");
    db.create_collection(DB, TBL).expect("create collection");
    let proxy: Arc<Box<dyn ProxyDatabaseTrait>> =
        Arc::new(Box::new(OtterbrixProxy::new(db)));
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let conn: DatabaseConnection = rt
        .block_on(async {
            sea_orm::Database::connect_proxy(DbBackend::Sqlite, proxy).await
        })
        .expect("connect_proxy");

    match mode {
        "update" => {
            seed(&rt, &conn, SEED_SMALL);
            run_update(&rt, &conn);
        }
        "delete" => {
            seed(&rt, &conn, SEED_MUTATE);
            run_delete(&rt, &conn);
        }
        _ => unreachable!(),
    }
}
