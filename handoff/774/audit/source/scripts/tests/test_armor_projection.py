"""Dense-to-native projection must ignore physical interior/return geometry."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
try:
    import bpy
except ImportError:
    bpy = None


@unittest.skipIf(bpy is None, "run with Blender's Python")
class ProjectionTests(unittest.TestCase):
    def test_outer_source_wins_over_a_closer_construction_face(self):
        from armor_bake_projection import project
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete(use_global=False)
        scene = bpy.context.scene
        scene.render.engine = "CYCLES"
        scene.cycles.samples = 1
        mesh = bpy.data.meshes.new("target")
        mesh.from_pydata([[0, 0, 0], [1, 0, 0], [0, 1, 0]], [], [[0, 1, 2]])
        mesh.update()
        layer = mesh.uv_layers.new(name="material")
        for loop, uv in zip(mesh.loops, [[0, 0], [1, 0], [0, 1]]):
            layer.data[loop.index].uv = uv
        target = bpy.data.objects.new("plate", mesh)
        bpy.context.collection.objects.link(target)
        material = bpy.data.materials.new("target")
        material.use_nodes = True
        node = material.node_tree.nodes.new("ShaderNodeTexImage")
        material.node_tree.nodes.active = node
        mesh.materials.append(material)
        image = bpy.data.images.new("projected", 64, 64, alpha=False)
        image.colorspace_settings.name = "Non-Color"
        direction = np.array([.4, 0, np.sqrt(1 - .4 ** 2)])
        source = {"positions": [[0, 0, .01], [1, 0, .01], [0, 1, .01],
                                [0, 0, .02], [1, 0, .02], [0, 1, .02]],
                  "normals": [direction.tolist()] * 3 + [(-direction).tolist()] * 3,
                  "indices": [0, 1, 2, 3, 4, 5], "components": [],
                  "construction_faces": [{"start": 3, "end": 6}]}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "dense.json"
            path.write_text(json.dumps(source), encoding="utf-8")
            pixels, evidence = project(path, "plate", target, image)
        np.testing.assert_allclose(pixels[47, 16, :3] * 2 - 1, direction, atol=.015)
        self.assertEqual(evidence["source_triangles"], 1)
        self.assertFalse(scene.render.bake.use_selected_to_active)


if __name__ == "__main__":
    unittest.main(argv=[__file__])
