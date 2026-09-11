"""Material channels retain independent meanings when transferred to body UVs."""
import sys
from pathlib import Path
import tempfile
import unittest

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from bake_mail_weave import STEEL_BASE_COLOR_SRGB
from bake_underlayer_materials import Weave, bake


class UnderlayerMaterialTests(unittest.TestCase):
    def test_ambient_visibility_and_normals_cannot_shade_base_color(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            # Deliberately varying source RGB must not become albedo shading.
            color = np.zeros((4, 4, 4), dtype=np.uint8)
            color[:, :, :3] = np.arange(4)[None, :, None] * 70
            color[:, :, 3] = [0, 100, 200, 255]
            normal = np.full((4, 4, 4), [200, 100, 225, 255], dtype=np.uint8)
            occlusion = np.full((4, 4, 4), 255, dtype=np.uint8)
            occlusion[:, :, :3] = np.arange(4)[None, :, None] * 80
            for name, values in [("color", color), ("normal", normal),
                                 ("occlusion", occlusion)]:
                Image.fromarray(values).save(directory / f"{name}.png")
            weave = Weave(directory)
            positions = np.array([[0, 0, 0], [.02, 0, 0], [0, .02, 0]])
            body = {"triangles": [{
                "positions": positions.tolist(),
                "normals": [[0, 0, 1]] * 3,
                "uv": [[0, 0], [1, 0], [0, 1]],
                "chart": positions[:, :2].tolist(),
            }]}
            color, normal, occlusion = bake(body, 32, weave)
            visible = color[:, :, 3] > 128
            self.assertTrue(visible.any())
            self.assertTrue((color[visible, :3] == int(255 * STEEL_BASE_COLOR_SRGB)).all())
            self.assertTrue((color[:, :, :3] == int(255 * STEEL_BASE_COLOR_SRGB)).all())
            self.assertGreater(np.ptp(occlusion[visible]), 20)
            decoded = normal[visible].astype(float) / 255 * 2 - 1
            self.assertTrue((decoded[:, 0] > .3).all())
            self.assertTrue((decoded[:, 1] < -.1).all())
            self.assertTrue(np.allclose(np.linalg.norm(decoded, axis=1), 1, atol=.015))

    def test_tangent_normal_follows_rotated_uv_axes(self):
        class ConstantWeave:
            def sample(self, chart):
                count = len(chart)
                return (np.full((count, 4), 1.),
                        np.tile([-.5, 0], (count, 1)), np.ones(count))

        triangle = {
            "positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0]],
            "normals": [[0, 0, 1]] * 3,
            "chart": [[0, 0], [1, 0], [0, 1]],
            "uv": [[0, 0], [0, 1], [1, 0]],
        }
        color, normal, _ = bake({"triangles": [triangle]}, 16, ConstantWeave())
        decoded = normal[color[:, :, 3] > 128].astype(float) / 255 * 2 - 1
        # World +X points along tangent +Y when the UV axes are exchanged.
        self.assertTrue((np.abs(decoded[:, 0]) < .01).all())
        self.assertTrue((decoded[:, 1] > .4).all())
        self.assertTrue((decoded[:, 2] > .85).all())


if __name__ == "__main__":
    unittest.main()
