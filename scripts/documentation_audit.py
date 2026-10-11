#!/usr/bin/env python3
"""Create a review report when implementation changes without documentation updates."""
from pathlib import Path
import os
import subprocess


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


head = os.environ.get("GITHUB_SHA") or git("rev-parse", "HEAD")
try:
    parent = git("rev-parse", f"{head}^1")
except subprocess.CalledProcessError:
    print("No parent commit; skipping documentation audit for the initial commit.")
    raise SystemExit(0)

changed = [p for p in git("diff", "--name-only", parent, head).splitlines() if p]
implementation = [
    p for p in changed
    if p.startswith(("src/", "benches/", "tests/", "examples/"))
    or p in {"Cargo.toml", "Cargo.lock", "build.rs"}
    or (p.startswith("scripts/") and p.endswith(".py") and p != "scripts/documentation_audit.py")
]
documentation = [
    p for p in changed
    if p == "README.md" or p == "roadmap.md" or p == "puntos.md"
    or p.startswith(("docs/", "artifacts/README"))
]

report = Path("docs/reports/documentation-audit.md")
if not implementation or documentation:
    if report.exists():
        report.unlink()
    print("No documentation review PR needed: no implementation change, or documentation changed in the same commit.")
    raise SystemExit(0)

lines = [
    "# Documentation review requested",
    "",
    f"Implementation commit: `{head}`",
    "",
    "This report is a **review prompt**, not a claim that documentation is definitely missing. Review whether these implementation changes require updates to README.md, roadmap.md, puntos.md, or the relevant docs/ pages.",
    "",
    "## Implementation files changed",
    "",
]
lines.extend(f"- `{p}`" for p in implementation)
lines += [
    "",
    "## Review checklist",
    "",
    "- [ ] Does the public API or user-visible behavior change?",
    "- [ ] Should README.md or the repository map be updated?",
    "- [ ] Should a design/architecture page under docs/ be updated?",
    "- [ ] Does roadmap.md or puntos.md need a status or scope adjustment? (Do not mark points complete without evidence.)",
    "- [ ] If no documentation change is needed, explain why in the PR and close it.",
    "",
    "Generated automatically by the DINAH documentation audit workflow. This check is rule-based and asks for human review; it does not use an AI model or automatically invent documentation claims.",
    "",
]
report.parent.mkdir(parents=True, exist_ok=True)
report.write_text("\n".join(lines), encoding="utf-8")
print(f"Documentation review needed for {len(implementation)} implementation file(s).")
