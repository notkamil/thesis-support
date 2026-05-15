#include "bench_common.h"
#include "bench_data.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INSERT_SQL "INSERT INTO " BENCH_DB "." BENCH_TBL \
                   " (id, name, x) VALUES ($1, $2, $3);"

int main(void) {
    char workdir[4096];
    bench_fresh_workdir("s2_c_", workdir, sizeof(workdir));
    char log[4096], wal[4096], disk[4096], main_path[4096];
    snprintf(log,       sizeof(log),       "%s/log",  workdir);
    snprintf(wal,       sizeof(wal),       "%s/wal",  workdir);
    snprintf(disk,      sizeof(disk),      "%s/disk", workdir);
    snprintf(main_path, sizeof(main_path), "%s/main", workdir);

    config_t cfg = {
        .level        = 6,
        .log_path     = bench_sv(log),
        .wal_path     = bench_sv(wal),
        .disk_path    = bench_sv(disk),
        .main_path    = bench_sv(main_path),
        .wal_on       = false,
        .disk_on      = false,
        .sync_to_disk = false,
    };
    otterbrix_ptr db = otterbrix_create(cfg);
    assert(db);
    cursor_ptr cur = create_database(db, bench_sv(BENCH_DB));
    release_cursor(cur);
    cur = create_collection(db, bench_sv(BENCH_DB), bench_sv(BENCH_TBL));
    release_cursor(cur);

    string_view_t insert_sv = bench_sv(INSERT_SQL);
    size_t total = N_INSERT + N_WARMUP;
    bench_row_t *rows = malloc(total * sizeof(bench_row_t));
    bench_load_rows(rows, total);

    for (size_t i = 0; i < N_WARMUP; ++i) {
        sql_param_t p[3] = {
            { .index = 1, .kind = SQL_PARAM_INT64,  .int64_value  = rows[i].id },
            { .index = 2, .kind = SQL_PARAM_STRING,
              .string_value = bench_svn(rows[i].name, BENCH_NAME_LEN) },
            { .index = 3, .kind = SQL_PARAM_DOUBLE, .double_value = rows[i].x },
        };
        cur = execute_sql_params(db, insert_sv, p, 3);
        release_cursor(cur);
    }

    uint64_t *samples = malloc(N_INSERT * sizeof(uint64_t));
    for (size_t i = 0; i < N_INSERT; ++i) {
        bench_row_t *r = &rows[N_WARMUP + i];
        sql_param_t p[3] = {
            { .index = 1, .kind = SQL_PARAM_INT64,  .int64_value  = r->id },
            { .index = 2, .kind = SQL_PARAM_STRING,
              .string_value = bench_svn(r->name, BENCH_NAME_LEN) },
            { .index = 3, .kind = SQL_PARAM_DOUBLE, .double_value = r->x },
        };
        uint64_t t0 = bench_now_ns();
        cur = execute_sql_params(db, insert_sv, p, 3);
        release_cursor(cur);
        samples[i] = bench_now_ns() - t0;
    }

    otterbrix_destroy(db);
    bench_write_csv("s2_insert_c", samples, N_INSERT);
    free(samples);
    free(rows);
    return 0;
}
