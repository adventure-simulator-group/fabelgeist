"""Body-visible material comparison of original and finished equipment GLBs.

blender --background --python scripts/render_armor_materials.py -- SOURCE FINISHED BODY OUTPUT
The optional decimated view is a static bake demonstration, not a runtime LOD.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import sys
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bpy
from mathutils import Vector
from armor_glb import Asset


def build(path, detail, color=None):
    asset, objects = Asset(path), []
    for mesh in asset.doc["meshes"]:
        for primitive in mesh["primitives"]:
            a = primitive["attributes"]
            data = bpy.data.meshes.new(mesh["name"])
            data.from_pydata([(x, -z, y) for x, y, z in asset.array(a["POSITION"])], [],
                             asset.array(primitive["indices"]).reshape(-1, 3).tolist())
            data.update()
            for polygon in data.polygons:
                polygon.use_smooth = True
            data.normals_split_custom_set_from_vertices([(x, -z, y) for x, y, z in asset.array(a["NORMAL"])])
            obj = bpy.data.objects.new(mesh["name"], data)
            bpy.context.collection.objects.link(obj)
            material = bpy.data.materials.new(mesh["name"])
            material.use_nodes = True
            shader = material.node_tree.nodes.get("Principled BSDF")
            pbr = asset.doc["materials"][primitive["material"]].get("pbrMetallicRoughness", {})
            shader.inputs["Base Color"].default_value = color or pbr.get("baseColorFactor", [.5, .5, .5, 1])
            shader.inputs["Metallic"].default_value = 0 if color else pbr.get("metallicFactor", 0)
            shader.inputs["Roughness"].default_value = .75 if color else pbr.get("roughnessFactor", .3)
            if "TEXCOORD_0" in a:
                uv = asset.array(a["TEXCOORD_0"])
                layer = data.uv_layers.new(name="armor_material")
                for loop in data.loops:
                    u, v = uv[loop.vertex_index]
                    layer.data[loop.index].uv = (float(u), float(1 - v))
            surface = asset.doc["materials"][primitive["material"]]
            if detail and "normalTexture" in surface:
                texture = asset.doc["textures"][surface["normalTexture"]["index"]]
                image = asset.doc["images"][texture["source"]]
                nodes, links = material.node_tree.nodes, material.node_tree.links
                sampler = nodes.new("ShaderNodeTexImage")
                sampler.image = bpy.data.images.load(str((path.parent / image["uri"]).resolve()))
                sampler.image.colorspace_settings.name = "Non-Color"
                # Direct GLB tangents use glTF's downwards V. Blender's UV layer
                # has upwards V, so invert the tangent-space green component.
                split, combine = nodes.new("ShaderNodeSeparateXYZ"), nodes.new("ShaderNodeCombineXYZ")
                invert = nodes.new("ShaderNodeMath")
                invert.operation = "SUBTRACT"
                invert.inputs[0].default_value = 1
                links.new(sampler.outputs["Color"], split.inputs[0])
                links.new(split.outputs["X"], combine.inputs["X"])
                links.new(split.outputs["Y"], invert.inputs[1])
                links.new(invert.outputs[0], combine.inputs["Y"])
                links.new(split.outputs["Z"], combine.inputs["Z"])
                normal = nodes.new("ShaderNodeNormalMap")
                normal.uv_map = "armor_material"
                links.new(combine.outputs[0], normal.inputs["Color"])
                links.new(normal.outputs["Normal"], shader.inputs["Normal"])
            data.materials.append(material)
            objects.append(obj)
    return objects


def transfer_carrier_in_uv(obj, source):
    """Transfer within the atlas, avoiding nearest-surface jumps to plate backs."""
    graph = bpy.context.evaluated_depsgraph_get()
    old = obj.data
    obj.data = bpy.data.meshes.new_from_object(obj.evaluated_get(graph), depsgraph=graph)
    obj.modifiers.clear()
    bpy.data.meshes.remove(old)
    source_uv = np.array([v.uv[:] for v in source.data.uv_layers.active.data])
    triangles = np.array([p.loop_indices[:] for p in source.data.polygons])
    atlas = source_uv[triangles]
    fields = np.array([n.vector[:] for n in source.data.corner_normals])[triangles]
    points = np.array([v.uv[:] for v in obj.data.uv_layers.active.data])
    output = np.zeros((len(points), 3))
    bins = {}
    size = 64
    for index, triangle in enumerate(atlas):
        lower = np.floor(triangle.min(axis=0) * size).astype(int)
        upper = np.floor(triangle.max(axis=0) * size).astype(int)
        for y in range(lower[1], upper[1] + 1):
            for x in range(lower[0], upper[0] + 1):
                bins.setdefault((x, y), []).append(index)
    for index, point in enumerate(points):
        candidates = bins.get(tuple(np.floor(point * size).astype(int)), [])
        sample = atlas[candidates]
        delta, a, b = point - sample[:, 0], sample[:, 1] - sample[:, 0], sample[:, 2] - sample[:, 0]
        det = a[:, 0] * b[:, 1] - a[:, 1] * b[:, 0]
        v = (delta[:, 0] * b[:, 1] - delta[:, 1] * b[:, 0]) / det
        w = (a[:, 0] * delta[:, 1] - a[:, 1] * delta[:, 0]) / det
        weights = np.column_stack((1 - v - w, v, w))
        match = np.argmax(weights.min(axis=1))
        if weights[match].min() < -1e-3:
            raise ValueError("simplification moved a material coordinate outside its chart")
        output[index] = weights[match] @ fields[candidates[match]]
    output /= np.linalg.norm(output, axis=1, keepdims=True)
    obj.data.normals_split_custom_set(output.tolist())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "finished", "body", "output"):
        parser.add_argument(name, type=Path)
    parser.add_argument("--ratio", type=float, default=.65)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    args.output.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 32
    scene.cycles.use_denoising = True
    scene.render.resolution_x, scene.render.resolution_y = 1000, 800
    scene.world.color = (.18, .18, .18)
    body = build(args.body, False, [.24, .16, .10, 1])
    original = build(args.source, False)
    baked = build(args.finished, True)
    low = build(args.finished, True)
    for obj, source in zip(low, baked):
        edge_counts = Counter(tuple(sorted(edge)) for face in obj.data.polygons for edge in face.edge_keys)
        protected = {vertex for edge, count in edge_counts.items() if count == 1 for vertex in edge}
        protected |= {vertex for edge in obj.data.edges if protected.intersection(edge.vertices)
                      for vertex in edge.vertices}
        group = obj.vertex_groups.new(name="Chart interiors")
        interior = [vertex.index for vertex in obj.data.vertices if vertex.index not in protected]
        if interior:
            group.add(interior, 1.0, "REPLACE")
        modifier = obj.modifiers.new("Static simplified bake demonstration", "DECIMATE")
        modifier.ratio = args.ratio
        modifier.vertex_group = group.name
        modifier.use_collapse_triangulate = True
        # A simplifier must preserve the low-frequency shading carrier as well
        # as material UVs. Recomputed geometric normals would apply relief twice.
        transfer_carrier_in_uv(obj, source)
    graph = bpy.context.evaluated_depsgraph_get()
    (args.output / "triangles.json").write_text(json.dumps({
        "source": sum(len(obj.data.polygons) for obj in original),
        "simplified": sum(len(obj.evaluated_get(graph).data.polygons) for obj in low),
    }))
    points = [obj.matrix_world @ vertex.co for obj in original for vertex in obj.data.vertices]
    center = sum(points, Vector()) / len(points)
    extent = max((point - center).length for point in points)
    camera = bpy.data.objects.new("Camera", bpy.data.cameras.new("Camera"))
    bpy.context.collection.objects.link(camera)
    camera.location = center + Vector((1, -3, 1))
    camera.rotation_euler = (center - camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.type, camera.data.ortho_scale = "ORTHO", extent * 3.6
    scene.camera = camera
    for offset, energy, size in [((2, -3, 3), 250, 2), ((-3, -1, 1), 150, 2), ((0, 2, 2), 250, 2)]:
        light = bpy.data.lights.new("Softbox", "AREA")
        light.energy, light.size = energy, size
        obj = bpy.data.objects.new(light.name, light)
        bpy.context.collection.objects.link(obj)
        obj.location = center + Vector(offset)
        obj.rotation_euler = (center - obj.location).to_track_quat("-Z", "Y").to_euler()
    for name, visible in [("source", original), ("baked", baked), ("simplified-baked", low)]:
        for obj in original + baked + low:
            obj.hide_render = obj not in visible
        scene.render.filepath = str((args.output / f"{name}.png").resolve())
        bpy.ops.render.render(write_still=True)
    (args.output / "scope.txt").write_text(
        "Actual GLB geometry, UVs and material maps. Body included. Simplified view\n"
        "uses static Blender decimation and transfers carrier\n"
        "normals along with material UVs; no runtime LOD\n"
        "switching claimed. Cycles AO is ray traced, not multiplied into albedo.\n")


if __name__ == "__main__":
    main()
