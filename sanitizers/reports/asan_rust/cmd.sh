#!/usr/bin/env bash
DIR=/home/notkamil/ITMO/S8/diploma/thesis-support/sanitizers/reports/asan_rust
cd /home/notkamil/ITMO/S8/diploma/my/otterbrix/integration/rust
RUSTFLAGS="-Z sanitizer=address" \
RUSTDOCFLAGS="-Z sanitizer=address" \
ASAN_OPTIONS="detect_leaks=0:abort_on_error=0:halt_on_error=0" \
cargo +nightly test --workspace --target x86_64-unknown-linux-gnu \
    --no-fail-fast --color never 2>&1 \
    | grep -E '^(running |test |test result:|failures:|---- |==[0-9]+==|SUMMARY:|AddressSanitizer|ERROR:|WARNING:)' \
    > "$DIR/asan.log"
