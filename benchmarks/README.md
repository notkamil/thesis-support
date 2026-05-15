# otterbrix benchmarks

Замер overhead-а Rust-FFI обёрток над движком [otterbrix](https://github.com/duckstax/otterbrix). Десять сценариев (point lookup, range scan, INSERT, UPDATE/DELETE, JOIN, агрегация, открытие БД и т.д.) на пяти слоях: pure-C baseline, raw FFI (`otterbrix-sys`), safe-Rust обёртка (`otterbrix`) и адаптеры для `seaorm`/`sqlx`. Все слои выходят на движок через одни и те же `extern "C"` точки `libotterbrix.so`, поэтому разница между ними — это ровно то, что мы и хотим измерить.

- [scenarios.md](scenarios.md) — что меряем
- [methodology.md](methodology.md) — как меряем (статистика, контроль шума, протокол прогона)

## Quickstart на чистом сервере

```bash
# 1. Системные пакеты + conan + rustup (Debian/Ubuntu или Fedora)
./scripts/setup-server.sh

# 2. (опционально, требует root) Тихий профиль CPU: turbo off,
#    governor=performance, SMT off, swap off
sudo ./scripts/tune-host.sh

# 3. Полная сборка: clone otterbrix на пиннутом SHA -> libotterbrix.so
#    -> cargo build (rust бенчи) -> clang (c бенчи) -> data/*.bin
./scripts/bootstrap.sh

# 4. Один прогон всех 10 сценариев на 5 слоях
#    (на дедике пинуй на физические ядра: taskset -c 2,3 ./scripts/run_stage.sh)
./scripts/run_stage.sh

# 5. Сводка по всем стейджам в results/aggregated/{summary.md, aggregated.json}
python3 scripts/build_quick_summary.py
```

Для воспроизводимости: SHA otterbrix зашит в [benchmarks.config](benchmarks.config), `bootstrap.sh` пишет manifest для `.workdir/dist/<sha>/`, а `run_stage.sh` — `label.json` для каждого стейджа (host, kernel, CPU, RAM, toolchain, флаги, env).
