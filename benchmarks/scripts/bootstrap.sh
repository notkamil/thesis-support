#!/usr/bin/env bash
set -euo pipefail

BENCH_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKDIR="$BENCH_ROOT/.workdir"
CONFIG="$BENCH_ROOT/benchmarks.config"

if [[ ! -f "$CONFIG" ]]; then
    echo "fatal: $CONFIG missing" >&2
    exit 1
fi
source "$CONFIG"

REV="$DEFAULT_REV"
REPO="$REPO_URL"
REBUILD=0
LOCAL_SRC=""
SKIP_LIB=0
SKIP_RUST=0
SKIP_C=0
SKIP_DATA=0

usage() {
    cat <<USAGE
Usage: $0 [options]

Options:
  --rev <ref>            git ref (branch / tag / SHA) of otterbrix to build
  --repo <url>           override the otterbrix git remote
  --rebuild              force rebuild of libotterbrix.so even if dist exists
  --local-source <path>  use a local checkout of otterbrix instead of git clone
  --skip-lib             skip the libotterbrix.so build step
  --skip-rust            skip cargo build of the Rust workspace
  --skip-c               skip the C benchmarks build
  --skip-data            skip generation of the deterministic bench input data
  -h, --help             show this help and exit

By default, this script provisions every dependency the benchmark
suite needs:
  1. clone or update otterbrix into .workdir/repo/
  2. build libotterbrix.so via Conan + CMake + Ninja
  3. assemble .workdir/dist/<sha>/{lib,include}/ + manifest.json
  4. cargo build --release of the Rust workspace
  5. clang build of the C benchmarks (code/c/build.sh)
  6. cargo run --release --bin gen_bench_data (writes data/*.bin)
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --rev) REV="$2"; shift 2 ;;
        --repo) REPO="$2"; shift 2 ;;
        --rebuild) REBUILD=1; shift ;;
        --local-source) LOCAL_SRC="$2"; shift 2 ;;
        --skip-lib) SKIP_LIB=1; shift ;;
        --skip-rust) SKIP_RUST=1; shift ;;
        --skip-c) SKIP_C=1; shift ;;
        --skip-data) SKIP_DATA=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown flag: $1" >&2; usage >&2; exit 2 ;;
    esac
done

log() { printf '[bootstrap] %s\n' "$*"; }

require() {
    local bin="$1"
    if ! command -v "$bin" >/dev/null 2>&1; then
        echo "fatal: required tool '$bin' is not installed" >&2
        exit 1
    fi
}

log "checking prerequisites..."
require git
require cmake
require ninja
require conan
require g++
require clang
require cargo
require rustc
require python3

mkdir -p "$WORKDIR"

REPO_DIR="$WORKDIR/repo"

if [[ -n "$LOCAL_SRC" ]]; then
    log "using local source: $LOCAL_SRC (NOT for reportable runs)"
    rm -rf "$REPO_DIR"
    ln -s "$(realpath "$LOCAL_SRC")" "$REPO_DIR"
elif [[ ! -d "$REPO_DIR/.git" ]]; then
    log "cloning $REPO into $REPO_DIR..."
    git clone "$REPO" "$REPO_DIR"
else
    log "updating origin URL to $REPO..."
    git -C "$REPO_DIR" remote set-url origin "$REPO"
fi

if [[ -z "$LOCAL_SRC" ]]; then
    log "fetching $REV from origin..."
    git -C "$REPO_DIR" fetch --tags origin "$REV"
    log "checking out FETCH_HEAD ($REV)..."
    git -C "$REPO_DIR" -c advice.detachedHead=false checkout --detach FETCH_HEAD
    git -C "$REPO_DIR" reset --hard HEAD
fi

INNER_WS="$REPO_DIR/integration/rust/Cargo.toml"
if [[ -f "$INNER_WS" ]]; then
    log "neutralizing inner workspace manifest in cloned tree..."
    mv -f "$INNER_WS" "$INNER_WS.upstream-disabled"
fi

SHA_FULL=$(git -C "$REPO_DIR" rev-parse HEAD)
SHA_SHORT=$(git -C "$REPO_DIR" rev-parse --short=12 HEAD)
DIRTY=0
if [[ -n "$LOCAL_SRC" ]]; then
    if [[ -n "$(git -C "$REPO_DIR" status --porcelain)" ]]; then
        DIRTY=1
    fi
fi

DIST_DIR="$WORKDIR/dist/$SHA_SHORT"
MANIFEST="$DIST_DIR/manifest.json"

if [[ $SKIP_LIB -eq 1 ]]; then
    log "skipping libotterbrix build (--skip-lib)"
    if [[ ! -f "$MANIFEST" ]]; then
        echo "fatal: --skip-lib requested but $MANIFEST does not exist" >&2
        exit 1
    fi
elif [[ -f "$MANIFEST" && $REBUILD -eq 0 ]]; then
    log "dist for $SHA_SHORT already present, skipping rebuild (use --rebuild to force)"
else
    log "configuring and building libotterbrix from $SHA_SHORT..."
    BUILD_DIR="$REPO_DIR/build"
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"

    pushd "$BUILD_DIR" >/dev/null

    conan profile detect --exist-ok >/dev/null
    cp ../conanfile.py .
    conan install . \
        --build missing \
        -s build_type="$CMAKE_BUILD_TYPE" \
        -s compiler.cppstd=20

    TOOLCHAIN=$(find . -name conan_toolchain.cmake -print -quit)
    if [[ -z "$TOOLCHAIN" ]]; then
        echo "fatal: conan_toolchain.cmake not generated" >&2
        exit 1
    fi
    log "using conan toolchain at $TOOLCHAIN"

    EXTRA_CMAKE=""
    if [[ -n "$EXTRA_CXX_FLAGS" ]]; then
        EXTRA_CMAKE="-DCMAKE_CXX_FLAGS=$EXTRA_CXX_FLAGS"
    fi

    cmake .. -G Ninja \
        -DCMAKE_BUILD_TYPE="$CMAKE_BUILD_TYPE" \
        -DCMAKE_TOOLCHAIN_FILE="$TOOLCHAIN" \
        -DDEV_MODE=ON \
        $EXTRA_CMAKE

    JOBS_VAL="${JOBS:-$(nproc)}"
    cmake --build . --target c_otterbrix -- -j "$JOBS_VAL"

    popd >/dev/null

    log "assembling distribution at $DIST_DIR..."
    rm -rf "$DIST_DIR"
    mkdir -p "$DIST_DIR/lib" "$DIST_DIR/include"
    cp "$REPO_DIR/build/integration/c/libotterbrix.so" "$DIST_DIR/lib/"
    cp "$REPO_DIR/integration/c/otterbrix.h"           "$DIST_DIR/include/"

    HOST=$(uname -a)
    GCC_VER=$(g++ --version | head -n1)
    cat >"$MANIFEST" <<JSON
{
  "git_repo": "$REPO",
  "git_rev_requested": "$REV",
  "git_commit_full": "$SHA_FULL",
  "git_commit_short": "$SHA_SHORT",
  "local_source": "${LOCAL_SRC:-}",
  "dirty": $DIRTY,
  "cmake_build_type": "$CMAKE_BUILD_TYPE",
  "cxx_compiler": "$GCC_VER",
  "extra_cxx_flags": "$EXTRA_CXX_FLAGS",
  "built_at_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "host": "$HOST"
}
JSON
fi

log "updating .workdir/dist/current symlink -> $SHA_SHORT"
ln -sfn "$SHA_SHORT" "$WORKDIR/dist/current"

log "verifying libotterbrix.so..."
if [[ -z "$(nm -D "$DIST_DIR/lib/libotterbrix.so" 2>/dev/null | grep ' otterbrix_create$' || true)" ]]; then
    echo "fatal: libotterbrix.so does not export otterbrix_create" >&2
    exit 1
fi

if [[ $SKIP_RUST -eq 1 ]]; then
    log "skipping cargo build (--skip-rust)"
else
    log "building Rust workspace..."
    cargo build --release --workspace --manifest-path "$BENCH_ROOT/Cargo.toml"
fi

if [[ $SKIP_C -eq 1 ]]; then
    log "skipping C benchmarks build (--skip-c)"
else
    log "building C benchmarks..."
    "$BENCH_ROOT/code/c/build.sh"
fi

if [[ $SKIP_DATA -eq 1 ]]; then
    log "skipping bench input data generation (--skip-data)"
else
    log "generating deterministic bench input data..."
    cargo run --release --manifest-path "$BENCH_ROOT/Cargo.toml" \
              --bin gen_bench_data
fi

log "done. dist=$DIST_DIR"
log "next step: ./scripts/run_stage.sh"
