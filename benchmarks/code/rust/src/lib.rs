pub mod csv;
pub mod data;
pub mod scenarios;

use otterbrix::Config;
use std::path::{Path, PathBuf};

pub const LOG_OFF: i32 = 6;

#[must_use]
pub fn bench_config(base: impl AsRef<Path>) -> Config {
    let base = base.as_ref();
    Config::builder()
        .level(LOG_OFF)
        .log_path(base.join("log"))
        .wal_path(base.join("wal"))
        .disk_path(base.join("disk"))
        .main_path(base.join("main"))
        .wal_on(false)
        .disk_on(false)
        .sync_to_disk(false)
        .build()
}

#[must_use]
pub fn dist_root() -> PathBuf {
    let lib = PathBuf::from(env!("OTTERBRIX_LIB_DIR"));
    lib.parent()
        .expect("OTTERBRIX_LIB_DIR has a parent")
        .to_path_buf()
}

pub fn fresh_workdir(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .expect("tempdir for bench scenario")
}
