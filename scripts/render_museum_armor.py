"""Render a museum outfit from actual GLBs or exported review JSON.

blender --background --python-exit-code 1 --python scripts/render_museum_armor.py -- PRESET.json OUTPUT_DIR

All positions, look-at points and pose axes use game coordinates (Y up,
Z forward), in metres. Colors are linear RGBA. Input paths resolve against
the preset directory, or --source-root for separately generated assets.
This is an external display renderer, not fit acceptance.
"""
import argparse
from dataclasses import dataclass
import hashlib
import json
import math
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))


def fields(row, allowed, required=()):
    if not isinstance(row, dict) or set(row) - set(allowed) or set(required) - set(row):
        raise ValueError(f"Expected fields {allowed}; required {required}")


def vector(value, length, label, unit=False):
    if not isinstance(value, list) or len(value) != length or any(
        isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v)
        or (unit and not 0 <= v <= 1) for v in value
    ):
        raise ValueError(f"Invalid {label}: expected {length} finite numbers")
    return tuple(value)


def number(value, low, high, label):
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or not low <= value <= high:
        raise ValueError(f"Invalid {label}: expected {low}..{high}")
    return value


@dataclass(frozen=True)
class Material:
    base_color: tuple | None
    metallic: float | None
    roughness: float | None

    @classmethod
    def parse(cls, row):
        fields(row, ("base_color", "metallic", "roughness"))
        return cls(vector(row["base_color"], 4, "base_color", True) if "base_color" in row else None,
                   number(row["metallic"], 0, 1, "metallic") if "metallic" in row else None,
                   number(row["roughness"], 0, 1, "roughness") if "roughness" in row else None)


@dataclass(frozen=True)
class Camera:
    position: tuple
    look_at: tuple
    orthographic_scale: float | None
    focal_length_mm: float | None

    @classmethod
    def parse(cls, row):
        fields(row, ("position", "look_at", "orthographic_scale", "focal_length_mm"),
               ("position", "look_at"))
        if ("orthographic_scale" in row) == ("focal_length_mm" in row):
            raise ValueError("Choose either orthographic_scale or focal_length_mm")
        position, target = vector(row["position"], 3, "position"), vector(row["look_at"], 3, "look_at")
        if position == target:
            raise ValueError("Camera position and look_at coincide")
        return cls(position, target,
                   number(row["orthographic_scale"], .01, 100, "orthographic_scale")
                   if "orthographic_scale" in row else None,
                   number(row["focal_length_mm"], 15, 300, "focal_length_mm")
                   if "focal_length_mm" in row else None)

    def configure(self, data):
        if self.focal_length_mm is not None:
            data.type = "PERSP"
            data.lens = self.focal_length_mm
            data.sensor_fit = "HORIZONTAL"
            data.sensor_width = 36
        else:
            data.type = "ORTHO"
            data.ortho_scale = self.orthographic_scale


@dataclass(frozen=True)
class Item:
    name: str
    path: Path
    material: Material | None
    components: dict

    @classmethod
    def parse(cls, row, directory):
        fields(row, ("name", "path", "material", "components"), ("name", "path"))
        if not isinstance(row["name"], str) or not row["name"] or not isinstance(row["path"], str):
            raise ValueError("Items require nonempty names and file paths")
        path = (directory / row["path"]).resolve()
        if path.suffix.lower() not in (".json", ".glb") or not path.is_file():
            raise ValueError(f"Missing GLB or review JSON: {path}")
        components = row.get("components", {})
        if not isinstance(components, dict) or any(not isinstance(k, str) or not k for k in components):
            raise ValueError("components must map exact mesh names/roles to material overrides")
        return cls(row["name"], path, Material.parse(row["material"]) if "material" in row else None,
                   {key: Material.parse(value) for key, value in components.items()})


