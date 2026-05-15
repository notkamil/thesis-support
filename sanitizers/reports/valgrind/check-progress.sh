#!/usr/bin/env bash
DIR=/home/notkamil/ITMO/S8/diploma/thesis-support/sanitizers/reports/valgrind
P=$DIR/progress.log

if [[ ! -s "$P" ]]; then
    echo "progress.log пуст или не существует"
    exit 1
fi

now=$(date +%s)
start=$(date -d "$(grep -m1 '^\[run-start\]' $P | awk '{print $2,$3}')" +%s 2>/dev/null)
elapsed=$((now - start))
mins=$((elapsed / 60))
secs=$((elapsed % 60))

started=$(grep -cE '^\[start\]' $P)
done_ok=$(grep -cE '^\[done rc=0' $P)
done_fail=$(grep -E '^\[done rc=' $P | grep -vc 'rc=0')
skipped=$(grep -cE '^\[skip\]' $P)
finished=$(grep -cE '^\[done' $P)
in_progress=$((started - finished))

run_end=$(grep -m1 '^\[run-end\]' $P)

current=$(awk '
    /^\[start\]/{name=$3}
    /^\[done/{name=""}
    END{print name}' "$P")

last_done=$(grep -E '^\[done' $P | tail -1)

echo "============================================="
echo "  Valgrind run progress"
echo "============================================="
echo "elapsed:       ${mins}m ${secs}s"
echo ""
echo "started:       $started binaries"
echo "  ok:          $done_ok"
echo "  with errors: $done_fail"
echo "skipped (large_dataset): $skipped"
echo "in progress:   $in_progress"
echo ""
if [[ -n "$current" ]]; then
    echo "now running:   $current"
fi
echo "last finished: $last_done"
echo ""
if [[ -n "$run_end" ]]; then
    echo "*** run finished: $run_end ***"
else
    echo "(still running)"
fi
echo ""
echo "--- last 8 progress lines ---"
tail -8 $P
