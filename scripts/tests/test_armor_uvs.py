"""UV splits must not alter rig, anatomy, or morph correspondence."""
import copy
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from armor_glb import Asset
from check_armor_uvs import audit, overlaps


class ArmorUvsTests(unittest.TestCase):
    def test_untouched_plate_cannot_pass_atlas_audit(self):
        asset = Asset.__new__(Asset)
        asset.doc = {"asset": {"version": "2.0"}, "buffers": [{"byteLength": 0}],
                     "bufferViews": [], "accessors": [], "materials": [{}]}
        asset.binary = bytearray()
        position = asset.append([[0, 0, 0], [1, 0, 0], [0, 1, 0]])
        indices = asset.append([0, 1, 2], component=5125)
        asset.doc["meshes"] = [{"name": "plate", "primitives": [{
            "attributes": {"POSITION": position}, "indices": indices, "material": 0}]}]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "breastplate--worn.glb"
            asset.write(path)
            with self.assertRaisesRegex(AssertionError, "missing material atlas"):
                audit(path, path)

    def test_seam_split_roundtrip_preserves_all_attributes_and_morphs(self):
        asset = Asset.__new__(Asset)
        asset.doc = {"asset": {"version": "2.0"}, "buffers": [{"byteLength": 0}],
                     "bufferViews": [], "accessors": []}
        asset.binary = bytearray()
        attributes = {
            "POSITION": asset.append([[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]]),
            "TEXCOORD_0": asset.append([[.1, .2], [.3, .4], [.5, .6], [.7, .8]]),
            "JOINTS_0": asset.append([[1, 2, 3, 4]] * 4, component=5123),
            "WEIGHTS_0": asset.append([[.4, .3, .2, .1]] * 4),
        }
        delta = asset.append([[0, 0, .1], [0, .2, 0], [.3, 0, 0], [1, 2, 3]])
        primitive = {"attributes": attributes, "indices": asset.append([0, 1, 2, 1, 3, 2], component=5125),
                     "targets": [{"POSITION": delta}]}
        asset.doc["meshes"] = [{"name": "visor", "primitives": [primitive],
                                "extras": {"targetNames": ["shape"]}}]
        asset.doc["nodes"] = [{"mesh": 0, "extras": {"hinge": {"axis": [1, 0, 0]}}}]
        source = np.array([0, 1, 2, 1, 3, 2])
        expected = {name: asset.array(index)[source] for name, index in attributes.items()}
        expected_delta = asset.array(delta)[source]
        nodes = copy.deepcopy(asset.doc["nodes"])
        asset.remap(primitive, source, np.arange(6))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "split.glb"
            asset.write(path)
            result = Asset(path)
        after = result.doc["meshes"][0]["primitives"][0]
        for name, values in expected.items():
            np.testing.assert_array_equal(values, result.array(after["attributes"][name]))
        np.testing.assert_array_equal(expected_delta, result.array(after["targets"][0]["POSITION"]))
        self.assertEqual(nodes, result.doc["nodes"])
        self.assertEqual(len(result.doc["accessors"]), 6)

    def test_overlap_predicate_distinguishes_shared_edges_and_nested_triangles(self):
        triangle = [[0, 0], [1, 0], [0, 1]]
        shared = [[1, 0], [1, 1], [0, 1]]
        nested = [[.1, .1], [.2, .1], [.1, .2]]
        self.assertEqual(overlaps(np.array([triangle, shared])), 0)
        self.assertEqual(overlaps(np.array([triangle, nested])), 1)
        self.assertEqual(overlaps(np.array([triangle, triangle])), 1)


if __name__ == "__main__":
    unittest.main()
