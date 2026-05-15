# ASan-Rust report

Прогон `cargo +nightly test --workspace` с `RUSTFLAGS="-Z sanitizer=address"` с релизной `libotterbrix.so`. Цель — поймать UAF / OOB / use-after-scope / double-free / UB в Rust-коде FFI-обёрток.

Команда — в [`cmd.sh`](cmd.sh), хост и тулчейн — в [`meta.json`](meta.json), полный лог — [`asan.log`](asan.log).

## Итог

- **Тесты**: 244 passed, 0 failed, 0 ignored — те же что и в обычном `correctness/`-прогоне (см. [`../../tests/reports/correctness/`](../../../tests/reports/correctness/)).
- **ASan-репортов**: 0 (`AddressSanitizer:`, `==…== ERROR:`, `SUMMARY:` не встретились ни разу).
