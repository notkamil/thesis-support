# Сценарии бенчмарков

Десять сценариев `S1`…`S10`, покрывающих типичные точки нагрузки на движок: точечный поиск, write-сценарии, диапазонные выборки, агрегации, JOIN, стоимость открытия БД, интерактивный смешанный workflow и mutating-операции (UPDATE/DELETE). Числа итераций и размеры выборок зафиксированы в [`code/rust/src/scenarios.rs`](code/rust/src/scenarios.rs) и продублированы в [`code/c/include/bench_common.h`](code/c/include/bench_common.h).

## Общие соглашения

**Слои.** Каждый сценарий гоняется на пяти слоях, выходящих на движок через одни и те же `extern "C"` точки `libotterbrix.so`:

- `c` — pure-C бенч через C-ABI (baseline);
- `sys` — Rust raw FFI (`otterbrix-sys`);
- `otterbrix` — safe Rust-обёртка (`otterbrix`);
- `seaorm` — `seaorm-otterbrix` поверх `otterbrix`;
- `sqlx` — `sqlx-otterbrix` поверх `otterbrix`.

Исключение — `S7`: гоняется только на `seaorm` и `sqlx` (у `c`/`sys`/`otterbrix` нет понятия пользовательского соединения).

**Конфигурация движка.** Везде один и тот же `Config`:

- `level = 6` — `spdlog` отключён (иначе движок генерирует тысячи trace-строк на каждый statement и доминирует над любым измерением);
- `wal_on = false`, `disk_on = false`, `sync_to_disk = false` — все durability-подсистемы выключены, чтобы не подмешивать шум диска;
- `log_path` / `wal_path` / `disk_path` / `main_path` — пути в свежем tempdir (один на запуск бенча).

**Флаги сборки.** Преднамеренно симметричны для Rust и C, чтобы «выиграл компилятор, а не язык» не было объяснением разницы.

- Rust ([`benchmarks/Cargo.toml`](Cargo.toml) + [`benchmarks/.cargo/config.toml`](.cargo/config.toml)): `opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks = false`, `-C target-cpu=x86-64-v3`.
- C ([`code/c/build.sh`](code/c/build.sh)): `clang -std=c11 -O3 -flto=thin -march=x86-64-v3 -DNDEBUG -fno-omit-frame-pointer`.

**Схема.** Все сценарии работают с таблицей `bench.t(id BIGINT, name STRING, x DOUBLE)`; имена в `name` — фиксированные 16-байтные алфавитно-цифровые строки. `S9` дополнительно заводит `bench.u` той же схемы. Данные генерируются детерминированным потоком `rand_chacha::ChaCha8Rng` с фиксированным seed'ом — Rust-бенчи обращаются к нему напрямую, C-бенчи читают тот же байтовый поток из заранее дампнутого `data/rows_max.bin` / `data/lookup_ids_max.bin`.

**Warm-up.** В сценариях с длинным однородным timed loop перед записью первой точки выполняется `N_WARMUP = 1 000` неизмеряемых итераций той же формы, чтобы аллокатор, branch predictors и CPU-кеши вышли в steady state. У сценариев с тяжёлыми timed-итерациями warm-up отдельный и меньший, чтобы прогрев не доминировал над замером: `N_RANGE_WARMUP = 250` (S3, на каждый `k`), `N_JOIN_WARMUP = 250` (S9), `N_BULK_WARMUP = 10` (S5), `N_OPEN_WARMUP = 20` (S6).

**Формат сырого вывода.** Каждый бенч-бинарник пишет per-iteration latency в нс в CSV с единственной колонкой `ns` (одно число на строку). Один CSV на пару `(scenario, layer)`; для сценариев с вариантами (`S3` по `k`, `S8` по `sum`/`max`) — отдельный CSV на каждый вариант.

## S1. Headline — point lookup (steady-state)

Pre-seed: 1 000 строк. Warm-up 1 000 + 25 000 параметризованных запросов `SELECT name FROM bench.t WHERE id = $1` со случайным `id` из заранее сгенерированного массива. В timed region — `execute_sql_params` + `cursor_get_value_by_name` + материализация одной строковой ячейки в owned heap-буфер + `release_value` + `release_cursor`. Размер выборки 25 000 даёт узкий доверительный интервал медианы и одновременно служит «знаменем» — это та цифра, которой обычно описывают «обычный latency point lookup'а».

## S2. Insert — per-row INSERT

Одна открытая база. Warm-up 1 000 + 10 000 параметризованных `INSERT INTO bench.t (id, name, x) VALUES ($1, $2, $3)` один-за-другим. Текст SQL не меняется между итерациями — переаллокация связана только с биндингом параметров. Time region — каждый `execute_sql_params` + `release_cursor`.