@dataclass(frozen=True)
class Preset:
    camera: Camera
    items: tuple
    resolution: tuple
    background: tuple
    samples: int
    pose: dict
    skeleton: Path | None

    @classmethod
    def load(cls, path, source_root=None):
        directory = source_root.resolve() if source_root is not None else path.parent
        row = json.loads(path.read_text(encoding="utf-8-sig"))
        fields(row, ("version", "camera", "items", "resolution", "background", "samples", "pose", "skeleton"),
               ("version", "camera", "items"))
        if type(row["version"]) is not int or row["version"] != 1:
            raise ValueError("Unsupported render preset version")
        if not isinstance(row["items"], list) or not row["items"]:
            raise ValueError("An outfit needs at least one item, including its body if desired")
        items = tuple(Item.parse(value, directory) for value in row["items"])
        if len({item.name for item in items}) != len(items):
            raise ValueError("Item names must be unique")
        resolution = vector(row.get("resolution", [900, 1200]), 2, "resolution")
        if any(type(v) is not int or not 64 <= v <= 8192 for v in resolution):
            raise ValueError("resolution must contain integers from 64 to 8192")
        samples = row.get("samples", 32)
        if type(samples) is not int or not 1 <= samples <= 4096:
            raise ValueError("samples must be an integer from 1 to 4096")
        pose = row.get("pose", {})
        if not isinstance(pose, dict):
            raise ValueError("pose must map bone names to axis/degrees rotations")
        for bone, rotation in pose.items():
            if not isinstance(bone, str) or not bone:
                raise ValueError("Pose bone names cannot be empty")
            fields(rotation, ("axis", "degrees"), ("axis", "degrees"))
            axis = vector(rotation["axis"], 3, "pose axis")
            if sum(v * v for v in axis) < 1e-12:
                raise ValueError("Pose axis cannot be zero")
            number(rotation["degrees"], -180, 180, "pose degrees")
        if "skeleton" in row and not isinstance(row["skeleton"], str):
            raise ValueError("skeleton must be a GLB path")
        skeleton = (directory / row["skeleton"]).resolve() if "skeleton" in row else None
        if skeleton and (not skeleton.is_file() or skeleton.suffix.lower() != ".glb"):
            raise ValueError("skeleton must refer to an existing GLB")
        return cls(Camera.parse(row["camera"]), items, resolution,
                   vector(row.get("background", [.22, .22, .22]), 3, "background", True), samples, pose, skeleton)


def game_vector(point):
    from mathutils import Vector
    return Vector((point[0], -point[2], point[1]))


