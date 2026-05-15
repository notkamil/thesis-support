#!/usr/bin/env bash
DIR=/home/notkamil/ITMO/S8/diploma/thesis-support/sanitizers/reports/asan_cpp

ninja -C /home/notkamil/ITMO/S8/diploma/my/otterbrix/build-asan c_otterbrix

cd /home/notkamil/ITMO/S8/diploma/my/otterbrix/integration/rust
unset RUSTFLAGS RUSTDOCFLAGS ASAN_OPTIONS LD_PRELOAD
OTTERBRIX_LIB_DIR=/home/notkamil/ITMO/S8/diploma/my/otterbrix/build-asan/integration/c \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="env LD_PRELOAD=/usr/lib64/libasan.so.8 ASAN_OPTIONS=detect_leaks=1:abort_on_error=0:halt_on_error=0:symbolize=1:strict_string_checks=1:fast_unwind_on_malloc=0" \
cargo test --workspace --target x86_64-unknown-linux-gnu \
    --no-fail-fast --color never 2>&1 \
    | grep -E '^(running |test |test result:|failures:|---- |==[0-9]+==|SUMMARY:|AddressSanitizer|LeakSanitizer|ERROR:|WARNING:|    Running |   Doc-tests |    Finished|   Compiling)' \
    > "$DIR/asan.log"
