# tests

`cargo test` и `cargo llvm-cov` для четырёх крейтов FFI-обёрток (`otterbrix-sys`, `otterbrix`, `seaorm-otterbrix`, `sqlx-otterbrix`). Support-материал к ВКР: артефакты прогонов на конкретном SHA otterbrix-а, не воспроизводится из коробки.

```
reports/
├── correctness/        # cargo test --workspace --no-fail-fast
│   ├── cargo_test.log  # отфильтрованный вывод libtest (без spdlog-шума)
│   ├── cmd.sh          # точная команда прогона
│   └── meta.json       # SHA, host, kernel, rustc, тулчейн
└── coverage/           # cargo llvm-cov --workspace + per-package report --html
    ├── summary.md      # workspace + по крейтам + по файлам, ссылки на html
    ├── cmd.sh
    ├── meta.json
    └── html/
        ├── workspace/             # общий отчёт
        ├── otterbrix/
        ├── otterbrix-sys/         # пуст: крейт это include!() bindgen
        ├── seaorm-otterbrix/
        └── sqlx-otterbrix/
```
