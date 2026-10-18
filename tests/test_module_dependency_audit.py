"""Regression tests for the source-level module dependency audit."""
import tempfile
import unittest
from pathlib import Path

from scripts.check_module_cycles import dependency_graph, find_cycles


class FindCyclesTests(unittest.TestCase):
    def test_source_graph_ignores_comments_and_finds_cross_module_refs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "src"
            root.mkdir()
            (root / "lib.rs").write_text("pub mod a;\npub mod b;\n", encoding="utf-8")
            (root / "a.rs").write_text("use crate::b::Thing;\n", encoding="utf-8")
            (root / "b.rs").write_text("// crate::a::Fake\npub struct Thing;\n", encoding="utf-8")
            graph = dependency_graph(root)
        self.assertEqual(graph, {"a": {"b"}, "b": set()})

    def test_acyclic_graph_has_no_cycles(self):
        graph = {"core": set(), "runtime": {"core"}, "compute": {"runtime"}}
        self.assertEqual(find_cycles(graph), [])

    def test_reports_a_cycle(self):
        graph = {"runtime": {"compute"}, "compute": {"ecs"}, "ecs": {"runtime"}}
        self.assertEqual(find_cycles(graph), [["compute", "ecs", "runtime", "compute"]])

    def test_ignores_unreachable_nodes_after_cycle_scan(self):
        graph = {"a": {"b"}, "b": {"a"}, "isolated": set()}
        self.assertEqual(find_cycles(graph), [["a", "b", "a"]])


if __name__ == "__main__":
    unittest.main()
