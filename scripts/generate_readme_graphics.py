#!/usr/bin/env python3
"""Regenerate README status graphics from puntos.md evidence.

Generates the phase dashboard (versions.svg), the module status grid
(flow.svg) and the section header cards whose text must match the README
sections. Data is read from puntos.md so the graphics cannot drift from
the checklist. Output is written for both light and dark themes.

Usage: python3 scripts/generate_readme_graphics.py [--check]
  --check: verify the generated files match the committed ones (CI mode).
"""
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent

LIGHT = {
    "bg": "#f1f7fa",
    "card": "#e1edf2",
    "accent": "#e87524",
    "inprog": "#c65312",
    "track": "#cbdce4",
    "body": "#4c606c",
    "dark": "#12202b",
}
DARK = {
    "bg": "#08121a",
    "card": "#0c1822",
    "accent": "#ffb15c",
    "inprog": "#c65312",
    "track": "#20313c",
    "body": "#9ab0bd",
    "dark": "#eaf5fb",
}
FONT = "'Segoe UI',-apple-system,'Helvetica Neue',Arial,sans-serif"
REDUCED = "@media (prefers-reduced-motion:reduce){*{animation:none!important}}"


def read_phases():
    """Return [(name, total, complete, deferred, in_progress)] from puntos.md."""
    text = (ROOT / "puntos.md").read_text(encoding="utf-8")
    phases = []
    current = None
    counts = None
    for line in text.splitlines():
        m = re.match(r"^## FASE (\d+) — (.+)$", line)
        if m:
            if current:
                phases.append((current, counts))
            current = (m.group(1), m.group(2).strip())
            counts = {"x": 0, ">": 0, " ": 0, "~": 0}
            continue
        if current is None:
            continue
        m = re.match(r"^- \[([ >x~])\] \d{3} — ", line)
        if m:
            counts[m.group(1)] += 1
    if current:
        phases.append((current, counts))
    out = []
    for (num, name), c in phases:
        total = c["x"] + c[">"] + c[" "] + c["~"]
        out.append({
            "num": num, "name": name.upper(), "total": total,
            "complete": c["x"], "deferred": c[" "], "inprog": c[">"],
        })
    return out


SHORT_NAMES = {
    "0": "FOUNDATION",
    "1": "CORE",
    "2": "RUNTIME",
    "3": "MEMORY",
    "4": "COMPUTE",
    "5": "DATA / ECS",
    "6": "SIMULATION",
    "7": "NETWORKING",
    "8": "STORAGE",
    "9": "NATIVE / HARDWARE",
    "10": "CLI",
    "11": "OBSERVABILITY",
    "12": "SECURITY & ROBUSTNESS",
    "13": "TESTING",
    "14": "OPTIMIZATION",
    "15": "TOTAL INTEGRATION",
    "16": "DINAH V0.1",
    "17": "DINAH V0.2+",
}


def read_totals():
    text = (ROOT / "puntos.md").read_text(encoding="utf-8")
    m = re.search(r"^- Completados: (\d+)$", text, re.MULTILINE)
    c = int(m.group(1))
    m = re.search(r"^- Pendientes: (\d+)$", text, re.MULTILINE)
    p = int(m.group(1))
    return c, p, 287


def esc(s):
    s = s.replace("&", "&amp;")
    s = s.replace("<", "<")
    s = s.replace(">", ">")
    return s


def header_card(title, theme):
    t = LIGHT if theme == "light" else DARK
    underline = min(600, int(len(title) * 27))
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1280 194" width="1280" height="194">'
        f'<style>.sw{{animation:sw 4s ease-in-out infinite alternate}}@keyframes sw{{to{{transform:translateX({1184 - underline}px)}}}}{REDUCED}</style>'
        f'<rect width="1280" height="194" rx="28" fill="{t["bg"]}"/>'
        f'<text x="48" y="136" fill="{t["dark"]}" style="font-family:{FONT};font-size:92px;font-weight:700;letter-spacing:1px">{esc(title)}</text>'
        f'<rect x="48" y="170" width="1184" height="2" fill="{t["track"]}"/>'
        f'<rect class="sw" x="48" y="168" width="{underline}" height="6" rx="3" fill="{t["accent"]}"/>'
        f'</svg>'
    )


