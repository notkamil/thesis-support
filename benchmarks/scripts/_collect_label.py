#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def _run(cmd: list[str]) -> str:
    try:
        return subprocess.check_output(cmd, text=True, stderr=subprocess.DEVNULL).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return ""


def _read_first(path: str, prefix: str, sep: str = ":") -> str:
    try:
        with open(path) as f:
            for line in f:
                if line.startswith(prefix):
                    return line.split(sep, 1)[1].strip()
    except OSError:
        pass
    return ""


def _os_pretty() -> str:
    try:
        with open("/etc/os-release") as f:
            for line in f:
                if line.startswith("PRETTY_NAME="):
                    return line.split("=", 1)[1].strip().strip('"')
    except OSError:
        pass
    return platform.platform()


def _cpu_model() -> str:
    return _read_first("/proc/cpuinfo", "model name")


def _ram_mb() -> int:
    raw = _read_first("/proc/meminfo", "MemTotal", sep=":")
    parts = raw.split()
    if len(parts) >= 1 and parts[0].isdigit():
        return int(parts[0]) // 1024
    return 0


def _otterbrix(bench_root: Path) -> dict:
    manifest = bench_root / ".workdir" / "dist" / "current" / "manifest.json"
    out = {"dist_path": "", "git_commit_short": "", "git_commit_full": "", "git_rev_requested": ""}
    if not manifest.exists():
        return out
    try:
        data = json.loads(manifest.read_text())
        out["dist_path"] = str(manifest.parent.resolve())
        out["git_commit_short"] = data.get("git_commit_short", "")
        out["git_commit_full"] = data.get("git_commit_full", "")
        out["git_rev_requested"] = data.get("git_rev_requested", "")
    except (OSError, json.JSONDecodeError):
        pass
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--output", required=True, type=Path)
    ap.add_argument("--stage", required=True, type=int)
    ap.add_argument("--timestamp", required=True)
    ap.add_argument("--scenarios", required=True)
    ap.add_argument("--layers", required=True)
    ap.add_argument("--bench-root", required=True, type=Path)
    args = ap.parse_args()

    label = {
        "stage": args.stage,
        "timestamp_utc": args.timestamp,
        "generated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "scenarios_requested": [s for s in args.scenarios.split(",") if s],
        "layers_requested": [s for s in args.layers.split(",") if s],
        "host": {
            "hostname": platform.node(),
            "kernel": platform.release(),
            "os": _os_pretty(),
            "arch": platform.machine(),
            "cpu_model": _cpu_model(),
            "cpu_cores_logical": os.cpu_count() or 0,
            "ram_mb": _ram_mb(),
        },
        "toolchain": {
            "rustc": _run(["rustc", "--version"]),
            "cargo": _run(["cargo", "--version"]),
            "clang": (_run(["clang", "--version"]).split("\n", 1) or [""])[0],
            "python": platform.python_version(),
        },
        "compile_flags": {
            "rust_profile": "release: opt-level=3, lto=thin, codegen-units=1, debug=0, overflow-checks=false, panic=abort",
            "rust_rustflags": "-C target-cpu=x86-64-v3",
            "c": "-std=c11 -O3 -flto=thin -march=x86-64-v3 -fno-omit-frame-pointer -DNDEBUG",
        },
        "otterbrix": _otterbrix(args.bench_root),
        "env": {
            "LD_LIBRARY_PATH": os.environ.get("LD_LIBRARY_PATH", ""),
            "TMPDIR": os.environ.get("TMPDIR", ""),
            "OTTERBRIX_BENCH_RESULTS_DIR": os.environ.get("OTTERBRIX_BENCH_RESULTS_DIR", ""),
            "BENCH_RESULTS_DIR": os.environ.get("BENCH_RESULTS_DIR", ""),
            "BENCH_DATA_DIR": os.environ.get("BENCH_DATA_DIR", ""),
        },
    }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(label, indent=2, ensure_ascii=False) + "\n")
    print(f"wrote {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
