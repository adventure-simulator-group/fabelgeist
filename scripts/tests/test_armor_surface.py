"""Run with Blender's Python to exercise the actual morph clearance reader."""
import sys
import tempfile
from pathlib import Path
import unittest
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from armor_glb import Asset
from check_parametric_armor_assets import EXPECTED_TARGETS
try:
    from check_underlayer_assets import Surface
except ImportError:
    Surface = None


@unittest.skipIf(Surface is None, "requires Blender mathutils")
class ArmorSurfaceTests(unittest.TestCase):
    def test_components_rebase_indices_and_keep_their_own_morphs_and_skin(self):
        asset = Asset.__new__(Asset)
        asset.doc = {"asset": {"version": "2.0"}, "buffers": [{"byteLength": 0}],
                     "bufferViews": [], "accessors": [], "meshes": []}
        asset.binary = bytearray()
        for index, name in enumerate(["plate", "leather_straps"]):
            positions = np.array([[0, 0, 0], [1, 0, 0], [0, 1, 0]]) + index * 3
            attributes = {"POSITION": asset.append(positions),
                "JOINTS_0": asset.append([[index, 0, 0, 0]] * 3, component=5123),
                "WEIGHTS_0": asset.append([[1, 0, 0, 0]] * 3)}
            delta = asset.append([[0, 0, index + .1]] * 3)
            primitive = {"attributes": attributes,
                "indices": asset.append([0, 1, 2], component=5125),
                "targets": [{"POSITION": delta} for _ in EXPECTED_TARGETS]}
            asset.doc["meshes"].append({"name": name, "primitives": [primitive],
                "extras": {"targetNames": EXPECTED_TARGETS}})
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "assembly.glb"
            asset.write(path)
            whole = Surface(path)
            selected = Surface(path, {"leather_straps"})
            np.testing.assert_array_equal(whole.faces, [[0, 1, 2], [3, 4, 5]])
            np.testing.assert_array_equal(selected.faces, [[0, 1, 2]])
            np.testing.assert_array_equal(whole.positions[3:], selected.positions)
            np.testing.assert_array_equal(whole.targets[:, 3:], selected.targets)
            np.testing.assert_array_equal(selected.joints[:, 0], [1, 1, 1])
            self.assertFalse(np.array_equal(whole.targets[:, :3], selected.targets))
            with self.assertRaisesRegex(AssertionError, "No matching"):
                Surface(path, {"missing"})


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0]])