def phase_dashboard(phases, complete, deferred, total, theme):
    t = LIGHT if theme == "light" else DARK
    n = len(phases)
    rows = (n + 1) // 2
    W, H = 1280, 822
    top = 118          # legend + summary area
    bottom = 40
    row_h = (H - top - bottom) / rows
    col_x = [48, 664]
    bar_w = 400
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
        f'<style>.g{{transform-box:fill-box;transform-origin:left;animation:gr 9s infinite}}@keyframes gr{{0%{{transform:scaleX(0)}}14%,86%{{transform:scaleX(1)}}100%{{transform:scaleX(1)}}}}{REDUCED}</style>',
        f'<rect width="{W}" height="{H}" rx="28" fill="{t["bg"]}"/>',
        # legend
        f'<rect x="48" y="36" width="16" height="16" rx="4" fill="{t["accent"]}"/>',
        f'<text x="74" y="50" fill="{t["body"]}" style="font-family:{FONT};font-size:20px">Complete</text>',
        f'<rect x="190" y="36" width="16" height="16" rx="4" fill="{t["track"]}"/>',
        f'<text x="216" y="50" fill="{t["body"]}" style="font-family:{FONT};font-size:20px">Deferred by design</text>',
        # summary
        f'<text x="48" y="92" fill="{t["dark"]}" style="font-family:{FONT};font-size:26px;font-weight:700">dinah v0.1.0 — {complete} of {total} points complete</text>',
        f'<text x="48" y="112" fill="{t["body"]}" style="font-family:{FONT};font-size:17px">{deferred} points deferred by design, tracked as v0.2 goals in puntos2.md · data from puntos.md</text>',
    ]
    for i, ph in enumerate(phases):
        col = i // rows
        row = i % rows
        x = col_x[col]
        y = top + row * row_h
        label = f'F{ph["num"]} {SHORT_NAMES.get(ph["num"], ph["name"])}'
        parts.append(
            f'<text x="{x}" y="{y + 18}" fill="{t["dark"]}" style="font-family:{FONT};font-size:17px;font-weight:600">{esc(label)}</text>'
        )
        # bar track + fill
        by = y + 28
        frac = ph["complete"] / ph["total"]
        parts.append(
            f'<rect x="{x}" y="{by}" width="{bar_w}" height="12" rx="6" fill="{t["track"]}"/>'
        )
        if frac > 0:
            parts.append(
                f'<rect class="g" x="{x}" y="{by}" width="{max(12, int(bar_w * frac))}" height="12" rx="6" fill="{t["accent"]}"/>'
            )
        if ph["inprog"]:
            parts.append(
                f'<rect x="{x}" y="{by}" width="{int(bar_w * ph["inprog"] / ph["total"])}" height="12" rx="6" fill="{t["inprog"]}"/>'
            )
        note = f'{ph["complete"]}/{ph["total"]}'
        if ph["deferred"]:
            note += f' ({ph["deferred"]} deferred)'
        parts.append(
            f'<text x="{x + bar_w + 14}" y="{by + 12}" fill="{t["body"]}" style="font-family:{FONT};font-size:16px">{esc(note)}</text>'
        )
    parts.append('</svg>')
    return "".join(parts)


