#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_ROOT="$(cd "$HERE/../.." && pwd)"
DIST="$BENCH_ROOT/.workdir/dist/current"

CC=${CC:-clang}
CFLAGS=(
    -std=c11
    -O3
    -flto=thin
    -march=x86-64-v3
    -fno-omit-frame-pointer
    -DNDEBUG
    -D_XOPEN_SOURCE=700
    -Wall -Wextra -Wpedantic
    -I "$HERE/include"
)
LDFLAGS=(
    -flto=thin
    -L "$DIST/lib"
    -Wl,-rpath,"$DIST/lib"
    -lotterbrix
    -lstdc++
    -lm
)

OUT="$BENCH_ROOT/code/c/bin"
mkdir -p "$OUT"

echo "[build] compiling shared object files…"
"$CC" "${CFLAGS[@]}" -c "$HERE/src/bench_data.c" -o "$OUT/bench_data.o"

shopt -s nullglob
SCENARIOS=("$HERE"/src/s*_*.c)
if [ ${#SCENARIOS[@]} -eq 0 ]; then
    echo "[build] no scenarios in $HERE/src" >&2
    exit 1
fi

for src in "${SCENARIOS[@]}"; do
    name=$(basename "$src" .c)
    echo "[build] $name"
    "$CC" "${CFLAGS[@]}" "$src" "$OUT/bench_data.o" \
          "${LDFLAGS[@]}" -o "$OUT/$name"
done

echo "[build] done -> $OUT"
