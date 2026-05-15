#ifndef OTTERBRIX_BENCH_COMMON_H
#define OTTERBRIX_BENCH_COMMON_H

#include "otterbrix_capi.h"
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <time.h>

#define BENCH_DB    "bench"
#define BENCH_TBL   "t"
#define BENCH_TBL2  "u"

#define BENCH_NAME_LEN 16

#define N_WARMUP        1000

#define SEED_SMALL      1000
#define SEED_LARGE      10000
#define SEED_JOIN       500

#define N_HEADLINE      25000
#define N_INSERT        10000
#define N_RANGE_RUNS    125
#define N_RANGE_WARMUP  250
#define N_ROUND_TRIP    10000
#define N_BULK          10000
#define N_BULK_RUNS     100
#define N_BULK_WARMUP   10
#define N_OPEN          200
#define N_OPEN_WARMUP   20
#define N_AGG_RUNS      1000
#define N_JOIN_RUNS     125
#define N_JOIN_WARMUP   250

#define N_UPDATE        10000
#define N_DELETE        5000
#define SEED_MUTATE     (SEED_SMALL + N_DELETE + N_WARMUP)

extern const int64_t N_RANGE_K[];
extern const size_t  N_RANGE_K_LEN;

static inline string_view_t bench_sv(const char *s) {
    string_view_t v;
    v.data = s;
    v.size = strlen(s);
    return v;
}

static inline string_view_t bench_svn(const char *s, size_t n) {
    string_view_t v;
    v.data = s;
    v.size = n;
    return v;
}

static inline uint64_t bench_now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

#endif
