#!/usr/bin/env python3
"""Run DINAH's release benchmarks and archive comparable environment metadata."""

from __future__ import annotations

import os
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCHMARKS = ("runtime", "memory", "compute", "ecs", "simulation")


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

    for name in BENCHMARKS:
        print(f"\n=== cargo bench --bench {name} ===", flush=True)
        run(
            ["cargo", "bench", "--bench", name],
            output=artifact_dir / f"{name}-benchmark-results.txt",
        )

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
