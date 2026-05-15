#!/usr/bin/env bash
set -euo pipefail

# Tunes the host for low-noise benchmarking. Requires root.
# Mirrors the requirements documented in methodology.md §5 "Контроль шума".
#
# Applies (idempotent):
#   - turbo boost off              (intel_pstate/no_turbo  or cpufreq/boost)
#   - scaling governor performance (cpupower or per-cpu sysfs)
#   - SMT / hyperthreading off     (/sys/devices/system/cpu/smt/control)
#   - swap off
#
# Reverts to factory defaults after reboot (sysfs settings are not persisted).
# Re-run after every reboot of the dedicated host before a series of stages.

log() { printf '[tune-host] %s\n' "$*"; }
warn() { printf '[tune-host] WARN: %s\n' "$*" >&2; }
fail() { printf '[tune-host] FATAL: %s\n' "$*" >&2; exit 1; }

if [[ $EUID -ne 0 ]]; then
    fail "must be run as root (sudo $0)"
fi

log "1/4  turbo boost -> off"
if [[ -w /sys/devices/system/cpu/intel_pstate/no_turbo ]]; then
    echo 1 > /sys/devices/system/cpu/intel_pstate/no_turbo
    log "     intel_pstate/no_turbo = 1"
elif [[ -w /sys/devices/system/cpu/cpufreq/boost ]]; then
    echo 0 > /sys/devices/system/cpu/cpufreq/boost
    log "     cpufreq/boost = 0 (AMD)"
else
    warn "     no turbo control sysfs node found, skipping"
fi

log "2/4  scaling governor -> performance"
if command -v cpupower >/dev/null 2>&1; then
    cpupower frequency-set -g performance >/dev/null
else
    warn "     cpupower not installed, falling back to per-cpu sysfs"
    shopt -s nullglob
    for c in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        echo performance > "$c" 2>/dev/null || true
    done
    shopt -u nullglob
fi
log "     governor on cpu0 = $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"

log "3/4  SMT / hyperthreading -> off"
if [[ -w /sys/devices/system/cpu/smt/control ]]; then
    echo off > /sys/devices/system/cpu/smt/control
    log "     smt/active = $(cat /sys/devices/system/cpu/smt/active)"
else
    warn "     /sys/devices/system/cpu/smt/control not writable; disable HT in BIOS instead"
fi

log "4/4  swap -> off"
if swapoff -a 2>/dev/null; then
    swap_used="$(free -h | awk '/^Swap:/ {print $3}')"
    log "     swap used now = $swap_used"
else
    warn "     swapoff failed; check 'free -h'"
fi

echo
log "summary:"
printf '  %-10s %s\n' governor "$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
printf '  %-10s %s\n' smt      "$(cat /sys/devices/system/cpu/smt/active 2>/dev/null || echo unknown)"
printf '  %-10s %s\n' swap     "$(free -h | awk '/^Swap:/ {print $2 " total / " $3 " used"}')"
echo
log "next: pin run_stage.sh to physical cores, e.g."
log "      taskset -c 2,3 ./scripts/run_stage.sh"
