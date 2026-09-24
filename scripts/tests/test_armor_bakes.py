"""Normal bakes must encode relief without modifying unlit material color."""
from pathlib import Path
import sys
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from armor_bake_math import carrier_normals, frame_tangents, normal_atlas, unit


class ArmorBakeTests(unittest.TestCase):
    def test_curved_face_inverts_the_raw_bevy_fragment_frame(self):
        uv, faces = np.array([[0., 0.], [1., 0.], [0., 1.]]), np.array([[0, 1, 2]])
        carrier = unit(np.array([[.7, 0., .7], [0., .7, .7], [-.7, 0., .7]]))
        tangents = frame_tangents(carrier, np.tile([1., 0., 0., 1.], (3, 1)))
        detailed = unit(np.tile([.2, .3, 1.], (3, 1)))
        image, _ = normal_atlas(uv, faces, detailed, carrier, tangents, 32)
        x, y = 8, 10
        weights = np.array([1 - (x + .5) / 32 - (y + .5) / 32, (x + .5) / 32, (y + .5) / 32])
        n, t = weights @ carrier, weights @ tangents[:, :3]
        b = (weights @ tangents[:, 3]) * np.cross(n, t)
        # Exact Bevy fragment reconstruction, without Gram-Schmidt or normalizing
        # the interpolated columns before applying the sampled normal map.
        recovered = unit(np.column_stack((t, b, n)) @ (image[y, x, :3] * 2 - 1))
        np.testing.assert_allclose(recovered, detailed[0], atol=1e-6)

    def test_tangent_map_reconstructs_detail_and_handedness(self):
        uv = np.array([[0, 0], [1, 0], [0, 1], [1, 1]])
        faces = np.array([[0, 1, 2], [1, 3, 2]])
        detail = unit(np.tile([.4, .2, 1.0], (4, 1)))
        normal = np.tile([0., 0., 1.], (4, 1))
        for sign in (-1., 1.):
            tangent = np.tile([1., 0., 0., sign], (4, 1))
            image, covered = normal_atlas(uv, faces, detail, normal, tangent, 32)
            self.assertTrue(covered.all())
            recovered = (image[:, :, :3] * 2 - 1) * [1, sign, 1]
            np.testing.assert_allclose(recovered, np.broadcast_to(detail[0], recovered.shape), atol=1e-6)

    def test_smoothing_joins_uv_seams_but_keeps_opposite_plate_surfaces(self):
        positions = np.array([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 0.],
                              [0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])
        normals = unit(np.array([[.2, 0, 1], [-.2, 0, 1], [0, 0, 1], [.2, 0, 1],
                                  [0, 0, -1], [0, 0, -1], [0, 0, -1]]))
        fields = normals[:, None, :]
        result = carrier_normals(positions, np.array([[0, 1, 2], [3, 1, 2], [4, 6, 5]]), fields)
        np.testing.assert_array_equal(result[0], result[3])
        self.assertLess(abs(result[0, 0, 0]), .01)
        np.testing.assert_array_equal(result[4:, 0], normals[4:])

    def test_carrier_tangent_is_orthogonal_to_new_normal(self):
        normals = unit(np.array([[.3, .4, 1.]]))
        tangent = frame_tangents(normals, np.array([[1., 0., 0., -1.]]))
        self.assertAlmostEqual(float((normals * tangent[:, :3]).sum()), 0)
        self.assertAlmostEqual(float(np.linalg.norm(tangent[0, :3])), 1)
        self.assertEqual(tangent[0, 3], -1)


if __name__ == "__main__":
    unittest.main()
