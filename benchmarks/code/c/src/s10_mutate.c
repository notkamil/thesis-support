#include "bench_common.h"
#include "bench_data.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INSERT_SQL "INSERT INTO " BENCH_DB "." BENCH_TBL \
                   " (id, name, x) VALUES ($1, $2, $3);"
#define UPDATE_SQL "UPDATE " BENCH_DB "." BENCH_TBL \
                   " SET x = $1 WHERE id = $2;"
#define DELETE_SQL "DELETE FROM " BENCH_DB "." BENCH_TBL \
                   " WHERE id = $1;"

static void seed(otterbrix_ptr db, size_t n) {
    bench_row_t *rows = malloc(n * sizeof(bench_row_t));
    bench_load_rows(rows, n);
    string_view_t insert_sv = bench_sv(INSERT_SQL);
    for (size_t i = 0; i < n; ++i) {
        sql_param_t p[3] = {
            { .index = 1, .kind = SQL_PARAM_INT64,  .int64_value  = rows[i].id },
            { .index = 2, .kind = SQL_PARAM_STRING,
              .string_value = bench_svn(rows[i].name, BENCH_NAME_LEN) },
            { .index = 3, .kind = SQL_PARAM_DOUBLE, .double_value = rows[i].x },
        };
        cursor_ptr cur = execute_sql_params(db, insert_sv, p, 3);
        release_cursor(cur);
    }
    free(rows);
}

static void run_update(otterbrix_ptr db) {
    string_view_t sv = bench_sv(UPDATE_SQL);

    for (size_t i = 0; i < N_WARMUP; ++i) {
        int64_t id = (int64_t)(i % SEED_SMALL) + 1;
        sql_param_t p[2] = {
            { .index = 1, .kind = SQL_PARAM_DOUBLE,
              .double_value = (double)i * 0.5 },
            { .index = 2, .kind = SQL_PARAM_INT64,
              .int64_value = id },
        };
        cursor_ptr cur = execute_sql_params(db, sv, p, 2);
        release_cursor(cur);
    }

    uint64_t *samples = malloc(N_UPDATE * sizeof(uint64_t));
    for (size_t i = 0; i < N_UPDATE; ++i) {
        int64_t id = (int64_t)((i + N_WARMUP) % SEED_SMALL) + 1;
        sql_param_t p[2] = {
            { .index = 1, .kind = SQL_PARAM_DOUBLE,
              .double_value = (double)i * 0.5 },
            { .index = 2, .kind = SQL_PARAM_INT64,
              .int64_value = id },
        };
        uint64_t t0 = bench_now_ns();
        cursor_ptr cur = execute_sql_params(db, sv, p, 2);
        release_cursor(cur);
        samples[i] = bench_now_ns() - t0;
    }
    bench_write_csv("s10_mutate_c_update", samples, N_UPDATE);
    free(samples);
}

static void run_delete(otterbrix_ptr db) {
    string_view_t sv = bench_sv(DELETE_SQL);

    for (size_t i = 0; i < N_WARMUP; ++i) {
        int64_t id = (int64_t)(SEED_SMALL + i + 1);
        sql_param_t p = { .index = 1, .kind = SQL_PARAM_INT64, .int64_value = id };
        cursor_ptr cur = execute_sql_params(db, sv, &p, 1);
        release_cursor(cur);
    }

    uint64_t *samples = malloc(N_DELETE * sizeof(uint64_t));
    for (size_t i = 0; i < N_DELETE; ++i) {
        int64_t id = (int64_t)(SEED_SMALL + N_WARMUP + i + 1);
        sql_param_t p = { .index = 1, .kind = SQL_PARAM_INT64, .int64_value = id };
        uint64_t t0 = bench_now_ns();
        cursor_ptr cur = execute_sql_params(db, sv, &p, 1);
        release_cursor(cur);
        samples[i] = bench_now_ns() - t0;
    }
    bench_write_csv("s10_mutate_c_delete", samples, N_DELETE);
    free(samples);
}

int main(int argc, char **argv) {
    if (argc != 2 ||
        (strcmp(argv[1], "update") != 0 && strcmp(argv[1], "delete") != 0)) {
        fprintf(stderr, "usage: %s <update|delete>\n",
                argc > 0 ? argv[0] : "s10_mutate");
        return 2;
    }
    const char *mode = argv[1];

    char workdir[4096], log[4096], wal[4096], disk[4096], main_path[4096];
    char prefix[64];
    snprintf(prefix, sizeof(prefix), "s10_c_%s_", mode);
    bench_fresh_workdir(prefix, workdir, sizeof(workdir));
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

    if (strcmp(mode, "update") == 0) {
        seed(db, SEED_SMALL);
        run_update(db);
    } else {
        seed(db, SEED_MUTATE);
        run_delete(db);
    }

    otterbrix_destroy(db);
    return 0;
}
