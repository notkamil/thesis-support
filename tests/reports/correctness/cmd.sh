#!/usr/bin/env bash
cd /home/notkamil/ITMO/S8/diploma/my/otterbrix/integration/rust && \
    cargo test --workspace --no-fail-fast --color never 2>&1 \
    | grep -E '^(running |test |test result:|failures:|---- |    Running |   Doc-tests |    Finished|   Compiling)' \
    > /home/notkamil/ITMO/S8/diploma/thesis-support/tests/reports/20260515T065911Z/cargo_test.log
