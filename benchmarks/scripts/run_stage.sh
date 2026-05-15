#!/usr/bin/env bash
set -euo pipefail

BENCH_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RAW_ROOT="$BENCH_ROOT/results/raw"
DIST_LIB="$BENCH_ROOT/.workdir/dist/current/lib"

usage() {
    cat <<USAGE
Usage: $0 [options]

Runs every (scenario, layer) combination once and dumps per-iteration CSVs
plus a label.json into results/raw/stage_<N>_<UTC-timestamp>/.

Options:
  --stage N                explicit stage number (default: max existing + 1)
  --scenarios s1,s2,...    comma-separated subset of scenarios (default: s1..s10)
  --layers c,sys,...       comma-separated subset of layers (default: c,sys,otterbrix,seaorm,sqlx)
  --dry-run                print what would be run, do not create the stage dir
  -h, --help               show this help and exit

Layers:
  c          plain-C reference, baseline for the summary
  sys        otterbrix-sys (raw FFI bindings)
  otterbrix  the safe Rust wrapper
  seaorm     seaorm-otterbrix
  sqlx       sqlx-otterbrix

Scenario s7 (interactive) only has seaorm and sqlx; other layers are
skipped automatically.
USAGE
}

STAGE_OVERRIDE=""
SCENARIOS_OVERRIDE=""
LAYERS_OVERRIDE=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --stage)     STAGE_OVERRIDE="$2"; shift 2 ;;
        --scenarios) SCENARIOS_OVERRIDE="$2"; shift 2 ;;
        --layers)    LAYERS_OVERRIDE="$2"; shift 2 ;;
        --dry-run)   DRY_RUN=1; shift ;;
        -h|--help)   usage; exit 0 ;;
        *)           echo "unknown flag: $1" >&2; usage >&2; exit 2 ;;
    esac
done

SCENARIOS_DEFAULT="s1,s2,s3,s4,s5,s6,s7,s8,s9,s10"
LAYERS_DEFAULT="c,sys,otterbrix,seaorm,sqlx"
SCENARIOS="${SCENARIOS_OVERRIDE:-$SCENARIOS_DEFAULT}"
LAYERS="${LAYERS_OVERRIDE:-$LAYERS_DEFAULT}"

if [[ ! -d "$DIST_LIB" ]]; then
    echo "fatal: $DIST_LIB missing; run ./scripts/bootstrap.sh first" >&2
    exit 1
fi
if [[ ! -d "$BENCH_ROOT/code/c/bin" ]]; then
    echo "fatal: $BENCH_ROOT/code/c/bin missing; run ./scripts/bootstrap.sh first" >&2
    exit 1
fi
if [[ ! -d "$BENCH_ROOT/target/release" ]]; then
    echo "fatal: $BENCH_ROOT/target/release missing; run ./scripts/bootstrap.sh first" >&2
    exit 1
fi
if [[ ! -f "$BENCH_ROOT/data/rows_max.bin" ]] || [[ ! -f "$BENCH_ROOT/data/lookup_ids_max.bin" ]]; then
    echo "fatal: bench input data missing in $BENCH_ROOT/data; run ./scripts/bootstrap.sh (or cargo run --release --bin gen_bench_data)" >&2
    exit 1
fi

mkdir -p "$RAW_ROOT"

if [[ -n "$STAGE_OVERRIDE" ]]; then
    if [[ ! "$STAGE_OVERRIDE" =~ ^[0-9]+$ ]]; then
        echo "fatal: --stage must be a non-negative integer, got '$STAGE_OVERRIDE'" >&2
        exit 2
    fi
    STAGE_N="$STAGE_OVERRIDE"
else
    MAX=0
    for d in "$RAW_ROOT"/stage_*_*/; do
        [[ -d "$d" ]] || continue
        n="$(basename "$d" | sed -E 's/^stage_([0-9]+)_.*/\1/')"
        if [[ "$n" =~ ^[0-9]+$ && "$n" -gt "$MAX" ]]; then
            MAX="$n"
        fi
    done
    STAGE_N=$((MAX + 1))
fi

TS="$(date -u +%Y%m%dT%H%M%SZ)"
STAGE_DIR="$RAW_ROOT/stage_${STAGE_N}_${TS}"
LOG_FILE="$STAGE_DIR/_run.log"

