#include "bench_common.h"
#include "bench_data.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *build_bulk_sql(size_t *out_len) {
    bench_row_t *rows = malloc(N_BULK * sizeof(bench_row_t));
    bench_load_rows(rows, N_BULK);

    size_t cap = 64 + (size_t)N_BULK * 80;
    char *buf = malloc(cap);
    if (!buf) { fprintf(stderr, "oom\n"); exit(1); }
    int n = snprintf(buf, cap, "INSERT INTO %s.%s (id, name, x) VALUES ",
                     BENCH_DB, BENCH_TBL);
    if (n < 0) { fprintf(stderr, "snprintf\n"); exit(1); }
    size_t pos = (size_t)n;
    for (size_t i = 0; i < N_BULK; ++i) {
        char name_buf[BENCH_NAME_LEN + 1];
        memcpy(name_buf, rows[i].name, BENCH_NAME_LEN);
        name_buf[BENCH_NAME_LEN] = '\0';
        int w = snprintf(buf + pos, cap - pos, "%s(%lld, '%s', %.17g)",
                         (i == 0 ? "" : ", "),
                         (long long)rows[i].id, name_buf, rows[i].x);
        if (w < 0 || (size_t)w >= cap - pos) {
            fprintf(stderr, "bulk SQL buffer too small\n"); exit(1);
        }
        pos += (size_t)w;
    }
    if (pos + 1 >= cap) { fprintf(stderr, "bulk SQL buffer too small\n"); exit(1); }
    buf[pos++] = ';';
    buf[pos] = '\0';

    free(rows);
    *out_len = pos;
    return buf;
}

static uint64_t one_run(string_view_t bulk_sv, const char *prefix, size_t i) {
    char full_prefix[64];
    snprintf(full_prefix, sizeof(full_prefix), "%s%zu_", prefix, i);
    char workdir[4096], log[4096], wal[4096], disk[4096], main_path[4096];
    bench_fresh_workdir(full_prefix, workdir, sizeof(workdir));
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

    uint64_t t0 = bench_now_ns();
    cur = execute_sql(db, bulk_sv);
    release_cursor(cur);
    uint64_t elapsed = bench_now_ns() - t0;

    otterbrix_destroy(db);
    return elapsed;
}

int main(void) {
    size_t bulk_len = 0;
    char *bulk_sql = build_bulk_sql(&bulk_len);
    string_view_t bulk_sv = bench_svn(bulk_sql, bulk_len);

    for (size_t i = 0; i < N_BULK_WARMUP; ++i) {
        (void)one_run(bulk_sv, "s5_c_warm_", i);
    }

    uint64_t *samples = malloc(N_BULK_RUNS * sizeof(uint64_t));
    for (size_t i = 0; i < N_BULK_RUNS; ++i) {
        samples[i] = one_run(bulk_sv, "s5_c_", i);
    }

    bench_write_csv("s5_bulk_insert_c", samples, N_BULK_RUNS);
    free(samples);
    free(bulk_sql);
    return 0;
}
