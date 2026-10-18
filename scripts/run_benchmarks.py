#!/usr/bin/env python3
"""Run DINAH's release benchmarks and archive comparable environment metadata."""

from __future__ import annotations

import json
import os
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCHMARKS = ("runtime", "memory", "compute", "ecs", "simulation", "network", "storage")


def run(command: list[str], *, output: Path | None = None) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    result = completed.stdout
    if output is not None:
        output.write_text(result, encoding="utf-8")
    print(result, end="" if result.endswith("\n") else "\n")
    return result


def main() -> int:
    artifact_dir = ROOT / "benchmark-results"
    artifact_dir.mkdir(exist_ok=True)
    metadata = [
        "DINAH benchmark environment",
        f"timestamp_utc={datetime.now(timezone.utc).isoformat()}",
        f"os={platform.platform()}",
        f"system={platform.system()}",
        f"release={platform.release()}",
        f"machine={platform.machine()}",
        f"processor={platform.processor() or 'not reported by platform'}",
        f"python={platform.python_version()}",
        f"runner_os={os.environ.get('RUNNER_OS', 'not set')}",
        f"runner_arch={os.environ.get('RUNNER_ARCH', 'not set')}",
        f"github_sha={os.environ.get('GITHUB_SHA', 'local')}",
        "command=python3 scripts/run_benchmarks.py",
        "profile=release (cargo bench)",
        "",
        "$ rustc -Vv",
        run(["rustc", "-Vv"]),
        "$ cargo --version",
        run(["cargo", "--version"]),
    ]
    (artifact_dir / "environment.txt").write_text(
        "\n".join(metadata) + "\n", encoding="utf-8"
    )

    suite_results: dict[str, dict[str, object]] = {}
    for name in BENCHMARKS:
        print(f"\n=== cargo bench --bench {name} ===", flush=True)
        started = time.perf_counter()
        result = run(
            ["cargo", "bench", "--bench", name],
            output=artifact_dir / f"{name}-benchmark-results.txt",
        )
        elapsed_seconds = time.perf_counter() - started
        cases = [line.strip() for line in result.splitlines() if line.strip().startswith("case=")]
        suite_results[name] = {
            "elapsed_seconds": round(elapsed_seconds, 3),
            "case_count": len(cases),
            "cases": cases,
        }

    # Keep this machine/commit-specific: values from unlike runners are not
    # silently merged into a universal performance claim.
    baseline = {
        "schema_version": 1,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "commit": os.environ.get("GITHUB_SHA", "local"),
        "environment": {
            "os": platform.platform(),
            "system": platform.system(),
            "machine": platform.machine(),
            "processor": platform.processor() or "not reported by platform",
            "rustc": run(["rustc", "-Vv"]).strip(),
            "cargo": run(["cargo", "--version"]).strip(),
        },
        "measurement_notes": [
            "Per-case measurements are reported by each benchmark executable.",
            "elapsed_seconds is suite wall time including process startup and compilation/cache effects.",
            "Compare cases only when workload, toolchain, and runner context match.",
            "This is benchmark-based profiling, not a sampled CPU or allocation profile.",
        ],
        "suites": suite_results,
    }
    (artifact_dir / "baseline.json").write_text(
        json.dumps(baseline, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    summary = [
        "DINAH benchmark baseline summary",
        f"commit={baseline['commit']}",
        f"timestamp_utc={baseline['timestamp_utc']}",
        f"runner={platform.system()} {platform.machine()}",
        "Note: suite wall time is not a per-operation performance metric.",
    ]
    for name, details in suite_results.items():
        summary.append(
            f"suite={name} elapsed_seconds={details['elapsed_seconds']} case_count={details['case_count']}"
        )
    (artifact_dir / "baseline-summary.txt").write_text("\n".join(summary) + "\n", encoding="utf-8")
    print("\n".join(summary))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        print(
            f"Benchmark command failed with exit code {error.returncode}: {error.cmd}",
            file=sys.stderr,
        )
        raise SystemExit(error.returncode)
