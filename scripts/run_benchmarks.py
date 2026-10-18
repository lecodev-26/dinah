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
RUNTIME_REPETITIONS = int(os.environ.get("DINAH_RUNTIME_BENCH_REPETITIONS", "3"))

if not 1 <= RUNTIME_REPETITIONS <= 5:
    raise ValueError("DINAH_RUNTIME_BENCH_REPETITIONS must be between 1 and 5")


def case_medians(cases: list[str]) -> dict[str, int]:
    """Extract the executable-reported median for each named benchmark case."""
    result: dict[str, int] = {}
    for line in cases:
        fields = dict(
            field.split("=", 1)
            for field in line.split()
            if "=" in field
        )
        name = fields.get("case")
        median = fields.get("median_total_ns")
        if name is not None and median is not None:
            result[name] = int(median)
    return result


def repeated_case_summary(repetitions: list[dict[str, object]]) -> dict[str, object]:
    """Summarize independent Runtime process medians without implying significance."""
    values_by_case: dict[str, list[int]] = {}
    for repetition in repetitions:
        cases = repetition["cases"]
        assert isinstance(cases, list)
        for name, median_ns in case_medians(cases).items():
            values_by_case.setdefault(name, []).append(median_ns)

    summary: dict[str, object] = {}
    for name, values in sorted(values_by_case.items()):
        ordered = sorted(values)
        summary[name] = {
            "runs": len(values),
            "median_of_run_medians_ns": ordered[len(ordered) // 2],
            "min_run_median_ns": ordered[0],
            "max_run_median_ns": ordered[-1],
            "run_medians_ns": values,
        }

    comparisons: dict[str, object] = {}
    for workload in ("noop", "cpu_1m"):
        for workers in (2, 4):
            scheduler_name = f"scheduler_queue_parallel_{workload}_{workers}_workers"
            pool_name = f"persistent_pool_{workload}_{workers}_workers"
            scheduler_values = values_by_case.get(scheduler_name, [])
            pool_values = values_by_case.get(pool_name, [])
            if not scheduler_values or not pool_values:
                continue
            scheduler_median = sorted(scheduler_values)[len(scheduler_values) // 2]
            pool_median = sorted(pool_values)[len(pool_values) // 2]
            comparisons[f"{workload}_{workers}_workers"] = {
                "scheduler_median_of_run_medians_ns": scheduler_median,
                "persistent_pool_median_of_run_medians_ns": pool_median,
                "scheduler_to_pool_ratio": round(scheduler_median / pool_median, 3),
                "note": "steady-state pool comparison excludes per-call worker startup; ratio is workload-specific",
            }

    lifecycle_comparisons: dict[str, object] = {}
    for workers in (2, 4):
        scheduler_name = f"scheduler_queue_parallel_noop_{workers}_workers"
        pool_name = f"persistent_pool_noop_lifecycle_{workers}_workers"
        scheduler_values = values_by_case.get(scheduler_name, [])
        pool_values = values_by_case.get(pool_name, [])
        if not scheduler_values or not pool_values:
            continue
        scheduler_median = sorted(scheduler_values)[len(scheduler_values) // 2]
        pool_median = sorted(pool_values)[len(pool_values) // 2]
        lifecycle_comparisons[f"noop_{workers}_workers"] = {
            "scheduler_median_of_run_medians_ns": scheduler_median,
            "pool_lifecycle_median_of_run_medians_ns": pool_median,
            "scheduler_to_pool_lifecycle_ratio": round(scheduler_median / pool_median, 3),
            "note": "both include per-call worker startup/shutdown; queueing, result/error semantics and worker dispatch still differ",
        }
    return {
        "cases": summary,
        "scheduler_vs_persistent_pool": comparisons,
        "scheduler_vs_pool_lifecycle": lifecycle_comparisons,
    }


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
        repetitions = RUNTIME_REPETITIONS if name == "runtime" else 1
        repetition_results: list[dict[str, object]] = []
        total_elapsed_seconds = 0.0
        for repetition in range(1, repetitions + 1):
            suffix = f"-run-{repetition}" if repetitions > 1 else ""
            output_path = artifact_dir / f"{name}-benchmark-results{suffix}.txt"
            print(
                f"\n=== cargo bench --bench {name}"
                f" (independent run {repetition}/{repetitions}) ===",
                flush=True,
            )
            started = time.perf_counter()
            result = run(["cargo", "bench", "--bench", name], output=output_path)
            elapsed_seconds = time.perf_counter() - started
            total_elapsed_seconds += elapsed_seconds
            cases = [
                line.strip()
                for line in result.splitlines()
                if line.strip().startswith("case=")
            ]
            repetition_results.append(
                {
                    "run": repetition,
                    "elapsed_seconds": round(elapsed_seconds, 3),
                    "cases": cases,
                }
            )
            # Preserve the original filename and schema's top-level cases for
            # existing consumers, using the first independent run as the baseline.
            if repetitions > 1 and repetition == 1:
                (artifact_dir / f"{name}-benchmark-results.txt").write_text(
                    result, encoding="utf-8"
                )

        first_cases = repetition_results[0]["cases"]
        suite_results[name] = {
            "elapsed_seconds": round(total_elapsed_seconds, 3),
            "case_count": len(first_cases),
            "cases": first_cases,
            "repetitions": repetition_results,
        }

    runtime_repetitions = suite_results["runtime"]["repetitions"]
    runtime_repeat_summary = repeated_case_summary(runtime_repetitions)

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
            "Runtime repeated_case_summary aggregates per-process medians; it is descriptive, not a confidence interval.",
            "The persistent pool is a benchmark-only prototype and excludes per-call worker startup.",
            "This is benchmark-based profiling, not a sampled CPU or allocation profile.",
        ],
        "runtime_repeated_case_summary": runtime_repeat_summary,
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
        f"runtime_independent_repetitions={RUNTIME_REPETITIONS}",
    ]
    for name, details in suite_results.items():
        repetition_count = len(details["repetitions"])
        summary.append(
            f"suite={name} elapsed_seconds={details['elapsed_seconds']} "
            f"case_count={details['case_count']} repetitions={repetition_count}"
        )
    summary.append("Runtime repeated case medians (median of process medians; descriptive only):")
    for case_name, details in runtime_repeat_summary["cases"].items():
        summary.append(
            f"case={case_name} runs={details['runs']} "
            f"median_of_run_medians_ns={details['median_of_run_medians_ns']} "
            f"min_run_median_ns={details['min_run_median_ns']} "
            f"max_run_median_ns={details['max_run_median_ns']}"
        )
    summary.append("Runtime scheduler vs benchmark-only persistent pool (steady-state; workload-specific):")
    for case_name, details in runtime_repeat_summary["scheduler_vs_persistent_pool"].items():
        summary.append(
            f"comparison={case_name} scheduler_ns={details['scheduler_median_of_run_medians_ns']} "
            f"pool_ns={details['persistent_pool_median_of_run_medians_ns']} "
            f"ratio={details['scheduler_to_pool_ratio']}x"
        )
    summary.append("Runtime scheduler vs pool with per-sample startup/shutdown (workload-specific):")
    for case_name, details in runtime_repeat_summary["scheduler_vs_pool_lifecycle"].items():
        summary.append(
            f"comparison={case_name} scheduler_ns={details['scheduler_median_of_run_medians_ns']} "
            f"pool_lifecycle_ns={details['pool_lifecycle_median_of_run_medians_ns']} "
            f"ratio={details['scheduler_to_pool_lifecycle_ratio']}x"
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
