#  ASan-C++ report

Прогон тестов `cargo test --workspace` (без `-Z sanitizer=address` на стороне Rust) против ASan-инструментированной `libotterbrix.so` (GCC `-fsanitize=address`, сборка в `build-asan/`). GCC ASan-runtime подгружается в тестовые бинарники через `LD_PRELOAD=/usr/lib64/libasan.so.8` с помощью `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER` (на сам `cargo`/`rustc` ASan не накатывается). Цель — поймать UAF / OOB / double-free / use-after-scope / leaks внутри C++-кода `libotterbrix.so` и на FFI-границе при реальных вызовах из тестов.

Этот прогон дополняет [`asan_rust`](../asan_rust/): там Rust-сторона была инструментирована LLVM-ASan против обычной `libotterbrix.so`, тут — наоборот, инструментирована C++-сторона. Совмещать LLVM-ASan и GCC-ASan в одном процессе нельзя (несовместимые runtime'ы), поэтому покрытие даётся двумя отдельными прогонами.

Команда — в [`cmd.sh`](cmd.sh), хост и тулчейн — в [`meta.json`](meta.json), полный лог — [`asan.log`](asan.log).

## Итог

- **Тесты**: 244 passed, 0 failed, 0 ignored — те же что и в обычном `correctness/`-прогоне (см. [`../../../tests/reports/correctness/`](../../../tests/reports/correctness/)).
- **ASan-репортов**: 0 (`AddressSanitizer:`, `==…== ERROR:`, `SUMMARY:` не встретились ни разу).
- **LSan-репортов**: 0 (`detect_leaks=1` был включён).
