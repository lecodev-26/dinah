#!/usr/bin/env python3
"""Validate stable checklist IDs and status summary."""
from pathlib import Path
import re
import sys

path = Path("puntos.md")
text = path.read_text(encoding="utf-8")
ids = [int(m.group(1)) for line in text.splitlines()
       if (m := re.match(r"^- \[[ >x~]\] (\d{3}) — ", line))]
expected = list(range(1, 288))
if ids != expected:
    print(f"Checklist IDs invalid: found {len(ids)} IDs; expected 287 sequential IDs 001–287.")
    missing = sorted(set(expected) - set(ids))
    duplicates = sorted({x for x in ids if ids.count(x) > 1})
    if missing:
        print("Missing:", ", ".join(f"{x:03}" for x in missing))
    if duplicates:
        print("Duplicates:", ", ".join(f"{x:03}" for x in duplicates))
    sys.exit(1)
if not re.search(r"Total de puntos numerados: 287", text):
    print("Declared total does not match the initial checklist.")
    sys.exit(1)
# Ensure the initial status summary agrees with the checklist markers.
counts = {"x": 0, ">": 0, " ": 0, "~": 0}
for line in text.splitlines():
    m = re.match(r"^- \[([ >x~])\] (\d{3}) — ", line)
    if m:
        counts[m.group(1)] += 1
summary = {
    "Completados": counts["x"],
    "En progreso": counts[">"],
    "Pendientes": counts[" "],
}
for label, actual in summary.items():
    match = re.search(rf"^- {re.escape(label)}: (\d+)$", text, re.MULTILINE)
    if not match or int(match.group(1)) != actual:
        print(f"Checklist summary mismatch for {label}: expected {actual}.")
        sys.exit(1)
print(f"Checklist OK: 287 stable IDs; {counts['x']} complete, {counts['>']} in progress, {counts[' ']} pending, {counts['~']} obsolete.")
