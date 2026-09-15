"""Run with Blender's Python to exercise the actual morph clearance reader."""
import sys
import tempfile
from pathlib import Path
import unittest
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from armor_glb import Asset
from check_parametric_armor_assets import EXPECTED_TARGETS, SKELETAL_FIT_TARGETS
try:
    from check_underlayer_assets import Surface, skeletal_fit_weights
except ImportError:
    Surface = None


@unittest.skipIf(Surface is None, "requires Blender mathutils")
class ArmorSurfaceTests(unittest.TestCase):
    def test_helmet_sweep_vectors_apply_to_the_complete_export_catalog(self):
        from check_close_helmet_assets import configurations
        deltas = np.zeros((len(EXPECTED_TARGETS), 3, 3))
        samples = list(configurations())
        for name, weights, _ in samples:
            with self.subTest(sample=name):
                result = np.einsum("i,ijk->jk", weights, deltas)
                self.assertEqual(result.shape, (3, 3))
                self.assertTrue(np.isfinite(weights).all())
        for index in range(45, len(EXPECTED_TARGETS)):
            self.assertTrue(any(weights[index] == 1 and np.any(weights[:45])
                                for _, weights, _ in samples))

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

    def test_residuals_and_bone_translations_recover_direct_fits(self):
        # A rigid plate changes dimensions with its wearer, while its joint also
        # translates. Residuals must supply only the part skinning does not.
        references = [np.array([.2, 0, -.3, 0, .1, .6, .4, -.1, 0]),
                      np.array([-.5, 0, 1, 0, -.5, -1, 1.1, -.4, 0])]
        positions = np.array([[0, 0, 0], [.1, 0, 0], [0, .2, 0]])
        translation_basis = np.zeros((9, 3))
        translation_basis[[0, 2, 4, 5, 6, 7]] = [
            [.04, 0, 0], [.03, -.07, 0], [0, .08, .01],
            [0, -.09, .02], [0, .1, 0], [0, .02, .01]]
        shape_basis = np.zeros((9, 3, 3))
        for index in [0, 2, 4, 5, 6, 7]:
            shape_basis[index] = positions * (index + 1) * .1
        for reference in references:
            asset = Asset.__new__(Asset)
            asset.doc = {"asset": {"version": "2.0"},
                         "buffers": [{"byteLength": 0}],
                         "bufferViews": [], "accessors": []}
            asset.binary = bytearray()
            attributes = {"POSITION": asset.append(positions),
                "JOINTS_0": asset.append([[0, 0, 0, 0]] * 3, component=5123),
                "WEIGHTS_0": asset.append([[1, 0, 0, 0]] * 3)}
            zero = asset.append(np.zeros_like(positions))
            targets = [{"POSITION": zero} for _ in range(45)]
            for _, index, endpoint in SKELETAL_FIT_TARGETS:
                interval = endpoint - reference[index]
                residual = interval * (shape_basis[index] - translation_basis[index])
                targets.append({"POSITION": asset.append(residual)})
            asset.doc["meshes"] = [{"extras": {"targetNames": EXPECTED_TARGETS},
                "primitives": [{"attributes": attributes, "targets": targets,
                    "indices": asset.append([0, 1, 2], component=5125)}]}]
            asset.doc["nodes"] = [{"name": "plate_anchor", "extras": {
                "adventuresim_proportions": {"reference": reference.tolist(),
                    "translation_metres": translation_basis.tolist()}}}]
            asset.doc["skins"] = [{"joints": [0], "inverseBindMatrices":
                asset.append(np.eye(4).reshape(1, 16), kind="MAT4")}]
            with tempfile.TemporaryDirectory() as temporary:
                path = Path(temporary) / "residual.glb"
                asset.write(path)
                surface = Surface(path)
                shapes = [reference.copy(), np.zeros(9)]
                for _, index, endpoint in SKELETAL_FIT_TARGETS:
                    for fraction in [.25, .5, 1]:
                        shape = reference.copy()
                        shape[index] += (endpoint - reference[index]) * fraction
                        shapes.append(shape)
                for shape in shapes:
                    with self.subTest(reference=reference.tolist(), shape=shape.tolist()):
                        weights = skeletal_fit_weights(shape, reference)
                        actual = surface.deform(weights, shape, {})
                        direct = positions + np.einsum(
                            "i,ijk->jk", shape - reference, shape_basis)
                        np.testing.assert_allclose(actual, direct, atol=5e-8)


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0]])
