"""Render actual cut meshes, their inherited UVs, and embedded source textures.

blender --background --python scripts/render_underlayer_review.py -- MESH_DIR OUTPUT_DIR MATERIAL_DIR
"""
import argparse
import json
import math
from pathlib import Path
import sys

import bpy
from mathutils import Vector

VIEWS = [
    ("front", (0, -4, 1), (0, 0, 0.9), 1.85),
    ("quarter", (3, -4, 1.6), (0, 0, 0.9), 1.85),
    ("rear", (0, 4, 1), (0, 0, 0.9), 1.85),
    ("side", (4, 0, 1), (0, 0, 0.9), 1.85),
    ("groin-front", (0, -3, 0.8), (0, 0, 0.81), 0.58),
    ("groin-rear", (0, 3, 0.8), (0, 0, 0.81), 0.58),
    ("groin-oblique", (1, -3, 0.35), (0, 0, 0.81), 0.58),
    ("groin-under", (0, -.9, .05), (0, 0, .79), .48),
    ("knee-rear", (0, 3, 0.45), (0, 0, 0.46), 0.48),
    ("collar-front", (0, -3, 1.52), (0, 0, 1.5), 0.48),
    ("collar-quarter", (1.4, -3, 1.7), (0, 0, 1.5), 0.48),
    ("armpit", (1.5, -3, 0.7), (0.24, 0, 1.30), 0.58),
    ("elbow", (0.8, -3, 1.25), (0.34, 0, 1.23), 0.22),
    ("rings", (1.5, -3, 0.9), (0.20, 0, 1.31), 0.15),
]


def material(name, color, metallic=0):
    result = bpy.data.materials.new(name)
    result.use_nodes = True
    shader = result.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = (*color, 1)
    shader.inputs["Metallic"].default_value = metallic
    shader.inputs["Roughness"].default_value = 0.65 if not metallic else 0.45
    return result


def mesh(name, positions, normals, faces, mat, uv=None):
    data = bpy.data.meshes.new(name)
    data.from_pydata([(x, -z, y) for x, y, z in positions], [], faces)
    data.update()
    data.normals_split_custom_set_from_vertices([(x, -z, y) for x, y, z in normals])
    for polygon in data.polygons:
        polygon.use_smooth = True
    if uv:
        layer = data.uv_layers.new(name="mhr_body_v1")
        for loop in data.loops:
            u, v = uv[loop.vertex_index]
            layer.data[loop.index].uv = (u, 1 - v)
    obj = bpy.data.objects.new(name, data)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(mat)
    return obj


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("maps", type=Path)
    parser.add_argument("--views", nargs="+", choices=[view[0] for view in VIEWS])
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    source, output, maps = [p.resolve() for p in (args.source, args.output, args.maps)]
    output.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 32
    scene.cycles.use_denoising = True
    scene.render.resolution_x = 1000
    scene.render.resolution_y = 1100
    scene.render.resolution_percentage = 100
    scene.world.use_nodes = True
    scene.world.node_tree.nodes["Background"].inputs[0].default_value = (0.18, 0.18, 0.18, 1)
    scene.world.node_tree.nodes["Background"].inputs[1].default_value = 0.35
    cloth = material("Thin linen arming garment", (0.36, 0.29, 0.18))
    plate = material("Plate steel", (0.42, 0.45, 0.48), 1)
    mail = material("Steel mail", (0.6, 0.6, 0.6), 1)
    shader = mail.node_tree.nodes.get("Principled BSDF")
    # Cycles has no separate glTF ambient-only occlusion input. Do not multiply
    # AO into Base Color: that would incorrectly darken direct illumination.
    # Native Bevy/glTF captures are authoritative for the independent AO map.
    (output / "material-renderer.txt").write_text(
        "Cycles review uses unlit steel base color, alpha and tangent normals.\n"
        "Separate baked AO is intentionally not applied here because Cycles\n"
        "has no ambient-only material input. Inspect native Bevy/glTF for AO.\n"
    )
    for filename, socket, normal in [("mail-base-color.png", "Base Color", False),
                                     ("mail-normal.png", "Normal", True)]:
        node = mail.node_tree.nodes.new("ShaderNodeTexImage")
        node.image = bpy.data.images.load(str(maps / filename))
        if normal:
            node.image.colorspace_settings.name = "Non-Color"
            mapping = mail.node_tree.nodes.new("ShaderNodeNormalMap")
            mail.node_tree.links.new(node.outputs["Color"], mapping.inputs["Color"])
            mail.node_tree.links.new(mapping.outputs["Normal"], shader.inputs[socket])
        else:
            mail.node_tree.links.new(node.outputs["Color"], shader.inputs[socket])
            mask = mail.node_tree.nodes.new("ShaderNodeMath")
            mask.operation = "GREATER_THAN"
            mask.inputs[1].default_value = 0.5
            mail.node_tree.links.new(node.outputs["Alpha"], mask.inputs[0])
            mail.node_tree.links.new(mask.outputs[0], shader.inputs["Alpha"])
    body = json.loads((source / "body.json").read_text())
    mesh("Body", body["positions"], body["normals"], body["faces"],
         material("Skin", (0.34, 0.20, 0.14)))
    for path in source.glob("*--*.json"):
        row = json.loads(path.read_text())
        triangles = [row["indices"][i:i+3] for i in range(0, len(row["indices"]), 3)]
        surface = mail if row["id"] in {
            "mail_voiders", "mail_brayette", "mail_knee_voider", "mail_standard"
        } else (
            cloth if row["id"] in {"arming_doublet", "padded_chausses"} else plate)
        mesh(path.stem, row["positions"], row["normals"], triangles,
             surface, row.get("texcoords"))
    for location, energy, size in [((3, -4, 5), 180, 4), ((-3, -2, 3), 100, 3), ((0, 3, 4), 150, 3)]:
        data = bpy.data.lights.new("Softbox", "AREA")
        data.energy, data.shape, data.size = energy, "DISK", size
        obj = bpy.data.objects.new("Softbox", data)
        bpy.context.collection.objects.link(obj)
        obj.location = location
        obj.rotation_euler = (Vector((0, 0, 1)) - obj.location).to_track_quat("-Z", "Y").to_euler()
    camera = bpy.data.objects.new("Camera", bpy.data.cameras.new("Camera"))
    bpy.context.collection.objects.link(camera)
    scene.camera = camera
    camera.data.type = "ORTHO"
    for name, location, target, scale in VIEWS:
        if args.views and name not in args.views:
            continue
        camera.location = location
        camera.rotation_euler = (Vector(target) - camera.location).to_track_quat("-Z", "Y").to_euler()
        camera.data.ortho_scale = scale
        scene.render.filepath = str(output / f"{name}.png")
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    main()