## S3. Range scan — projection of three columns

Pre-seed: 10 000 строк. Для каждого `k ∈ {1, 100, 1 000, 10 000}`: warm-up 250 + 125 итераций `SELECT id, name, x FROM bench.t WHERE id BETWEEN 1 AND $1`. В timed region — выполнение запроса + полная итерация по всем `k` строкам результата с извлечением каждой ячейки (`id` → int, `name` → owned `String`, `x` → double). Один CSV на каждый `k`. Меньшие warm-up и timed (по сравнению с однородными s1/s4) — потому что каждая итерация при `k = 10 000` материализует 10 000 строк и сама по себе тяжёлая.

## S4. Round trip — single-cell read

Pre-seed: 1 000 строк. Warm-up 1 000 + 10 000 параметризованных запросов `SELECT id FROM bench.t WHERE id = $1`. Проекция — только `id`, чтобы исключить из timed region аллокацию строки и изолировать чистую стоимость одного round-trip-а через FFI: `execute_sql_params` + `cursor_get_value` + `value_get_int` + `release_value` + `release_cursor`.

## S5. Bulk insert — single VALUES statement

Warm-up 10 + 100 запусков. На каждом: открывается свежая база; формируется (один раз, до timed region) текст `INSERT INTO bench.t (id, name, x) VALUES (...), (...), ...` с 10 000 row-литералами; выполняется один `execute_sql` на этот единственный statement. Time region — вызов `execute_sql` + `release_cursor`.

## S6. Database open

Warm-up 20 + 200 независимых пар `otterbrix_create` + `otterbrix_destroy` в свежих tempdir'ах. Time region — только сам `otterbrix_create`. SQL не выполняется. Изолирует стоимость инициализации движка: actor-zeta scheduler, memory-pool resource, manager_dispatcher, pmr-аллокаторы.

## S7. Interactive — INSERT + SELECT pair (sqlx / seaorm only)

Pre-seed: 1 000 строк. Warm-up 1 000 + 1 000 итераций. Каждая итерация — параметризованный INSERT новой строки и параметризованный SELECT-by-id той же строки; пара меряется как одна выборка CSV. Имитирует мелкий «запрос-ответ» рабочий процесс приложения через адаптер. У `c` / `sys` / `otterbrix` нет понятия пользовательского соединения, поэтому сценарий определён только для `seaorm` и `sqlx`.

## S8. Aggregation — `SUM(x)` and `MAX(x)`

Pre-seed: 10 000 строк. Два набора по: warm-up 1 000 + 1 000 итераций — `SELECT SUM(x) FROM bench.t` и `SELECT MAX(x) FROM bench.t`. Каждое значение — один double, материализуется в timed region. Раздельные CSV: `..._sum`, `..._max`.

## S9. Inner JOIN

Pre-seed: 500 строк в `bench.t` и 500 идентичных строк в `bench.u` (так что каждая строка `bench.t` стыкуется ровно с одной строкой `bench.u`). Warm-up 250 + 125 итераций `SELECT t.id, t.name, u.x FROM bench.t AS t INNER JOIN bench.u AS u ON t.id = u.id`. В timed region — выполнение JOIN'а + полная материализация всех 500 строк результата. Меньшие warm-up и timed (по сравнению с однородными s1/s4) — потому что каждая итерация JOIN'а тяжёлая (планировщик + 500 материализованных строк на одну точку).

## S10. Mutate — UPDATE / DELETE

Бинарник один (`s10_mutate_<layer>`), но запускается с обязательным аргументом `update` либо `delete`. Каждая фаза — в собственном процессе со свежей БД, что даёт полную изоляцию состояния между двумя разнородными write-операциями. `run_stage.sh` для `s10` сам запускает обе фазы подряд.

**Update-фаза.** Pre-seed 1 000 строк. Warm-up 1 000 + 10 000 итераций параметризованного `UPDATE bench.t SET x = $1 WHERE id = $2`. `id` подбирается последовательно как `(i mod 1 000) + 1`: цикл по таблице, каждая строка многократно перезаписывается. Это типичный counter-update / cache-update паттерн — нагружает write-path, не паттерн доступа.

**Delete-фаза.** Pre-seed `SEED_SMALL + N_WARMUP + N_DELETE = 7 000` строк. Warm-up 1 000 DELETE-ов на id из `[1 001, 2 000]` + 5 000 timed DELETE-ов на id из `[2 001, 7 000]` (sequential, без повторов — каждый DELETE удаляет ровно одну существующую строку).

CSV: `s10_mutate_<layer>_update.csv` и `s10_mutate_<layer>_delete.csv` — два файла на слой.
