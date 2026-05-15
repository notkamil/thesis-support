#!/usr/bin/env bash
DIR=/home/notkamil/ITMO/S8/diploma/thesis-support/sanitizers/reports/valgrind
PROGRESS=$DIR/progress.log
SUPP=$DIR/valgrind.supp

binary="$1"; shift
name=$(basename "$binary")
ts=$(date +%H:%M:%S)

if [[ "$name" == *large_dataset* ]]; then
    echo "[skip] $ts $name" >> "$PROGRESS"
    exit 0
fi

echo "[start] $ts $name" >> "$PROGRESS"
t_begin=$(date +%s)

valgrind --tool=memcheck --error-exitcode=42 --leak-check=full \
         --show-leak-kinds=definite,indirect --track-origins=no \
         --num-callers=30 \
         --suppressions="$SUPP" \
         --child-silent-after-fork=yes \
         "$binary" "$@"
rc=$?

t_end=$(date +%s)
dur=$((t_end - t_begin))
echo "[done rc=$rc dur=${dur}s] $(date +%H:%M:%S) $name" >> "$PROGRESS"
exit $rc