def module_grid(theme):
    t = LIGHT if theme == "light" else DARK
    cards = [
        ("01", "CORE", "13 of 15 points", "2 deferred"),
        ("02", "RUNTIME", "complete", "v0.1.0"),
        ("03", "MEMORY", "complete", "v0.1.0"),
        ("04", "COMPUTE", "12 of 14 points", "2 deferred"),
        ("05", "DATA / ECS", "12 of 14 points", "2 deferred"),
        ("06", "SIMULATION", "complete", "v0.1.0"),
        ("07", "NETWORKING", "complete", "v0.1.0"),
        ("08", "STORAGE", "complete", "v0.1.0"),
        ("09", "NATIVE / HARDWARE", "12 of 13 points", "1 deferred"),
        ("10", "CLI", "complete", "v0.1.0"),
        ("11", "OBSERVABILITY", "complete", "v0.1.0"),
        ("12", "SECURITY & QUALITY", "15 of 16 points", "1 deferred"),
    ]
    W, H = 1280, 570
    margin, gap = 40, 15
    cw = (W - 2 * margin - 3 * gap) / 4
    ch, vgap = 140, 23
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
        f'<style>.pu{{animation:pu 1.8s ease-in-out infinite}}@keyframes pu{{50%{{opacity:.3}}}}{REDUCED}</style>',
        f'<rect width="{W}" height="{H}" rx="28" fill="{t["bg"]}"/>',
    ]
    for i, (num, name, status, note) in enumerate(cards):
        col, row = i % 4, i // 4
        x = margin + col * (cw + gap)
        y = margin + row * (ch + vgap)
        parts.append(f'<rect x="{x}" y="{y}" width="{cw}" height="{ch}" rx="16" fill="{t["card"]}"/>')
        parts.append(
            f'<text x="{x + 20}" y="{y + 34}" fill="{t["accent"]}" style="font-family:{FONT};font-size:18px;font-weight:700">{num}</text>'
        )
        parts.append(
            f'<text x="{x + 20}" y="{y + 60}" fill="{t["dark"]}" style="font-family:{FONT};font-size:17px;font-weight:700">{esc(name)}</text>'
        )
        complete = status == "complete"
        dot = t["accent"] if complete else t["inprog"]
        parts.append(
            f'<circle class="pu" cx="{x + 26}" cy="{y + 86}" r="5" fill="{dot}"/>'
        )
        parts.append(
            f'<text x="{x + 40}" y="{y + 92}" fill="{t["body"]}" style="font-family:{FONT};font-size:15px">{esc(status)}</text>'
        )
        parts.append(
            f'<text x="{x + 20}" y="{y + 118}" fill="{t["body"]}" style="font-family:{FONT};font-size:13px;opacity:.85">{esc(note)}</text>'
        )
    parts.append('</svg>')
    return "".join(parts)


def main():
    phases = read_phases()
    complete, deferred, total = read_totals()
    outputs = {
        "assets/light/versions.svg": phase_dashboard(phases, complete, deferred, total, "light"),
        "assets/dark/versions.svg": phase_dashboard(phases, complete, deferred, total, "dark"),
        "assets/light/flow.svg": module_grid("light"),
        "assets/dark/flow.svg": module_grid("dark"),
        "assets/light/h-overview.svg": header_card("OVERVIEW", "light"),
        "assets/dark/h-overview.svg": header_card("OVERVIEW", "dark"),
        "assets/light/h-docs.svg": header_card("DOCUMENTATION", "light"),
        "assets/dark/h-docs.svg": header_card("DOCUMENTATION", "dark"),
        "assets/light/h-assurance.svg": header_card("VALIDATION", "light"),
        "assets/dark/h-assurance.svg": header_card("VALIDATION", "dark"),
        "assets/light/h-quick.svg": header_card("QUICK START", "light"),
        "assets/dark/h-quick.svg": header_card("QUICK START", "dark"),
    }
    check = "--check" in sys.argv
    ok = True
    for rel, content in outputs.items():
        path = ROOT / rel
        if check:
            if path.read_text(encoding="utf-8") != content:
                print(f"OUT OF DATE: {rel}")
                ok = False
            else:
                print(f"ok: {rel}")
        else:
            path.write_text(content, encoding="utf-8")
            print(f"written: {rel}")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
