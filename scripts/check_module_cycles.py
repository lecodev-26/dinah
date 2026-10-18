#!/usr/bin/env python3
"""Check the crate-root source-reference graph for module dependency cycles.

This intentionally checks source-level crate::<root_module> references. It is
an architectural guardrail, not a Rust parser and not proof that runtime or
conceptual dependencies are acyclic.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

MODULE_DECL = re.compile(r"^\s*pub\s+mod\s+([A-Za-z_]\w*)\s*;", re.MULTILINE)
ROOT_REF = re.compile(r"\bcrate::([A-Za-z_]\w*)\b")
BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.DOTALL)
LINE_COMMENT = re.compile(r"//.*?$", re.MULTILINE)


def dependency_graph(source_root: Path) -> dict[str, set[str]]:
    """Build a graph between root modules from crate-qualified source refs."""
    lib_rs = source_root / "lib.rs"
    modules = set(MODULE_DECL.findall(lib_rs.read_text(encoding="utf-8")))
    graph = {name: set() for name in sorted(modules)}

    for path in sorted(source_root.rglob("*.rs")):
        relative = path.relative_to(source_root)
        if not relative.parts:
            continue
        owner = path.stem if len(relative.parts) == 1 else relative.parts[0]
        if owner not in modules:
            continue
        source = path.read_text(encoding="utf-8")
        source = BLOCK_COMMENT.sub(" ", source)
        source = LINE_COMMENT.sub(" ", source)
        for target in ROOT_REF.findall(source):
            if target in modules and target != owner:
                graph[owner].add(target)
    return graph


def find_cycles(graph: dict[str, set[str]]) -> list[list[str]]:
    """Return each detected directed cycle as a closed path."""
    visited: set[str] = set()
    active: list[str] = []
    active_set: set[str] = set()
    cycles: list[list[str]] = []
    seen: set[tuple[str, ...]] = set()

    def visit(node: str) -> None:
        if node in visited:
            return
        visited.add(node)
        active.append(node)
        active_set.add(node)
        for neighbor in sorted(graph.get(node, set())):
            if neighbor in active_set:
                start = active.index(neighbor)
                body = active[start:]
                rotations = [tuple(body[i:] + body[:i]) for i in range(len(body))]
                canonical = min(rotations)
                if canonical not in seen:
                    seen.add(canonical)
                    cycles.append(list(canonical) + [canonical[0]])
            elif neighbor not in visited:
                visit(neighbor)
        active.pop()
        active_set.remove(node)

    for node in sorted(graph):
        visit(node)
    return cycles


def main() -> int:
    graph = dependency_graph(Path("src"))
    cycles = find_cycles(graph)
    edge_count = sum(len(targets) for targets in graph.values())
    print(f"Module dependency audit: {len(graph)} root modules, {edge_count} cross-module source-reference edges.")
    if cycles:
        print("Dependency cycle(s) detected:")
        for cycle in cycles:
            print("  " + " -> ".join(cycle))
        return 1
    print("No cycles detected in the crate-root source-reference graph.")
    print("Scope: source references only; this does not prove runtime/conceptual acyclicity.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