def override_material(obj, override):
    """Explicit overrides replace only their chosen scalar/color channels."""
    import bpy
    if not obj.data.materials:
        obj.data.materials.append(bpy.data.materials.new(obj.name))
    for slot in obj.material_slots:
        material = slot.material.copy() if slot.material else bpy.data.materials.new(obj.name)
        slot.material = material
        material.use_nodes = True
        shader = next(n for n in material.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        for label, value in (("Base Color", override.base_color), ("Metallic", override.metallic), ("Roughness", override.roughness)):
            if value is None:
                continue
            socket = shader.inputs[label]
            for link in list(socket.links):
                material.node_tree.links.remove(link)
            socket.default_value = value


def import_glb(path):
    import bpy
    import tempfile
    from urllib.parse import unquote
    from armor_glb import Asset
    before = set(bpy.data.objects)
    asset = Asset(path)
    if asset.doc.get("animations"):
        # Disable actions before the importer can evaluate them. Clearing
        # object actions afterward misses morph animation on mesh shape keys
        # and cannot recover the authored node/mesh default weights.
        asset.doc.pop("animations")
        for source in asset.doc.get("images", []):
            uri = source.get("uri")
            if uri and not uri.startswith(("data:", "http:", "https:")):
                source["uri"] = str((path.parent / unquote(uri)).resolve())
        with tempfile.TemporaryDirectory(prefix="museum-static-") as directory:
            static_path = Path(directory) / path.name
            asset.write(static_path)
            bpy.ops.import_scene.gltf(filepath=str(static_path), import_pack_images=False)
    else:
        bpy.ops.import_scene.gltf(filepath=str(path), import_pack_images=False)
    objects = list(set(bpy.data.objects) - before)
    glyphs = {bone.custom_shape for obj in objects if obj.type == "ARMATURE"
              for bone in obj.pose.bones if bone.custom_shape is not None}
    for glyph in glyphs:
        glyph.hide_render = True
    objects = [obj for obj in objects if obj not in glyphs]
    # An imported action must not silently change the requested display stance.
    for obj in objects:
        obj.animation_data_clear()
    return objects


def json_armature(row, skeleton):
    import bpy
    from mathutils import Matrix, Quaternion, Vector
    from armor_glb import Asset
    if skeleton is None:
        raise ValueError("Posed review JSON requires a skeleton GLB for joint hierarchy")
    for key in ("joint_names", "joints", "joint_indices", "joint_weights"):
        if key not in row:
            raise ValueError(f"Posed review JSON is missing {key}")
    asset = Asset(skeleton)
    nodes = asset.doc["nodes"]
    parent_names = {nodes[child]["name"]: node.get("name") for node in nodes for child in node.get("children", [])}
    source_joints = {nodes[index]["name"] for skin in asset.doc["skins"] for index in skin["joints"]}
    names = row["joint_names"]
    if len(set(names)) != len(names) or set(names) - source_joints:
        raise ValueError("Review joints must be unique and present in the skeleton GLB")
    if any(parent_names.get(name) in source_joints - set(names) for name in names):
        raise ValueError("Review joint hierarchy omits an ancestor")
    armature = bpy.data.objects.new("Review skeleton", bpy.data.armatures.new("Review skeleton"))
    bpy.context.collection.objects.link(armature)
    bpy.context.view_layer.objects.active = armature
    armature.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    basis = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))
    if len(row["joint_names"]) != len(row["joints"]):
        raise ValueError("Review joint names/states length mismatch")
    for name, state in zip(row["joint_names"], row["joints"]):
        vector(state, 8, "global joint state")
        if abs(sum(v * v for v in state[3:7]) - 1) > 1e-3 or state[7] <= 0:
            raise ValueError("Review joints require unit quaternions and positive scales")
        x, y, z, w = state[3:7]
        bone = armature.data.edit_bones.new(name)
        bone.head, bone.tail = (0, 0, 0), (0, .02, 0)
        bone.matrix = basis @ Matrix.LocRotScale(Vector(state[:3]), Quaternion((w, x, y, z)), Vector((1, 1, 1)))
    for bone in armature.data.edit_bones:
        parent = parent_names.get(bone.name)
        if parent in armature.data.edit_bones:
            bone.parent = armature.data.edit_bones[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    armature.select_set(False)
    return armature


def import_review(path, preset):
    import bpy
    row = json.loads(path.read_text(encoding="utf-8-sig"))
    positions, normals = row["positions"], row["normals"]
    faces = row.get("faces") or [row["indices"][i:i + 3] for i in range(0, len(row["indices"]), 3)]
    if len(normals) != len(positions):
        raise ValueError("Review normal count does not match positions")
    armature = json_armature(row, preset.skeleton) if preset.pose else None
    components = row.get("components") or [{"role": row.get("id", "body"),
        "vertices": {"start": 0, "end": len(positions)}, "indices": {"start": 0, "end": len(faces) * 3}}]
    objects = [armature] if armature else []
    for component in components:
        start, end = component["vertices"]["start"], component["vertices"]["end"]
        first, last = component["indices"]["start"], component["indices"]["end"]
        if not (0 <= start < end <= len(positions) and 0 <= first < last <= len(faces) * 3 and first % 3 == last % 3 == 0):
            raise ValueError("Invalid review component ranges")
        selected = [[v - start for v in face] for face in faces[first // 3:last // 3]]
        if any(len(f) != 3 or min(f) < 0 or max(f) >= end - start for f in selected):
            raise ValueError("Review component triangle escapes its vertex range")
        mesh = bpy.data.meshes.new(component["role"])
        mesh.from_pydata([game_vector(vector(p, 3, "position")) for p in positions[start:end]], [], selected)
        mesh.update()
        for polygon in mesh.polygons:
            polygon.use_smooth = True
        mesh.normals_split_custom_set_from_vertices([game_vector(vector(n, 3, "normal")) for n in normals[start:end]])
        obj = bpy.data.objects.new(component["role"], mesh)
        obj["museum_component"] = component["role"]
        bpy.context.collection.objects.link(obj)
        override_material(obj, Material.parse(component.get("material") or {
            "base_color": [.45, .48, .50, 1], "metallic": .9, "roughness": .35}))
        if armature:
            if len(row["joint_indices"]) != len(positions) or len(row["joint_weights"]) != len(positions):
                raise ValueError("Review skin counts do not match positions")
            groups = [obj.vertex_groups.new(name=name) for name in row["joint_names"]]
            for index in range(start, end):
                ids, weights = row["joint_indices"][index], row["joint_weights"][index]
                if len(ids) != len(weights) or abs(sum(weights) - 1) > 1e-4 or any(w < 0 for w in weights):
                    raise ValueError("Invalid review skin weights")
                for joint, weight in zip(ids, weights):
                    if type(joint) is not int or not 0 <= joint < len(groups) or not math.isfinite(weight):
                        raise ValueError("Invalid review skin influence")
                    if weight:
                        groups[joint].add([index - start], weight, "REPLACE")
            obj.modifiers.new("Exported skin", "ARMATURE").object = armature
        objects.append(obj)
    return objects


def apply_pose(objects, pose):
    from mathutils import Matrix, Quaternion
    matched = set()
    for obj in objects:
        if obj.type != "ARMATURE":
            continue
        for bone in obj.pose.bones:
            delta = Matrix.Identity(4)
            if bone.name in pose:
                rotation = pose[bone.name]
                axis = (obj.matrix_world.to_3x3().inverted() @ game_vector(rotation["axis"])).normalized()
                pivot = bone.bone.matrix_local.translation
                delta = Matrix.Translation(pivot) @ Quaternion(axis, math.radians(rotation["degrees"])).to_matrix().to_4x4() @ Matrix.Translation(-pivot)
                matched.add(bone.name)
            # Set the local basis directly. Assigning successive global pose
            # matrices uses stale parent evaluation and can stretch descendants.
            bone.matrix_basis = bone.bone.matrix_local.inverted() @ delta @ bone.bone.matrix_local
    if set(pose) - matched:
        raise ValueError(f"Pose bones absent from imported skeletons: {sorted(set(pose) - matched)}")


def studio(preset):
    import bpy
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device, scene.cycles.samples = "CPU", preset.samples
    scene.cycles.use_denoising = True
    scene.render.resolution_x, scene.render.resolution_y = preset.resolution
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "AgX"
    scene.world.use_nodes = True
    scene.world.node_tree.nodes.get("Background").inputs["Color"].default_value = (*preset.background, 1)
    scene.world.node_tree.nodes.get("Background").inputs["Strength"].default_value = .7
    camera = bpy.data.objects.new("Museum camera", bpy.data.cameras.new("Museum camera"))
    bpy.context.collection.objects.link(camera)
    camera.location = game_vector(preset.camera.position)
    target = game_vector(preset.camera.look_at)
    camera.rotation_euler = (target - camera.location).to_track_quat("-Z", "Y").to_euler()
    preset.camera.configure(camera.data)
    scene.camera = camera
    for offset, watts in [((2, 2, 3), 350), ((-3, 1, 1), 180), ((0, 2, -3), 250)]:
        light = bpy.data.lights.new("Studio softbox", "AREA")
        light.energy, light.size = watts, 3
        obj = bpy.data.objects.new(light.name, light)
        bpy.context.collection.objects.link(obj)
        obj.location = target + game_vector(offset)
        obj.rotation_euler = (target - obj.location).to_track_quat("-Z", "Y").to_euler()
    return scene


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("preset", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--source-root", type=Path,
                        help="Resolve item and skeleton paths against this generated asset directory")
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    preset = Preset.load(args.preset.resolve(), args.source_root)
    import bpy
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    objects, evidence = [], []
    for item in preset.items:
        imported = import_glb(item.path) if item.path.suffix.lower() == ".glb" else import_review(item.path, preset)
        matched = set()
        for obj in imported:
            if obj.type != "MESH":
                continue
            role = obj.get("museum_component", obj.data.name.rsplit(".", 1)[0] if obj.data.name[-4:-3] == "." else obj.data.name)
            if item.material:
                override_material(obj, item.material)
            if role in item.components:
                override_material(obj, item.components[role])
                matched.add(role)
            obj.name = f"{item.name}/{role}"
        if set(item.components) - matched:
            raise ValueError(f"Unmatched component overrides on {item.name}: {set(item.components) - matched}")
        objects.extend(imported)
        evidence.append({"name": item.name, "path": str(item.path), "sha256": hashlib.sha256(item.path.read_bytes()).hexdigest(),
                         "meshes": sum(o.type == "MESH" for o in imported), "armatures": sum(o.type == "ARMATURE" for o in imported)})
    apply_pose(objects, preset.pose)
    scene = studio(preset)
    args.output.mkdir(parents=True, exist_ok=True)
    scene.render.filepath = str((args.output / "outfit.png").resolve())
    bpy.ops.render.render(write_still=True)
    (args.output / "render-input.json").write_text(args.preset.read_text(encoding="utf-8-sig"), encoding="utf-8")
    (args.output / "render-evidence.json").write_text(json.dumps({"items": evidence, "blender": bpy.app.version_string,
        "source_root": str((args.source_root or args.preset.parent).resolve()),
        "scope": "Static display render; no intersection or historical acceptance implied. GLB PBR maps preserved unless explicitly overridden. JSON has scalar component materials only. Cycles computes AO through ray tracing; imported glTF AO is not multiplied into albedo.",
        "pose_axes": "game coordinates at rest; descendant bones inherit parent rotations"}, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
