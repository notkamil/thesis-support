#include "bench_common.h"
#include "bench_data.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INSERT_SQL "INSERT INTO " BENCH_DB "." BENCH_TBL \
                   " (id, name, x) VALUES ($1, $2, $3);"
#define SELECT_SQL "SELECT name FROM " BENCH_DB "." BENCH_TBL \
                   " WHERE id = $1;"

int main(void) {
    char workdir[4096];
    bench_fresh_workdir("s1_c_", workdir, sizeof(workdir));
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

    bench_row_t *rows = malloc(SEED_SMALL * sizeof(bench_row_t));
    bench_load_rows(rows, SEED_SMALL);
    for (size_t i = 0; i < SEED_SMALL; ++i) {
        sql_param_t p[3] = {
            { .index = 1, .kind = SQL_PARAM_INT64,  .int64_value  = rows[i].id },
            { .index = 2, .kind = SQL_PARAM_STRING,
              .string_value = bench_svn(rows[i].name, BENCH_NAME_LEN) },
            { .index = 3, .kind = SQL_PARAM_DOUBLE, .double_value = rows[i].x },
        };
        cur = execute_sql_params(db, bench_sv(INSERT_SQL), p, 3);
        release_cursor(cur);
    }
    free(rows);

    string_view_t select_sv = bench_sv(SELECT_SQL);
    string_view_t name_col  = bench_svn("name", 4);

    int64_t *ids = malloc((N_HEADLINE + N_WARMUP) * sizeof(int64_t));
    bench_load_ids(ids, N_HEADLINE + N_WARMUP);

    for (size_t i = 0; i < N_WARMUP; ++i) {
        sql_param_t p = { .index = 1, .kind = SQL_PARAM_INT64,
                          .int64_value = ids[i] };
        cur = execute_sql_params(db, select_sv, &p, 1);
        release_cursor(cur);
    }

    uint64_t *samples = malloc(N_HEADLINE * sizeof(uint64_t));
    for (size_t i = 0; i < N_HEADLINE; ++i) {
        sql_param_t p = { .index = 1, .kind = SQL_PARAM_INT64,
                          .int64_value = ids[N_WARMUP + i] };

        uint64_t t0 = bench_now_ns();
        cur = execute_sql_params(db, select_sv, &p, 1);
        value_ptr v = cursor_get_value_by_name(cur, 0, name_col);
        char *raw = value_get_string(v);

        char *owned = strdup(raw);
        release_value(v);
        release_cursor(cur);
        samples[i] = bench_now_ns() - t0;
        free(owned);
    }

    otterbrix_destroy(db);
    bench_write_csv("s1_headline_c", samples, N_HEADLINE);
    free(samples);
    free(ids);
    return 0;
}
