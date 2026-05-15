pub const DB: &str = "bench";
pub const TBL: &str = "t";
pub const TBL2: &str = "u";

pub const CREATE_TABLE_SQL: &str =
    "CREATE TABLE bench.t (id bigint, name string, x double);";

pub const SEED_SMALL: usize = 1_000;
pub const SEED_LARGE: usize = 10_000;
pub const SEED_JOIN: usize = 500;

pub const N_WARMUP: usize = 1_000;

pub const N_HEADLINE: usize = 25_000;

pub const N_INSERT: usize = 10_000;

pub const N_RANGE_K: &[i64] = &[1, 100, 1_000, 10_000];
pub const N_RANGE_RUNS: usize = 125;
pub const N_RANGE_WARMUP: usize = 250;

pub const N_ROUND_TRIP: usize = 10_000;

pub const N_BULK: usize = 10_000;
pub const N_BULK_RUNS: usize = 100;
pub const N_BULK_WARMUP: usize = 10;

pub const N_OPEN: usize = 200;
pub const N_OPEN_WARMUP: usize = 20;

pub const N_INTERACTIVE: usize = 1_000;

pub const N_AGG_RUNS: usize = 1_000;

pub const N_JOIN_RUNS: usize = 125;
pub const N_JOIN_WARMUP: usize = 250;

pub const N_UPDATE: usize = 10_000;
pub const N_DELETE: usize = 5_000;
pub const SEED_MUTATE: usize = SEED_SMALL + N_DELETE + N_WARMUP;
