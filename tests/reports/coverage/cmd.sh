#!/usr/bin/env bash
DIR=/home/notkamil/ITMO/S8/diploma/thesis-support/tests/reports/coverage
cd /home/notkamil/ITMO/S8/diploma/my/otterbrix/integration/rust
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --no-fail-fast --no-report
cargo llvm-cov report --html --output-dir "$DIR/html/workspace"
for pkg in otterbrix otterbrix-sys seaorm-otterbrix sqlx-otterbrix; do
    cargo llvm-cov report --html --package "$pkg" --output-dir "$DIR/html/$pkg"
done
