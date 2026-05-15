#ifndef OTTERBRIX_BENCH_DATA_H
#define OTTERBRIX_BENCH_DATA_H

#include <stddef.h>
#include <stdint.h>

#include "bench_common.h"

typedef struct bench_row_t {
    int64_t id;
    char    name[BENCH_NAME_LEN];
    double  x;
} bench_row_t;

const char *bench_data_path(const char *name);

void bench_load_rows(bench_row_t *out, size_t n);
void bench_load_ids(int64_t *out, size_t n);

void bench_write_csv(const char *name, const uint64_t *samples, size_t n);

void bench_fresh_workdir(const char *prefix, char *out, size_t out_size);

#endif
