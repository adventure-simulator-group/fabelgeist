"""Physical rim coordinates must not depend on material chart splits."""
from pathlib import Path
import sys
import unittest
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from armor_trim_math import boundary_field, pattern_mask
from trim_armor import validate


class TrimTests(unittest.TestCase):
    def test_uv_splits_do_not_create_rims_or_reset_pattern_phase(self):
        points = np.array([[0,0,0],[1,0,0],[1,1,0],[0,1,0]], float)
        faces = np.array([[0,1,2],[0,2,3]])
        segments = points[[[0,1],[1,2],[2,3],[3,0]]]
        welded = boundary_field(points, faces, segments)
        split = boundary_field(points[faces.flatten()], np.arange(6).reshape(-1,3), segments)
        for a,b in zip(welded, split): np.testing.assert_array_equal(a,b)
        self.assertAlmostEqual(welded[2].sum(),1)

    def test_json_fractional_coordinates_match_float32_vertex_stream(self):
        points = np.array([[.1920205, 1.223645, -.02105], [.2, 1.223645, -.02105], [.2, 1.3, -.02105]], np.float32)
        # serde_json emits decimal coordinates; parsing them uses double precision.
        segments = [[[float(format(v, '.8g')) for v in point] for point in points[edge]]
                    for edge in [[0,1],[1,2],[2,0]]]
        result = boundary_field(points, np.array([[0,1,2]]), segments)
        self.assertEqual(len(result[0]), 3)

    def test_patterns_repeat_and_do_not_paint_outside_band(self):
        distance = np.linspace(-.01,.02,400)
        phase = np.linspace(0,3,400)
        for style in ['plain','double','chevron','scallop','vine']:
            mask = pattern_mask(distance,phase,.01,style)
            self.assertTrue(mask.any())
            self.assertFalse(mask[(distance < 0) | (distance > .01)].any())
            np.testing.assert_array_equal(mask,pattern_mask(distance,phase+2,.01,style))

    def test_invalid_finish_controls_fail(self):
        recipe = dict(pattern='plain', color='#C7A45B', width_mm=6, repeats=32, metallic=1, roughness=.3)
        for key,value in [('width_mm',0),('repeats',1.5),('pattern','engrave_geometry'),('color','gold'),('roughness',2)]:
            with self.assertRaises(ValueError): validate({**recipe,key:value})


if __name__ == '__main__': unittest.main()
