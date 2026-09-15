"""Run in Blender's Python, where the audit's evaluated-mesh API is available."""
import sys
from pathlib import Path
import unittest
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_armor_isolation import Surface, audit, islands, separation, load_surfaces


class IsolationTests(unittest.TestCase):
    def surface(self, z):
        return Surface([[-1, -1, z], [1, -1, z], [0, 1, z]], [[0, 1, 2]])

    def test_distance_threshold_and_assembly_report(self):
        for gap in [0.005, 0.024, 0.026, 0.08]:
            rows = audit([("plate", self.surface(0)), ("disc", self.surface(gap))], .025)
            self.assertEqual([r["isolated"] for r in rows], [gap > .025] * 2)
            self.assertAlmostEqual(rows[0]["gap_mm"], gap * 1000, places=4)

    def test_edge_interiors_and_triangle_piercing(self):
        horizontal = Surface([[-1, -.01, 0], [1, -.01, 0], [0, .01, 0]], [[0, 1, 2]])
        vertical = Surface([[-.01, -1, .03], [-.01, 1, .03], [.01, 0, .03]], [[0, 1, 2]])
        self.assertAlmostEqual(separation(horizontal, vertical, .025), .03, places=5)
        piercing = Surface([[0, 0, -1], [0, 0, 1], [0, .02, 0]], [[0, 1, 2]])
        self.assertLess(separation(horizontal, piercing, .001), 1e-6)

    def test_material_seams_weld_but_detached_islands_remain(self):
        points = np.asarray([[0, 0, 0], [1, 0, 0], [0, 1, 0],
                             [0, 0, 0], [0, 1, 0], [-1, 0, 0],
                             [4, 0, 0], [5, 0, 0], [4, 1, 0]], dtype=float)
        faces = np.arange(9).reshape(-1, 3)
        self.assertEqual(len(list(islands(points, faces))), 2)

    def test_shipped_museum_assemblies_have_no_isolated_meshes(self):
        from types import SimpleNamespace
        root = Path(__file__).resolve().parents[2]
        for suit in ["henry", "nuremberg"]:
            with self.subTest(suit=suit):
                preset = SimpleNamespace(pose={}, items=[SimpleNamespace(
                    name=suit, path=root / "assets/art-demo/armor" / (suit + ".glb"))])
                rows = audit(load_surfaces(preset), .025)
                failures = [r for r in rows if r["isolated"]]
                self.assertFalse(failures, failures)

    def test_single_island_has_no_support(self):
        row, = audit([("floating", self.surface(0))], .025)
        self.assertTrue(row["isolated"])
        self.assertIsNone(row["neighbor"])


if __name__ == "__main__":
    unittest.main(argv=[__file__])
