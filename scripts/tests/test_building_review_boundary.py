"""Keep authored building reviews on the production presentation boundary."""
import json
from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
VIEWER = ROOT / "crates/adventuresim-tactical-client/src/tactical_scene_viewer"


class BuildingReviewBoundaryTests(unittest.TestCase):
    def test_review_inputs_reference_existing_buildings_and_unique_views(self):
        for name in ("shop-sign-review", "workplace-review"):
            scene = json.loads((ROOT / f"assets/tactical-scenes/{name}.json").read_text())
            review = json.loads((ROOT / f"assets/tactical-scenes/{name}.review.json").read_text(encoding="utf-8"))
            ids = {building["id"] for building in scene["buildings"]}
            self.assertTrue({int(key) for key in review["signs"]} <= ids)
            self.assertTrue({view["building"] for view in review["views"]} <= ids)
            slugs = [view["slug"] for view in review["views"]]
            self.assertEqual(len(slugs), len(set(slugs)))
            self.assertTrue(all(sum(v * v for v in view["offset"]) > 0 for view in review["views"]))

    def test_review_adapters_cannot_construct_a_second_render_pipeline(self):
        paths = [VIEWER / "buildings.rs", VIEWER / "building_review.rs", *sorted((VIEWER / "building_review").glob("*.rs"))]
        forbidden = r"StandardMaterial\s*\{|Mesh::new|Mesh::from|Mesh::try_from|meshes\.add|materials\.add|compile_building_detail|compile_static_building_detail|compile_building_lod|DefaultPlugins"
        for path in paths:
            with self.subTest(path=path.name):
                source = re.sub(r"//[^\n]*", "", path.read_text())
                self.assertIsNone(re.search(forbidden, source), "review may supply state or observe assets; production owns rendering")

    def test_retired_review_renderers_are_not_build_targets(self):
        cargo = tomllib.loads((ROOT / "crates/adventuresim-building-generator/Cargo.toml").read_text())
        names = {binary["name"] for binary in cargo["bin"]}
        self.assertTrue({"shop-sign-viewer", "workplace-viewer"}.isdisjoint(names))
        for name in ("shop-sign-viewer", "workplace-viewer"):
            self.assertFalse((ROOT / f"crates/adventuresim-building-generator/src/bin/{name}.rs").exists())
        source = (ROOT / "scripts/capture_building_review.py").read_text(encoding="utf-8")
        self.assertIn('"tactical-scene-viewer"', source)
        self.assertNotIn('"adventuresim-building-generator"', source)


if __name__ == "__main__":
    unittest.main()
