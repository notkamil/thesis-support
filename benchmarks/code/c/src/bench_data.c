#include "bench_data.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

const int64_t N_RANGE_K[]   = {1, 100, 1000, 10000};
const size_t  N_RANGE_K_LEN = sizeof(N_RANGE_K) / sizeof(N_RANGE_K[0]);

static void die(const char *fmt, const char *arg) {
    fprintf(stderr, "bench: ");
    fprintf(stderr, fmt, arg);
    fprintf(stderr, ": %s\n", strerror(errno));
    exit(1);
}

const char *bench_data_path(const char *name) {
    static char buf[4096];
    const char *base = getenv("BENCH_DATA_DIR");
    if (!base || !*base) {
        fprintf(stderr, "bench: BENCH_DATA_DIR not set; "
                        "expected <benchmarks>/data\n");
        exit(1);
    }
    int n = snprintf(buf, sizeof(buf), "%s/%s", base, name);
    if (n < 0 || (size_t)n >= sizeof(buf)) {
        fprintf(stderr, "bench: data path too long\n");
        exit(1);
    }
    return buf;
}

static void load_exact(const char *path, void *out, size_t bytes) {
    FILE *f = fopen(path, "rb");
    if (!f) die("open %s", path);
    size_t got = fread(out, 1, bytes, f);
    if (got != bytes) {
        fprintf(stderr, "bench: short read from %s: got %zu of %zu\n",
                path, got, bytes);
        exit(1);
    }
    fclose(f);
}

void bench_load_rows(bench_row_t *out, size_t n) {

    _Static_assert(sizeof(bench_row_t) == 32,
                   "bench_row_t must be 32 bytes for direct file read");
    const char *path = bench_data_path("rows_max.bin");
    load_exact(path, out, n * sizeof(bench_row_t));
}

void bench_load_ids(int64_t *out, size_t n) {
    const char *path = bench_data_path("lookup_ids_max.bin");
    load_exact(path, out, n * sizeof(int64_t));
}

void bench_write_csv(const char *name, const uint64_t *samples, size_t n) {
    const char *base = getenv("BENCH_RESULTS_DIR");
    if (!base || !*base) {
        fprintf(stderr, "bench: BENCH_RESULTS_DIR not set\n");
        exit(1);
    }
    char path[4096];
    int got = snprintf(path, sizeof(path), "%s/%s.csv", base, name);
    if (got < 0 || (size_t)got >= sizeof(path)) {
        fprintf(stderr, "bench: results path too long\n");
        exit(1);
    }

    FILE *f = fopen(path, "w");
    if (!f) die("open %s for writing", path);
    fputs("ns\n", f);
    for (size_t i = 0; i < n; ++i) {
        fprintf(f, "%llu\n", (unsigned long long)samples[i]);
    }
    fclose(f);
    fprintf(stderr, "[bench] %zu samples -> %s\n", n, path);
}

void bench_fresh_workdir(const char *prefix, char *out, size_t out_size) {
    const char *tmp = getenv("TMPDIR");
    if (!tmp || !*tmp) tmp = "/tmp";
    int got = snprintf(out, out_size, "%s/%sXXXXXX", tmp, prefix);
    if (got < 0 || (size_t)got >= out_size) {
        fprintf(stderr, "bench: workdir path too long\n");
        exit(1);
    }
    if (!mkdtemp(out)) die("mkdtemp %s", out);
}
