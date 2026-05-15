#!/usr/bin/env bash
DIR=/home/notkamil/ITMO/S8/diploma/thesis-support/sanitizers/reports/valgrind

: > "$DIR/progress.log"
echo "[run-start] $(date '+%Y-%m-%d %H:%M:%S')" >> "$DIR/progress.log"

cd /home/notkamil/ITMO/S8/diploma/my/otterbrix/integration/rust
unset RUSTFLAGS RUSTDOCFLAGS ASAN_OPTIONS LD_PRELOAD

OTTERBRIX_LIB_DIR=/home/notkamil/ITMO/S8/diploma/my/otterbrix/build/integration/c \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="$DIR/valgrind-wrapper.sh" \
cargo test --workspace --target x86_64-unknown-linux-gnu \
    --no-fail-fast --color never 2>&1 \
    | grep -E '^(running |test |test result:|failures:|---- |==[0-9]+==|SUMMARY:|ERROR:|WARNING:|    Running |   Doc-tests |    Finished|   Compiling|\[skip|\[start|\[done)' \
    > "$DIR/full.log"

echo "[run-end] $(date '+%Y-%m-%d %H:%M:%S')" >> "$DIR/progress.log"
