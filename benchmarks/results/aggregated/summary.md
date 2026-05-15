# Bench summary — median-of-medians (Δ% vs C)

Stages aggregated: **1, 2, 3, 4, 5, 6, 7, 8, 9** (9 total).

Each cell shows `median-of-medians (max-spread%) [Δ% vs C]`. `max-spread%` is `(max_med - min_med) / median_of_medians × 100` across stages — a quick stability check. `Δ% vs C` is the relative difference of the cell's median-of-medians against the C baseline of the same row; the C column itself is shown as `0%`. For rows without a C baseline (e.g. s7 interactive) Δ is omitted.

| scenario | variant | n/stage | c | sys | otterbrix | seaorm | sqlx |
|---|---|---:|---:|---:|---:|---:|---:|
| s10_mutate | delete | 5000 | 416.40 µs (0.4%) [+0%] | 410.12 µs (0.4%) [-2%] | 410.00 µs (0.6%) [-2%] | 417.49 µs (1.1%) [+0%] | 421.39 µs (3.4%) [+1%] |
| s10_mutate | update | 10000 | 447.21 µs (0.9%) [+0%] | 446.52 µs (0.7%) [-0%] | 448.27 µs (0.6%) [+0%] | 450.36 µs (0.6%) [+1%] | 456.81 µs (3.4%) [+2%] |
| s1_headline | — | 25000 | 429.31 µs (0.4%) [+0%] | 429.26 µs (0.2%) [-0%] | 429.16 µs (0.4%) [-0%] | 436.38 µs (0.7%) [+2%] | 438.35 µs (0.3%) [+2%] |
| s2_insert | — | 10000 |  34.64 µs (3.2%) [+0%] |  34.82 µs (4.6%) [+1%] |  35.30 µs (1.7%) [+2%] |  40.73 µs (4.7%) [+18%] |  42.02 µs (1.8%) [+21%] |
| s3_range_scan | k1 | 125 |   5.73 ms (0.4%) [+0%] |   5.73 ms (0.1%) [-0%] |   5.73 ms (0.3%) [-0%] |   5.76 ms (0.3%) [+0%] |   5.77 ms (0.2%) [+1%] |
| s3_range_scan | k100 | 125 |   5.82 ms (0.3%) [+0%] |   5.81 ms (0.2%) [-0%] |   5.88 ms (0.3%) [+1%] |   5.88 ms (0.2%) [+1%] |   5.92 ms (0.2%) [+2%] |
| s3_range_scan | k1000 | 125 |   6.60 ms (0.2%) [+0%] |   6.60 ms (0.2%) [-0%] |   7.11 ms (0.2%) [+8%] |   6.90 ms (0.6%) [+5%] |   7.22 ms (0.3%) [+9%] |
| s3_range_scan | k10000 | 125 |  14.85 ms (0.2%) [+0%] |  14.94 ms (0.3%) [+1%] |  20.00 ms (0.4%) [+35%] |  19.80 ms (1.4%) [+33%] |  21.22 ms (1.1%) [+43%] |
| s4_round_trip | — | 10000 | 428.58 µs (0.3%) [+0%] | 428.53 µs (0.4%) [-0%] | 428.44 µs (0.5%) [-0%] | 435.21 µs (0.4%) [+2%] | 437.40 µs (0.3%) [+2%] |
| s5_bulk_insert | — | 100 |  12.30 ms (3.6%) [+0%] |  12.28 ms (1.0%) [-0%] |  12.21 ms (2.7%) [-1%] |  12.90 ms (0.9%) [+5%] |  13.14 ms (0.9%) [+7%] |
| s6_open | — | 200 | 139.38 µs (4.0%) [+0%] | 149.56 µs (4.5%) [+7%] | 148.87 µs (4.6%) [+7%] | 153.77 µs (5.6%) [+10%] | 159.42 µs (6.2%) [+14%] |
| s7_interactive | — | 1000 | — | — | — | 295.82 µs (1.9%) | 295.47 µs (0.9%) |
| s8_aggregation | max | 1000 | 992.18 µs (0.8%) [+0%] | 990.16 µs (0.9%) [-0%] | 992.35 µs (0.8%) [+0%] |   1.00 ms (1.5%) [+1%] |   1.00 ms (0.2%) [+1%] |
| s8_aggregation | sum | 1000 | 992.96 µs (0.9%) [+0%] | 990.63 µs (1.0%) [-0%] | 991.60 µs (0.8%) [-0%] |   1.00 ms (1.1%) [+1%] |   1.01 ms (0.2%) [+1%] |
| s9_join | — | 125 |  44.54 ms (0.6%) [+0%] |  44.53 ms (0.7%) [-0%] |  44.54 ms (0.6%) [+0%] |  44.75 ms (0.5%) [+0%] |  44.90 ms (0.8%) [+1%] |

## Coverage

- Cells reported: 15
- No missing layers

## Stages

- stage **1** at `20260515T123818Z` → `results/raw/stage_1_20260515T123818Z`
- stage **2** at `20260515T124457Z` → `results/raw/stage_2_20260515T124457Z`
- stage **3** at `20260515T125011Z` → `results/raw/stage_3_20260515T125011Z`
- stage **4** at `20260515T125531Z` → `results/raw/stage_4_20260515T125531Z`
- stage **5** at `20260515T130105Z` → `results/raw/stage_5_20260515T130105Z`
- stage **6** at `20260515T130628Z` → `results/raw/stage_6_20260515T130628Z`
- stage **7** at `20260515T131155Z` → `results/raw/stage_7_20260515T131155Z`
- stage **8** at `20260515T131715Z` → `results/raw/stage_8_20260515T131715Z`
- stage **9** at `20260515T132228Z` → `results/raw/stage_9_20260515T132228Z`
