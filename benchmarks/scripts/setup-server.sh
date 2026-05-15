#!/usr/bin/env bash
set -euo pipefail

# Provisions a clean server with everything the benchmark suite needs.
# Supports Debian/Ubuntu (apt-get) and Fedora (dnf).
#
# Installs:
#   - system packages: git, curl, ca-certificates, build toolchain (gcc, g++,
#     make, cmake, ninja, clang), pkg-config, openssl headers, python3 + pipx,
#     kernel-tools (cpupower)
#   - conan via pipx (PEP 668-friendly)
#   - rustup + stable rust toolchain (rustc, cargo)
#
# After this script:
#   sudo ./scripts/tune-host.sh   # optional, low-noise CPU profile
#   ./scripts/bootstrap.sh        # build libotterbrix, rust + c bins, data
#   ./scripts/run_stage.sh        # one stage of measurements

log() { printf '[setup-server] %s\n' "$*"; }
fail() { printf '[setup-server] FATAL: %s\n' "$*" >&2; exit 1; }

if [[ $EUID -eq 0 ]]; then
    SUDO=""
else
    if ! command -v sudo >/dev/null 2>&1; then
        fail "must be run as root or have sudo installed"
    fi
    SUDO="sudo"
fi

if command -v apt-get >/dev/null 2>&1; then
    log "detected apt-get (Debian/Ubuntu)"
    export DEBIAN_FRONTEND=noninteractive
    $SUDO apt-get update -y
    $SUDO apt-get install -y --no-install-recommends \
        git curl ca-certificates gnupg \
        build-essential cmake ninja-build clang \
        bison flex \
        pkg-config libssl-dev \
        python3 python3-pip pipx \
        linux-tools-common "linux-tools-$(uname -r)" || \
    $SUDO apt-get install -y --no-install-recommends \
        git curl ca-certificates gnupg \
        build-essential cmake ninja-build clang \
        bison flex \
        pkg-config libssl-dev \
        python3 python3-pip pipx \
        linux-tools-common linux-tools-generic
elif command -v dnf >/dev/null 2>&1; then
    log "detected dnf (Fedora / RHEL family)"
    $SUDO dnf install -y \
        git curl ca-certificates gnupg2 \
        gcc gcc-c++ make cmake ninja-build clang \
        bison flex \
        pkgconf-pkg-config openssl-devel \
        python3 python3-pip pipx \
        kernel-tools
else
    fail "unsupported distro: need apt-get or dnf"
fi

log "ensuring \$HOME/.local/bin and \$HOME/.cargo/bin are on PATH"
case ":${PATH:-}:" in
    *":$HOME/.local/bin:"*) ;;
    *) export PATH="$HOME/.local/bin:$PATH" ;;
esac
case ":${PATH:-}:" in
    *":$HOME/.cargo/bin:"*) ;;
    *) export PATH="$HOME/.cargo/bin:$PATH" ;;
esac
pipx ensurepath >/dev/null 2>&1 || true

log "installing conan via pipx..."
if pipx list 2>/dev/null | grep -q '^\s*package conan '; then
    pipx upgrade conan
else
    pipx install conan
fi

log "ensuring conan profile + otterbrix remote..."
conan profile detect --force >/dev/null
if ! conan remote list 2>/dev/null | grep -q '^otterbrix:'; then
    conan remote add otterbrix http://conan.otterbrix.com
fi

log "installing rustup + stable rust toolchain..."
if ! command -v rustup >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --no-modify-path
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
fi
rustup default stable >/dev/null
rustup update stable >/dev/null

log "verifying installed versions:"
{
    printf '  %-10s %s\n' git       "$(git --version)"
    printf '  %-10s %s\n' cmake     "$(cmake --version | head -n1)"
    printf '  %-10s %s\n' ninja     "$(ninja --version)"
    printf '  %-10s %s\n' clang     "$(clang --version | head -n1)"
    printf '  %-10s %s\n' g++       "$(g++ --version | head -n1)"
    printf '  %-10s %s\n' python3   "$(python3 --version)"
    printf '  %-10s %s\n' conan     "$(conan --version)"
    printf '  %-10s %s\n' rustc     "$(rustc --version)"
    printf '  %-10s %s\n' cargo     "$(cargo --version)"
} 2>&1

log "done. Restart shell (or 'source \$HOME/.cargo/env') so PATH picks up rust/conan."
log "next: sudo ./scripts/tune-host.sh   # optional"
log "      ./scripts/bootstrap.sh"
