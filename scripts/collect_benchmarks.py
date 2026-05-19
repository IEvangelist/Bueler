#!/usr/bin/env python3
"""Collect criterion bench output and append a run to benchmarks.json.

Reads `target/criterion/<group>/<bench>/new/estimates.json` for every
bench that ran and produces a single JSON entry which is appended to
`examples/showcase/benchmarks.json`. Designed to be invoked by the weekly
`.github/workflows/benchmarks.yml` workflow after `cargo bench`, but it
also works locally:

    cargo bench -p bueler-core --bench reactive -- --output-format=bencher
    python3 scripts/collect_benchmarks.py

The benchmarks file has the shape:

    {
      "schema": 1,
      "runs": [
        {
          "timestamp": "2026-05-11T07:23:14Z",
          "commit": "abc1234567...",
          "commit_short": "abc1234",
          "results": {
            "signal/create": { "median_ns": 40.2, "lower_ns": 39.8, "upper_ns": 41.0 },
            ...
          }
        },
        ...
      ]
    }

Only the last 52 runs (≈ 1 year of weekly samples) are retained.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CRITERION_DIR = REPO_ROOT / "target" / "criterion"
BENCHMARKS_FILE = REPO_ROOT / "examples" / "showcase" / "benchmarks.json"
MAX_RUNS = 52

# Benches we expect — anything else is ignored, anything missing is flagged.
EXPECTED = {
    "signal/create",
    "signal/read",
    "signal/write",
    "effect/run",
    "memo/recompute",
    "batch/coalesce",
    "watch/trigger",
}


def git(*args: str) -> str:
    out = subprocess.check_output(["git", *args], cwd=REPO_ROOT)
    return out.decode().strip()


def collect_results() -> dict[str, dict[str, float]]:
    if not CRITERION_DIR.exists():
        sys.exit(
            f"error: {CRITERION_DIR} does not exist — did `cargo bench` run?"
        )

    results: dict[str, dict[str, float]] = {}
    for estimates in CRITERION_DIR.glob("*/*/new/estimates.json"):
        # path: target/criterion/<group>/<bench>/new/estimates.json
        group = estimates.parent.parent.parent.name
        bench = estimates.parent.parent.name
        name = f"{group}/{bench}"
        if name not in EXPECTED:
            # Ignore criterion's auxiliary "change" entries and any
            # benches we haven't formally documented.
            continue
        data = json.loads(estimates.read_text())
        median = data["median"]
        results[name] = {
            "median_ns": round(median["point_estimate"], 3),
            "lower_ns": round(median["confidence_interval"]["lower_bound"], 3),
            "upper_ns": round(median["confidence_interval"]["upper_bound"], 3),
        }

    missing = EXPECTED - set(results)
    if missing:
        sys.exit(
            f"error: missing bench results for: {sorted(missing)}. "
            "Did the bench harness change?"
        )
    return results


def load_history() -> dict:
    if BENCHMARKS_FILE.exists():
        with BENCHMARKS_FILE.open(encoding="utf-8") as f:
            data = json.load(f)
        data.setdefault("schema", 1)
        data.setdefault("runs", [])
        return data
    return {"schema": 1, "runs": []}


def main() -> None:
    results = collect_results()
    sha = os.environ.get("GITHUB_SHA") or git("rev-parse", "HEAD")
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    entry = {
        "timestamp": timestamp,
        "commit": sha,
        "commit_short": sha[:7],
        "results": results,
    }

    history = load_history()
    history["runs"].append(entry)
    history["runs"] = history["runs"][-MAX_RUNS:]

    BENCHMARKS_FILE.parent.mkdir(parents=True, exist_ok=True)
    with BENCHMARKS_FILE.open("w", encoding="utf-8") as f:
        json.dump(history, f, indent=2)
        f.write("\n")

    summary = ", ".join(
        f"{name}={results[name]['median_ns']}ns" for name in sorted(results)
    )
    print(f"appended run @ {timestamp} ({sha[:7]}): {summary}")


if __name__ == "__main__":
    main()