if [[ $DRY_RUN -eq 0 ]]; then
    mkdir -p "$STAGE_DIR"
fi

log() {
    local msg
    msg="[stage $STAGE_N] $*"
    if [[ $DRY_RUN -eq 0 ]]; then
        printf '%s\n' "$msg" | tee -a "$LOG_FILE"
    else
        printf '%s\n' "$msg"
    fi
}

log "scenarios: $SCENARIOS"
log "layers:    $LAYERS"
log "output:    $STAGE_DIR"
[[ $DRY_RUN -eq 1 ]] && log "(dry-run)"

export LD_LIBRARY_PATH="$DIST_LIB${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export OTTERBRIX_BENCH_RESULTS_DIR="$STAGE_DIR"
export BENCH_RESULTS_DIR="$STAGE_DIR"
export BENCH_DATA_DIR="$BENCH_ROOT/data"
export TMPDIR="${TMPDIR:-/tmp}"

if [[ $DRY_RUN -eq 0 ]]; then
    python3 "$BENCH_ROOT/scripts/_collect_label.py" \
        --output "$STAGE_DIR/label.json" \
        --stage "$STAGE_N" \
        --timestamp "$TS" \
        --scenarios "$SCENARIOS" \
        --layers "$LAYERS" \
        --bench-root "$BENCH_ROOT"
fi

declare -A BIN
BIN[s1]=s1_headline
BIN[s2]=s2_insert
BIN[s3]=s3_range_scan
BIN[s4]=s4_round_trip
BIN[s5]=s5_bulk_insert
BIN[s6]=s6_open
BIN[s7]=s7_interactive
BIN[s8]=s8_aggregation
BIN[s9]=s9_join
BIN[s10]=s10_mutate

declare -A SCEN_ARGS
SCEN_ARGS[s10]="update delete"

RAN=0
FAILED=0
SKIPPED=0
START_TS=$(date -u +%s)

IFS=',' read -ra SCEN_ARR  <<< "$SCENARIOS"
IFS=',' read -ra LAYER_ARR <<< "$LAYERS"

for scen in "${SCEN_ARR[@]}"; do
    bin_base="${BIN[$scen]:-}"
    if [[ -z "$bin_base" ]]; then
        log "WARN: unknown scenario '$scen', skipping"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi
    for layer in "${LAYER_ARR[@]}"; do
        if [[ "$scen" == "s7" && "$layer" != "seaorm" && "$layer" != "sqlx" ]]; then
            continue
        fi
        if [[ "$layer" == "c" ]]; then
            EXE="$BENCH_ROOT/code/c/bin/$bin_base"
        else
            EXE="$BENCH_ROOT/target/release/${bin_base}_${layer}"
        fi
        if [[ ! -x "$EXE" ]]; then
            log "WARN: missing binary $EXE, skipping"
            SKIPPED=$((SKIPPED + 1))
            continue
        fi
        ARGS_LIST="${SCEN_ARGS[$scen]:-}"
        if [[ -z "$ARGS_LIST" ]]; then
            ARGS_LIST="__none__"
        fi
        for ARG in $ARGS_LIST; do
            if [[ "$ARG" == "__none__" ]]; then
                tag="${bin_base} [${layer}]"
                EXE_ARGS=()
            else
                tag="${bin_base} [${layer}] ${ARG}"
                EXE_ARGS=("$ARG")
            fi
            log "running ${tag}..."
            if [[ $DRY_RUN -eq 1 ]]; then
                log "  (dry-run) would execute $EXE ${EXE_ARGS[*]}"
                continue
            fi
            t_start=$(date -u +%s)
            if "$EXE" "${EXE_ARGS[@]}" >>"$LOG_FILE" 2>&1; then
                t_end=$(date -u +%s)
                log "  ok ($((t_end - t_start)) s)"
                RAN=$((RAN + 1))
            else
                rc=$?
                log "  FAIL: exit $rc"
                FAILED=$((FAILED + 1))
            fi
        done
    done
done

END_TS=$(date -u +%s)
log "stage $STAGE_N done: ran=$RAN failed=$FAILED skipped=$SKIPPED  total=$((END_TS - START_TS))s"
log "raw -> $STAGE_DIR"

if [[ $FAILED -gt 0 ]]; then
    exit 1
fi
