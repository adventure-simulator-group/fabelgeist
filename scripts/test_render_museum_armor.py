"""Preset validation runs with Python; pose checks additionally run in Blender.

python -m unittest discover -s scripts -p test_render_museum_armor.py
blender --background --python-exit-code 1 --python scripts/test_render_museum_armor.py
"""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from render_museum_armor import Camera, Preset, apply_pose, game_vector, import_glb

try:
    import bpy
except ImportError:
    bpy = None


class PresetTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "preset.json"
        (self.path.parent / "body.json").write_text("{}")
        self.row = {"version": 1, "camera": {"position": [0, 1, 3],
            "look_at": [0, 1, 0], "orthographic_scale": 2},
            "items": [{"name": "body", "path": "body.json"}]}

    def load(self, row):
        self.path.write_text(json.dumps(row))
        return Preset.load(self.path)

    def test_paths_are_relative_to_preset_and_colors_are_explicit(self):
        self.row["items"][0]["components"] = {"plate": {"base_color": [.4, .3, .2, 1]}}
        result = self.load(self.row)
        self.assertEqual(result.items[0].path, self.path.parent / "body.json")
        self.assertIsNone(result.items[0].components["plate"].roughness)
        self.assertEqual(result.camera.orthographic_scale, 2)

    def test_camera_projection_is_explicit_and_unambiguous(self):
        camera = self.row["camera"]
        perspective = {"position": camera["position"], "look_at": camera["look_at"],
                       "focal_length_mm": 105}
        result = Camera.parse(perspective)
        self.assertIsNone(result.orthographic_scale)
        self.assertEqual(result.focal_length_mm, 105)
        for invalid in [perspective | {"orthographic_scale": 2},
                        perspective | {"focal_length_mm": True},
                        {key: value for key, value in perspective.items() if key != "focal_length_mm"}]:
            with self.subTest(camera=invalid), self.assertRaises(ValueError):
                Camera.parse(invalid)

    def test_rejects_ambiguous_or_invalid_render_controls(self):
        changes = [{"version": 1.0}, {"pose": {"root": {"axis": [0, 0, 0], "degrees": 20}}},
                   {"samples": True}, {"resolution": [360, 0]}, {"background": [-1, .2, .2]},
                   {"unknown": 1}, {"skeleton": 123}]
        for change in changes:
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.load(self.row | change)
        for change in [{"position": [0, 1, 0]}, {"orthographic_scale": float("nan")}, {"lookat": [0, 1, 0]}]:
            row = copy.deepcopy(self.row)
            row["camera"].update(change)
            with self.subTest(camera=change), self.assertRaises(ValueError):
                self.load(row)

    def test_generated_assets_can_live_separately_from_the_saved_preset(self):
        assets = self.path.parent / "generated"
        assets.mkdir()
        (assets / "body.json").write_text("{}")
        (assets / "skeleton.glb").write_bytes(b"fixture")
        self.row["skeleton"] = "skeleton.glb"
        self.path.write_text(json.dumps(self.row))
        result = Preset.load(self.path, assets)
        self.assertEqual(result.items[0].path, assets / "body.json")
        self.assertEqual(result.skeleton, assets / "skeleton.glb")
        with self.assertRaises(ValueError):
            Preset.load(self.path)

    def test_rejects_missing_assets_duplicate_items_and_unknown_material_fields(self):
        for items in [[{"name": "body", "path": "missing.glb"}], self.row["items"] * 2,
                      [{"name": "body", "path": "body.json", "material": {"gold": True}}]]:
            with self.subTest(items=items), self.assertRaises(ValueError):
                self.load(self.row | {"items": items})


