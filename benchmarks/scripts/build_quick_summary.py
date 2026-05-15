#!/usr/bin/env python3
"""Aggregate raw per-iteration CSVs across stages.

Reads ``results/raw/stage_<N>_<timestamp>/*.csv`` and writes:

  results/aggregated/summary.md     — markdown table:
                                      median-of-medians + Δ% vs the C baseline
  results/aggregated/aggregated.json — full per-cell, per-stage stats
                                      (median, mean, p50, p95, p99, min, max, n)
                                      ready for plotting

Usage:
  ./scripts/build_quick_summary.py             # all stages found on disk
  ./scripts/build_quick_summary.py 1 3 5       # only stages 1, 3, 5
"""
from __future__ import annotations

import csv
import json
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

BENCH_ROOT = Path(__file__).resolve().parents[1]
RAW_ROOT = BENCH_ROOT / "results" / "raw"
OUT_DIR = BENCH_ROOT / "results" / "aggregated"

LAYERS = ["c", "sys", "otterbrix", "seaorm", "sqlx"]
LAYER_RE = re.compile(r"_(" + "|".join(LAYERS) + r")(_.+)?$")
STAGE_RE = re.compile(r"^stage_(\d+)_(.+)$")


def fmt_ns(ns: float) -> str:
    if ns < 1_000:
        return f"{ns:6.0f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:6.2f} \u00b5s"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:6.2f} ms"
    return f"{ns / 1_000_000_000:6.2f}  s"


def parse_csv_name(stem: str) -> tuple[str, str, str]:
    m = LAYER_RE.search(stem)
    if not m:
        return stem, "?", ""
    base = stem[: m.start()]
    layer = m.group(1)
    variant = (m.group(2) or "").lstrip("_")
    return base, layer, variant


def load_csv(path: Path) -> list[int]:
    with path.open() as f:
        r = csv.reader(f)
        next(r, None)
        return [int(row[0]) for row in r if row]


def percentile(samples: list[int], p: float) -> float:
    if not samples:
        return float("nan")
    s = sorted(samples)
    k = (len(s) - 1) * p
    lo = int(k)
    hi = min(lo + 1, len(s) - 1)
    return s[lo] + (s[hi] - s[lo]) * (k - lo)


def per_stage_stats(samples: list[int]) -> dict:
    return {
        "n": len(samples),
        "median": float(statistics.median(samples)),
        "mean": float(statistics.fmean(samples)),
        "p50": percentile(samples, 0.50),
        "p95": percentile(samples, 0.95),
        "p99": percentile(samples, 0.99),
        "min": int(min(samples)),
        "max": int(max(samples)),
    }


def discover_stages(filter_ids: list[int] | None) -> list[tuple[int, str, Path]]:
    stages: list[tuple[int, str, Path]] = []
    if not RAW_ROOT.exists():
        return stages
    for entry in sorted(RAW_ROOT.iterdir()):
        if not entry.is_dir():
            continue
        m = STAGE_RE.match(entry.name)
        if not m:
            continue
        sid = int(m.group(1))
        ts = m.group(2)
        if filter_ids is not None and sid not in filter_ids:
            continue
        stages.append((sid, ts, entry))
    stages.sort(key=lambda x: x[0])
    return stages


def collect(stages: list[tuple[int, str, Path]]) -> dict:
    cells: dict = defaultdict(lambda: defaultdict(list))
    for sid, _ts, stage_dir in stages:
        for csv_path in sorted(stage_dir.glob("*.csv")):
            scenario, layer, variant = parse_csv_name(csv_path.stem)
            samples = load_csv(csv_path)
            if not samples:
                continue
            stats = per_stage_stats(samples)
            stats["stage"] = sid
            cells[(scenario, variant)][layer].append(stats)
    return cells


def aggregate_across_stages(per_stage: list[dict]) -> dict:
    if not per_stage:
        return {}
    medians = [s["median"] for s in per_stage]
    return {
        "stages_used": [s["stage"] for s in per_stage],
        "median_of_medians": float(statistics.median(medians)),
        "mean_of_medians": float(statistics.fmean(medians)),
        "min_median": min(medians),
        "max_median": max(medians),
        "spread_pct": (
            (max(medians) - min(medians)) / statistics.median(medians) * 100
            if statistics.median(medians) > 0 else 0.0
        ),
    }


