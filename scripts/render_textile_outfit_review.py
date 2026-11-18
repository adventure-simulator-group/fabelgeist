"""Render a textile_outfit_review export on its wearer with Blender.

blender --background --python scripts/render_textile_outfit_review.py -- REVIEW_DIR OUTPUT_DIR

REVIEW_DIR holds body.json and article-N.json from the creator's
textile_outfit_review example. Every article is shown together. Each component
uses its scalar material and, where the export baked them, its textile colour
and normal maps. Views cover the whole figure and the upper body. This is an
external static render, not the game's renderer.
"""
import json
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

VIEWS = [("front", (0, .05, 1)), ("quarter", (.7, .24, 1)), ("side", (1, .06, 0)),
         ("rear", (0, .08, -1))]
UNCOLORED = {"base_color": (0.56, 0.62, 0.68, 1), "metallic": 0, "roughness": 0.8}
SKIN = {"base_color": (0.49, 0.32, 0.25, 1), "metallic": 0, "roughness": 0.6}
# Sun directions in Blender axes (Z up, the wearer faces -Y) and strengths.
LIGHTS = [((-0.5, -1, 0.8), 2.6), ((0.9, -0.6, 0.3), 0.9), ((0, 1, 0.5), 1.2)]


def image_node(tree, path, color_space):
    node = tree.nodes.new("ShaderNodeTexImage")
    node.image = bpy.data.images.load(str(path))
    node.image.colorspace_settings.name = color_space
    return node


