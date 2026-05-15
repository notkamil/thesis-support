#include "bench_common.h"
#include "bench_data.h"

#include <stdio.h>
#include <stdlib.h>

static uint64_t one_run(const char *prefix, size_t i) {
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

    uint64_t t0 = bench_now_ns();
    otterbrix_ptr db = otterbrix_create(cfg);
    uint64_t elapsed = bench_now_ns() - t0;
    otterbrix_destroy(db);
    return elapsed;
}

int main(void) {
    for (size_t i = 0; i < N_OPEN_WARMUP; ++i) {
        (void)one_run("s6_c_warm_", i);
    }

    uint64_t *samples = malloc(N_OPEN * sizeof(uint64_t));
    for (size_t i = 0; i < N_OPEN; ++i) {
        samples[i] = one_run("s6_c_", i);
    }

    bench_write_csv("s6_open_c", samples, N_OPEN);
    free(samples);
    return 0;
}