def build_aggregated(cells: dict, stages: list[tuple[int, str, Path]]) -> dict:
    out: dict = {
        "stages": [
            {"stage": sid, "timestamp": ts, "path": str(p.relative_to(BENCH_ROOT))}
            for sid, ts, p in stages
        ],
        "cells": [],
    }
    for (scenario, variant), per_layer in sorted(cells.items()):
        cell_layers: dict = {}
        for layer, stage_list in per_layer.items():
            cell_layers[layer] = {
                "per_stage": stage_list,
                "across": aggregate_across_stages(stage_list),
            }
        c_med = (
            cell_layers["c"]["across"]["median_of_medians"]
            if "c" in cell_layers else None
        )
        deltas = {}
        if c_med and c_med > 0:
            for layer, data in cell_layers.items():
                m = data["across"]["median_of_medians"]
                deltas[layer] = (m - c_med) / c_med * 100
        out["cells"].append({
            "scenario": scenario,
            "variant": variant,
            "layers": cell_layers,
            "delta_vs_c_pct": deltas,
        })
    return out


def render_summary_md(aggregated: dict) -> str:
    lines: list[str] = []
    lines.append("# Bench summary — median-of-medians (Δ% vs C)")
    lines.append("")
    stages = aggregated["stages"]
    if stages:
        ids = ", ".join(str(s["stage"]) for s in stages)
        lines.append(f"Stages aggregated: **{ids}** ({len(stages)} total).")
    else:
        lines.append("No stages found.")
        return "\n".join(lines) + "\n"
    lines.append("")
    lines.append(
        "Each cell shows `median-of-medians (max-spread%) [Δ% vs C]`. "
        "`max-spread%` is `(max_med - min_med) / median_of_medians × 100` "
        "across stages — a quick stability check. `Δ% vs C` is the relative "
        "difference of the cell's median-of-medians against the C baseline "
        "of the same row; the C column itself is shown as `0%`. For rows "
        "without a C baseline (e.g. s7 interactive) Δ is omitted."
    )
    lines.append("")

    header_layers = LAYERS
    lines.append("| scenario | variant | n/stage | " + " | ".join(header_layers) + " |")
    lines.append("|---|---|---:|" + "|".join(["---:"] * len(header_layers)) + "|")

    for cell in aggregated["cells"]:
        scenario = cell["scenario"]
        variant = cell["variant"] or "—"
        first_n = ""
        for layer_data in cell["layers"].values():
            if layer_data["per_stage"]:
                first_n = str(layer_data["per_stage"][0]["n"])
                break
        c_med = (
            cell["layers"]["c"]["across"]["median_of_medians"]
            if "c" in cell["layers"] else None
        )
        cells_md = []
        for layer in header_layers:
            data = cell["layers"].get(layer)
            if data is None:
                cells_md.append("—")
                continue
            across = data["across"]
            med = across["median_of_medians"]
            spread = across["spread_pct"]
            base = f"{fmt_ns(med)} ({spread:.1f}%)"
            if c_med and c_med > 0:
                delta = (med - c_med) / c_med * 100
                sign = "+" if delta >= 0 else ""
                base += f" [{sign}{delta:.0f}%]"
            cells_md.append(base)
        lines.append(f"| {scenario} | {variant} | {first_n} | " + " | ".join(cells_md) + " |")

    lines.append("")
    lines.append("## Coverage")
    lines.append("")
    lines.append(f"- Cells reported: {len(aggregated['cells'])}")
    rust_layers = {"sys", "otterbrix", "seaorm", "sqlx"}
    missing: list[str] = []
    for cell in aggregated["cells"]:
        scen = cell["scenario"]
        present = set(cell["layers"].keys())
        if scen.startswith("s7_"):
            expected = {"seaorm", "sqlx"}
        else:
            expected = rust_layers | {"c"}
        absent = expected - present
        if absent:
            tag = scen + (f" ({cell['variant']})" if cell["variant"] else "")
            missing.append(f"  - {tag}: {', '.join(sorted(absent))}")
    if missing:
        lines.append("- Missing layers (failures / not yet run):")
        lines.extend(missing)
    else:
        lines.append("- No missing layers")

    lines.append("")
    lines.append("## Stages")
    lines.append("")
    for s in stages:
        lines.append(f"- stage **{s['stage']}** at `{s['timestamp']}` → `{s['path']}`")

    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    filter_ids: list[int] | None = None
    if argv:
        try:
            filter_ids = sorted({int(a) for a in argv})
        except ValueError:
            print(f"usage: {Path(__file__).name} [stage_id ...]", file=sys.stderr)
            return 2

    stages = discover_stages(filter_ids)
    if not stages:
        if filter_ids is not None:
            print(f"no stages matched ids {filter_ids} under {RAW_ROOT}", file=sys.stderr)
        else:
            print(f"no stages found under {RAW_ROOT}", file=sys.stderr)
        return 1

    cells = collect(stages)
    aggregated = build_aggregated(cells, stages)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    json_path = OUT_DIR / "aggregated.json"
    md_path = OUT_DIR / "summary.md"
    json_path.write_text(json.dumps(aggregated, indent=2, ensure_ascii=False) + "\n")
    md_text = render_summary_md(aggregated)
    md_path.write_text(md_text)

    print(f"wrote {json_path}")
    print(f"wrote {md_path}")
    print()
    print(md_text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
