# sanitizers

ASan и Valgrind memcheck для четырёх крейтов FFI-обёрток (`otterbrix-sys`, `otterbrix`, `seaorm-otterbrix`, `sqlx-otterbrix`) и нативной `libotterbrix.so`. Совместить в одном процессе LLVM-ASan (на Rust) и GCC-ASan (на C++) нельзя — runtime'ы несовместимы — поэтому ASan дан двумя независимыми прогонами, плюс отдельно Valgrind как независимая проверка через динамическую инструментацию. Support-материал к ВКР: артефакты прогонов на конкретном SHA otterbrix-а, не воспроизводится из коробки.

```
reports/
├── asan_rust/          # LLVM ASan, инструментирован Rust, обычная libotterbrix.so
│   ├── asan.log        # отфильтрованный лог cargo test
│   ├── cmd.sh
│   ├── meta.json
│   └── summary.md      # 244/244 ok, 0 ASan reports
├── asan_cpp/           # GCC ASan, инструментирована libotterbrix.so, обычный Rust
│   ├── asan.log
│   ├── cmd.sh          # cargo test без -Z sanitizer, LD_PRELOAD libasan через RUNNER
│   ├── meta.json
│   └── summary.md      # 244/244 ok, 0 ASan/LSan reports
└── valgrind/           # valgrind memcheck над обычной .so и обычными бинарниками
    ├── full.log        # отфильтрованный лог тестов и valgrind
    ├── progress.log    # таймлайн [start]/[done] по бинарникам
    ├── cmd.sh
    ├── valgrind-wrapper.sh  # CARGO RUNNER: пропускает large_dataset, остальное → valgrind
    ├── valgrind.supp        # suppressions для Rust std baseline (TLS, stack-overflow info)
    ├── check-progress.sh    # удобный скрипт-наблюдатель прогресса
    ├── meta.json
    └── summary.md      # 231/231 ok (12 large_dataset пропущено), 0 errors, 0 leaks
```
