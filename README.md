# thesis-support

Support-материалы к ВКР про Rust-FFI обёртки над [otterbrix](https://github.com/duckstax/otterbrix): полный воспроизводимый стенд бенчмарков (10 сценариев на 5 слоях), отчёты санитайзеров и тестовое покрытие. `benchmarks/` запускается из коробки скриптами; `sanitizers/` и `tests/` — артефакты прогонов на конкретном SHA otterbrix-а, без запуска из коробки.

## Что смотреть в первую очередь

- **Сводная таблица бенчмарков** (медиана-медиан, разброс, Δ% против C-baseline): [`benchmarks/results/aggregated/summary.md`](benchmarks/results/aggregated/summary.md)
- **Полный машиночитаемый дамп** всех замеров (для построения графиков): [`benchmarks/results/aggregated/aggregated.json`](benchmarks/results/aggregated/aggregated.json)
- **Покрытие тестами** (workspace + по крейтам): [`tests/reports/coverage/summary.md`](tests/reports/coverage/summary.md)
- **Корректность тестов** (244/244 passed): [`tests/reports/correctness/cargo_test.log`](tests/reports/correctness/cargo_test.log)
- **Санитайзеры** (три независимых прогона, везде 0 errors / 0 leaks):
  - LLVM ASan на Rust: [`sanitizers/reports/asan_rust/summary.md`](sanitizers/reports/asan_rust/summary.md)
  - GCC ASan на C++ части: [`sanitizers/reports/asan_cpp/summary.md`](sanitizers/reports/asan_cpp/summary.md)
  - Valgrind memcheck: [`sanitizers/reports/valgrind/summary.md`](sanitizers/reports/valgrind/summary.md)

## Структура

```
benchmarks/   стенд бенчмарков (скрипты + результаты)
  scripts/    setup-server.sh, tune-host.sh, bootstrap.sh, run_stage.sh, build_quick_summary.py
  code/       исходники C и Rust бенчмарков
  results/    raw CSV по стейджам + aggregated/{summary.md, aggregated.json}
  README.md
sanitizers/   отчёты ASan x2 + Valgrind, по cmd.sh / asan.log / meta.json / summary.md в каждом
tests/        cargo test (correctness/) + cargo llvm-cov (coverage/) с per-crate HTML
```

## Воспроизводимость

- `benchmarks/` — собирается и запускается из коробки скриптами в `benchmarks/scripts/`; SHA коммита otterbrix-а зафиксирован в `benchmarks/benchmarks.config`.
- `sanitizers/` и `tests/` — рядом с каждым `summary.md` лежит `cmd.sh` с точной командой прогона, но запуск требует подтянутых вручную путей до otterbrix.
