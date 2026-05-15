use otterbrix_bench::data::{bench_rows, lookup_ids, SEED};
use otterbrix_bench::scenarios::{
    N_BULK, N_HEADLINE, N_INSERT, N_WARMUP, SEED_LARGE, SEED_MUTATE, SEED_SMALL,
};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

const NAME_LEN: usize = 16;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("benchmarks/")
        .join("data")
}

fn write_rows(path: &std::path::Path, n: usize) {
    let rows = bench_rows(n);
    let mut w = BufWriter::new(File::create(path).expect("create rows file"));
    for r in &rows {
        w.write_all(&r.id.to_le_bytes()).unwrap();
        let bytes = r.name.as_bytes();
        assert_eq!(bytes.len(), NAME_LEN, "name must be {NAME_LEN} bytes");
        w.write_all(bytes).unwrap();
        w.write_all(&r.x.to_le_bytes()).unwrap();
    }
    w.flush().unwrap();
    println!("wrote {} rows -> {}", rows.len(), path.display());
}

fn write_ids(path: &std::path::Path, n: usize, max_id: i64) {
    let ids = lookup_ids(n, max_id);
    let mut w = BufWriter::new(File::create(path).expect("create ids file"));
    for &id in &ids {
        w.write_all(&id.to_le_bytes()).unwrap();
    }
    w.flush().unwrap();
    println!("wrote {} ids -> {}", ids.len(), path.display());
}

fn main() {
    let dir = data_dir();
    fs::create_dir_all(&dir).expect("create data/");

    let max_rows = [
        SEED_SMALL,
        SEED_LARGE,
        N_BULK,
        N_INSERT + N_WARMUP,
        SEED_MUTATE,
    ]
    .into_iter()
    .max()
    .unwrap();
    println!("rows seed={:#018x}, max n = {}", SEED, max_rows);
    write_rows(&dir.join("rows_max.bin"), max_rows);

    let max_ids = N_HEADLINE + N_WARMUP;
    println!(
        "ids seed=SEED^1, max n = {}, max_id = {}",
        max_ids, SEED_SMALL
    );
    write_ids(
        &dir.join("lookup_ids_max.bin"),
        max_ids,
        SEED_SMALL as i64,
    );

    println!("done; data dir: {}", dir.display());
}