def material(name, surface, color_map=None, normal_map=None, surface_map=None):
    result = bpy.data.materials.new(name)
    result.use_nodes = True
    tree = result.node_tree
    shader = tree.nodes["Principled BSDF"]
    # Scalar colours are sRGB-encoded like the colour maps; the shader is linear.
    shader.inputs["Base Color"].default_value = [
        *(((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92
          for c in surface["base_color"][:3]), 1]
    shader.inputs["Metallic"].default_value = surface["metallic"]
    shader.inputs["Roughness"].default_value = surface["roughness"]
    if color_map:
        color = image_node(tree, color_map, "sRGB")
        tree.links.new(color.outputs["Color"], shader.inputs["Base Color"])
        # Sheer cloth is partly transparent.
        tree.links.new(color.outputs["Alpha"], shader.inputs["Alpha"])
    if surface_map:
        # Roughness in green and metalness in blue, as glTF stores them.
        split = tree.nodes.new("ShaderNodeSeparateColor")
        tree.links.new(image_node(tree, surface_map, "Non-Color").outputs["Color"], split.inputs["Color"])
        tree.links.new(split.outputs["Green"], shader.inputs["Roughness"])
        tree.links.new(split.outputs["Blue"], shader.inputs["Metallic"])
    if normal_map:
        normal = tree.nodes.new("ShaderNodeNormalMap")
        tree.links.new(image_node(tree, normal_map, "Non-Color").outputs["Color"],
                       normal.inputs["Color"])
        tree.links.new(normal.outputs["Normal"], shader.inputs["Normal"])
    return result


def mesh_object(name, positions, faces, normals, texcoords, surface):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([(p[0], -p[2], p[1]) for p in positions], [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    if texcoords:
        # Row zero of a textile map is the proximal chart edge.
        layer = mesh.uv_layers.new()
        for loop in mesh.loops:
            u, v = texcoords[loop.vertex_index]
            layer.data[loop.index].uv = (u, 1 - v)
    mesh.materials.append(surface)
    for polygon in mesh.polygons:
        polygon.use_smooth = True
    mesh.normals_split_custom_set_from_vertices([(n[0], -n[2], n[1]) for n in normals])
    return obj


def light_scene(scene):
    for index, (direction, strength) in enumerate(LIGHTS):
        data = bpy.data.lights.new(f"Sun {index}", "SUN")
        data.energy = strength
        data.angle = math.radians(12)
        sun = bpy.data.objects.new(f"Sun {index}", data)
        bpy.context.collection.objects.link(sun)
        sun.rotation_euler = Vector(direction).to_track_quat("Z", "Y").to_euler()
    world = scene.world or bpy.data.worlds.new("Review world")
    scene.world = world
    world.use_nodes = True
    background = world.node_tree.nodes["Background"]
    background.inputs["Color"].default_value = (0.11, 0.105, 0.1, 1)
    background.inputs["Strength"].default_value = 1


def main():
    source, output = [Path(p).resolve() for p in sys.argv[sys.argv.index("--") + 1:][:2]]
    output.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    engines = [item.identifier for item in
               bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items]
    scene.render.engine = next(e for e in ("BLENDER_EEVEE_NEXT", "BLENDER_EEVEE") if e in engines)
    scene.render.resolution_x = 900
    scene.render.resolution_y = 1100
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.view_settings.view_transform = "Standard"
    light_scene(scene)
    body = json.loads((source / "body.json").read_text())
    mesh_object("Wearer", body["positions"], body["faces"], body["normals"], None,
                material("Wearer", SKIN))
    points = list(body["positions"])
    triangles = []
    for path in sorted(source.glob("article-*.json")):
        row = json.loads(path.read_text())
        faces = [row["indices"][i:i + 3] for i in range(0, len(row["indices"]), 3)]
        triangles.append({"article": path.stem, "triangles": len(faces)})
        points.extend(row["positions"])
        maps = {entry["component"]: entry for entry in row.get("textile_maps", [])}
        for ordinal, part in enumerate(row["components"]):
            start, end = part["vertices"]["start"], part["vertices"]["end"]
            first, last = part["indices"]["start"] // 3, part["indices"]["end"] // 3
            name = f"{path.stem}.{part['role']}"
            baked = maps.get(ordinal, {})
            surface = material(
                name, part.get("material") or UNCOLORED,
                path.parent / baked["base_color"] if baked.get("base_color") else None,
                path.parent / baked["normal"] if baked.get("normal") else None,
                path.parent / baked["metal_roughness"] if baked.get("metal_roughness") else None)
            mesh_object(name, row["positions"][start:end],
                        [[index - start for index in face] for face in faces[first:last]],
                        row["normals"][start:end], row["texcoords"][start:end], surface)
    low = Vector([min(p[i] for p in points) for i in range(3)])
    high = Vector([max(p[i] for p in points) for i in range(3)])
    height = high.y - low.y
    # The upper frame runs from the hips to the crown, where sleeves and bodice sit.
    frames = {
        "whole": ((low + high) * 0.5, max(height * 1.08, (high.x - low.x) * 1.1)),
        "upper": (Vector(((low.x + high.x) * 0.5, low.y + height * 0.72, (low.z + high.z) * 0.5)),
                  height * 0.62),
    }
    camera_data = bpy.data.cameras.new("Review camera")
    camera = bpy.data.objects.new("Review camera", camera_data)
    bpy.context.collection.objects.link(camera)
    scene.camera = camera
    camera_data.type = "ORTHO"
    for frame, (center, scale) in frames.items():
        for name, direction in VIEWS:
            target = Vector((center.x, -center.z, center.y))
            offset = Vector((direction[0], -direction[2], direction[1])).normalized() * 4
            camera.location = target + offset
            camera.rotation_euler = (target - camera.location).to_track_quat("-Z", "Y").to_euler()
            camera_data.ortho_scale = scale
            scene.render.filepath = str(output / f"{frame}-{name}.png")
            bpy.ops.render.render(write_still=True)
    (output / "render-settings.json").write_text(json.dumps({
        "engine": scene.render.engine, "size": [900, 1100], "source": str(source),
        "frames": list(frames), "views": [name for name, _ in VIEWS], "articles": triangles,
        "stage": "ordinary generator output; external static render",
    }, indent=2))


if __name__ == "__main__":
    main()