@unittest.skipIf(bpy is None, "Requires Blender")
class ImportTests(unittest.TestCase):
    def test_import_uses_authored_morph_default_and_keeps_external_texture(self):
        from armor_glb import Asset
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete(use_global=False)
        mesh = bpy.data.meshes.new("animated plate")
        mesh.from_pydata([(0, 0, 0), (1, 0, 0), (0, 1, 0)], [], [(0, 1, 2)])
        obj = bpy.data.objects.new("animated plate", mesh)
        bpy.context.collection.objects.link(obj)
        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)
        obj.shape_key_add(name="Basis")
        shape = obj.shape_key_add(name="Raise")
        shape.data[0].co.z = 1
        for frame, value in [(1, 0), (10, 1)]:
            shape.value = value
            shape.keyframe_insert(data_path="value", frame=frame)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "animated.glb"
            texture_path = path.parent / "metal color.png"
            texture = bpy.data.images.new("external metal", width=2, height=2)
            texture.filepath_raw = str(texture_path)
            texture.file_format = "PNG"
            texture.save()
            bpy.ops.export_scene.gltf(filepath=str(path), export_format="GLB", export_animations=True)
            asset = Asset(path)
            self.assertTrue(asset.doc.get("animations"))
            asset.doc['meshes'][0]['weights'] = [.35]
            asset.doc['images'] = [{'uri': 'metal%20color.png'}]
            asset.doc['textures'] = [{'source': 0}]
            asset.doc['materials'] = [{'pbrMetallicRoughness': {'baseColorTexture': {'index': 0}}}]
            asset.doc['meshes'][0]['primitives'][0]['material'] = 0
            asset.write(path)
            original = path.read_bytes()
            bpy.ops.object.select_all(action="SELECT")
            bpy.ops.object.delete(use_global=False)
            imported = import_glb(path)
            plate = next(item for item in imported if item.type == 'MESH')
            keys = plate.data.shape_keys
            self.assertIsNone(keys.animation_data)
            for frame in [1, 10]:
                bpy.context.scene.frame_set(frame)
                self.assertAlmostEqual(keys.key_blocks['Raise'].value, .35, places=6)
            images = [node.image for node in plate.data.materials[0].node_tree.nodes if node.type == 'TEX_IMAGE']
            self.assertTrue(images)
            for image in images:
                self.assertEqual(list(image.size), [2, 2], image.filepath)
                self.assertEqual(len(image.pixels), 16, image.filepath)
            self.assertEqual(path.read_bytes(), original)


@unittest.skipIf(bpy is None, "Requires Blender")
class CameraTests(unittest.TestCase):
    def test_perspective_retains_depth_cues_while_orthographic_keeps_scale(self):
        from bpy_extras.object_utils import world_to_camera_view
        scene = bpy.context.scene
        camera = bpy.data.objects.new("projection fixture", bpy.data.cameras.new("projection fixture"))
        scene.collection.objects.link(camera)
        try:
            camera.location = game_vector([0, 1, 3])
            camera.rotation_euler = (game_vector([0, 1, 0]) - camera.location).to_track_quat("-Z", "Y").to_euler()
            bpy.context.view_layer.update()
            for field, value in [("orthographic_scale", 2), ("focal_length_mm", 105)]:
                Camera.parse({"position": [0, 1, 3], "look_at": [0, 1, 0], field: value}).configure(camera.data)
                near = world_to_camera_view(scene, camera, game_vector([.2, 1, 1])).x - .5
                far = world_to_camera_view(scene, camera, game_vector([.2, 1, -1])).x - .5
                if field == "focal_length_mm":
                    self.assertAlmostEqual(near / far, 2, places=5)
                else:
                    self.assertAlmostEqual(near, far, places=6)
        finally:
            data = camera.data
            bpy.data.objects.remove(camera, do_unlink=True)
            bpy.data.cameras.remove(data)


@unittest.skipIf(bpy is None, "Requires Blender")
class PoseTests(unittest.TestCase):
    def setUp(self):
        from mathutils import Vector
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete(use_global=False)
        self.armature = bpy.data.objects.new("fixture", bpy.data.armatures.new("fixture"))
        bpy.context.collection.objects.link(self.armature)
        bpy.context.view_layer.objects.active = self.armature
        self.armature.select_set(True)
        bpy.ops.object.mode_set(mode="EDIT")
        root = self.armature.data.edit_bones.new("root")
        root.head, root.tail = Vector((0, 0, 0)), Vector((0, 1, 0))
        child = self.armature.data.edit_bones.new("child")
        child.head, child.tail, child.parent = Vector((0, 1, 0)), Vector((0, 2, 0)), root
        bpy.ops.object.mode_set(mode="OBJECT")

    def test_parent_rotation_preserves_child_length_and_rest_pose(self):
        from mathutils import Vector
        apply_pose([self.armature], {})
        bpy.context.view_layer.update()
        child = self.armature.pose.bones["child"]
        self.assertLess((child.tail - Vector((0, 2, 0))).length, 1e-6)
        apply_pose([self.armature], {"root": {"axis": [0, 1, 0], "degrees": 90}})
        bpy.context.view_layer.update()
        self.assertLess((child.tail - Vector((-2, 0, 0))).length, 1e-6)
        self.assertAlmostEqual((child.tail - child.head).length, 1, places=6)

    def test_unknown_bone_is_not_silently_ignored(self):
        with self.assertRaises(ValueError):
            apply_pose([self.armature], {"missing": {"axis": [0, 1, 0], "degrees": 20}})


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0]])
