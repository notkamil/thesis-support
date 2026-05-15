#include "bench_common.h"
#include "bench_data.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INSERT_SQL "INSERT INTO " BENCH_DB "." BENCH_TBL \
                   " (id, name, x) VALUES ($1, $2, $3);"

static void measure(otterbrix_ptr db, const char *sql, const char *out_name) {
    string_view_t sql_sv = bench_sv(sql);

    for (size_t i = 0; i < N_WARMUP; ++i) {
        cursor_ptr cur = execute_sql(db, sql_sv);
        release_cursor(cur);
    }

    uint64_t *samples = malloc(N_AGG_RUNS * sizeof(uint64_t));
    for (size_t i = 0; i < N_AGG_RUNS; ++i) {
        uint64_t t0 = bench_now_ns();
        cursor_ptr cur = execute_sql(db, sql_sv);
        value_ptr v = cursor_get_value(cur, 0, 0);
        (void)value_get_double(v);
        release_value(v);
        release_cursor(cur);
        samples[i] = bench_now_ns() - t0;
    }
    bench_write_csv(out_name, samples, N_AGG_RUNS);
    free(samples);
}

int main(void) {
    char workdir[4096], log[4096], wal[4096], disk[4096], main_path[4096];
    bench_fresh_workdir("s8_c_", workdir, sizeof(workdir));
    snprintf(log,       sizeof(log),       "%s/log",  workdir);
    snprintf(wal,       sizeof(wal),       "%s/wal",  workdir);
    snprintf(disk,      sizeof(disk),      "%s/disk", workdir);
    snprintf(main_path, sizeof(main_path), "%s/main", workdir);

    config_t cfg = {
        .level = 6,
        .log_path = bench_sv(log),     .wal_path = bench_sv(wal),
        .disk_path = bench_sv(disk),   .main_path = bench_sv(main_path),
        .wal_on = false, .disk_on = false, .sync_to_disk = false,
    };
    otterbrix_ptr db = otterbrix_create(cfg);
    assert(db);
    cursor_ptr cur = create_database(db, bench_sv(BENCH_DB));
    release_cursor(cur);
    cur = create_collection(db, bench_sv(BENCH_DB), bench_sv(BENCH_TBL));
    release_cursor(cur);

    bench_row_t *rows = malloc(SEED_LARGE * sizeof(bench_row_t));
    bench_load_rows(rows, SEED_LARGE);
    string_view_t insert_sv = bench_sv(INSERT_SQL);
    for (size_t i = 0; i < SEED_LARGE; ++i) {
        sql_param_t p[3] = {
            { .index = 1, .kind = SQL_PARAM_INT64,  .int64_value  = rows[i].id },
            { .index = 2, .kind = SQL_PARAM_STRING,
              .string_value = bench_svn(rows[i].name, BENCH_NAME_LEN) },
            { .index = 3, .kind = SQL_PARAM_DOUBLE, .double_value = rows[i].x },
        };
        cur = execute_sql_params(db, insert_sv, p, 3);
        release_cursor(cur);
    }
    free(rows);

    measure(db, "SELECT SUM(x) FROM " BENCH_DB "." BENCH_TBL ";",
            "s8_aggregation_c_sum");
    measure(db, "SELECT MAX(x) FROM " BENCH_DB "." BENCH_TBL ";",
            "s8_aggregation_c_max");

    otterbrix_destroy(db);
    return 0;
}
