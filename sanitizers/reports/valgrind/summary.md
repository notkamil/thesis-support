# Valgrind memcheck report

Прогон тестов `cargo test --workspace` под `valgrind --tool=memcheck` против обычной (без ASan) релизной `libotterbrix.so`. Каждый test-бинарник cargo'а оборачивается через `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER` в скрипт-обёртку [`valgrind-wrapper.sh`](valgrind-wrapper.sh), которая для бинарников `*large_dataset*` сразу делает `exit 0` (Valgrind на нагрузочных тестах непрактично долго), а для всех остальных запускает `valgrind --tool=memcheck --leak-check=full --show-leak-kinds=definite,indirect`.

Цель — независимо от ASan проверить отсутствие memory corruption (Invalid read/write, mismatched free), use-of-uninitialised, и реальных утечек в Rust-обёртках и в `libotterbrix.so` под нашим test suite. Дополняет [`asan_rust`](../asan_rust/) и [`asan_cpp`](../asan_cpp/) другим механизмом детекции (динамическая инструментация через JIT) — независимая проверка тех же путей.

Команда — в [`cmd.sh`](cmd.sh), обёртка — в [`valgrind-wrapper.sh`](valgrind-wrapper.sh), suppressions — [`valgrind.supp`](valgrind.supp), хост и тулчейн — в [`meta.json`](meta.json), полный лог — [`full.log`](full.log), пер-бинарник прогресс — [`progress.log`](progress.log).

## Итог

- **Тесты**: 231 passed, 0 failed, 0 ignored. 12 large_dataset-тестов в 3 бинарниках намеренно пропущены (в [`../../../tests/reports/correctness/`](../../../tests/reports/correctness/) их 244 — разница ровно совпадает).
- **Valgrind**: 44 бинарника завершились с `ERROR SUMMARY: 0 errors`, 0 бинарников с ненулевыми errors.
- **Leaks**: `definitely lost`, `indirectly lost`, `possibly lost` — 0 байт во всех бинарниках.
- **Suppressed**: сработал baseline для Rust std (TLS у `std::sync::mpmc::context::Context::new` и глобальная `BTreeMap` для `stack_overflow::thread_info`), [`valgrind.supp`](valgrind.supp).
- **Still reachable**: 3 бинарника показали ненулевой `still reachable` — это блоки, на которые есть живой указатель в момент `_exit` (глобальные статики), Valgrind их не классифицирует как leaks.
- Никаких `Invalid read`, `Invalid write`, `Conditional jump`, `Use of uninitialised value`, `Mismatched free`.

Wallclock — 323 секунды.

## Скрипт прогресса

Если ещё раз понадобится перезапустить и наблюдать ход — есть [`check-progress.sh`](check-progress.sh), показывает elapsed, сколько бинарников запущено / ok / с errors / пропущено, и кто сейчас в работе.
