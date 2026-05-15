#include "bench_common.h"
#include "bench_data.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INSERT_T "INSERT INTO " BENCH_DB "." BENCH_TBL \
                 " (id, name, x) VALUES ($1, $2, $3);"
#define INSERT_U "INSERT INTO " BENCH_DB "." BENCH_TBL2 \
                 " (id, name, x) VALUES ($1, $2, $3);"
#define JOIN_SQL "SELECT t.id, t.name, u.x FROM " BENCH_DB "." BENCH_TBL " AS t " \
                 "INNER JOIN " BENCH_DB "." BENCH_TBL2 " AS u ON t.id = u.id;"

int main(void) {
    char workdir[4096], log[4096], wal[4096], disk[4096], main_path[4096];
    bench_fresh_workdir("s9_c_", workdir, sizeof(workdir));
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
    cur = create_collection(db, bench_sv(BENCH_DB), bench_sv(BENCH_TBL2));
    release_cursor(cur);

    bench_row_t *rows = malloc(SEED_JOIN * sizeof(bench_row_t));
    bench_load_rows(rows, SEED_JOIN);
    string_view_t insert_t = bench_sv(INSERT_T);
    string_view_t insert_u = bench_sv(INSERT_U);
    for (size_t i = 0; i < SEED_JOIN; ++i) {
        sql_param_t p[3] = {
            { .index = 1, .kind = SQL_PARAM_INT64,  .int64_value  = rows[i].id },
            { .index = 2, .kind = SQL_PARAM_STRING,
              .string_value = bench_svn(rows[i].name, BENCH_NAME_LEN) },
            { .index = 3, .kind = SQL_PARAM_DOUBLE, .double_value = rows[i].x },
        };
        cur = execute_sql_params(db, insert_t, p, 3);
        release_cursor(cur);
        cur = execute_sql_params(db, insert_u, p, 3);
        release_cursor(cur);
    }
    free(rows);

    string_view_t join_sv = bench_sv(JOIN_SQL);
    for (size_t i = 0; i < N_JOIN_WARMUP; ++i) {
        cur = execute_sql(db, join_sv);
        release_cursor(cur);
    }

    uint64_t *samples = malloc(N_JOIN_RUNS * sizeof(uint64_t));
    for (size_t i = 0; i < N_JOIN_RUNS; ++i) {
        uint64_t t0 = bench_now_ns();
        cur = execute_sql(db, join_sv);
        int32_t n = cursor_size(cur);
        volatile int64_t sink = 0;
        for (int32_t r = 0; r < n; ++r) {
            value_ptr v = cursor_get_value(cur, r, 0);
            sink += value_get_int(v);
            release_value(v);
            v = cursor_get_value(cur, r, 1);
            char *raw = value_get_string(v);
            char *owned = strdup(raw);
            sink += (int64_t)strlen(owned);
            free(owned);
            release_value(v);
            v = cursor_get_value(cur, r, 2);
            sink += (int64_t)value_get_double(v);
            release_value(v);
        }
        release_cursor(cur);
        (void)sink;
        samples[i] = bench_now_ns() - t0;
    }
    bench_write_csv("s9_join_c", samples, N_JOIN_RUNS);
    free(samples);

    otterbrix_destroy(db);
    return 0;
}
