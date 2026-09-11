"""Bake high-detail armor normals and independent linear ambient occlusion.

blender --background --python-exit-code 1 --python scripts/bake_armor.py -- DIRECTORY
The material atlas is TEXCOORD_0; anatomical UVs, positions, skin and hinges
remain unchanged. Normal detail is measured against a smooth shading carrier.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bpy
import numpy as np
from armor_glb import Asset, retains_body_uvs
from armor_bake_math import carrier_normals, dilate, frame_tangents, normal_atlas, unit


def save_image(image, directory, channel):
    temporary = directory / f"armor-{channel}.png"
    image.file_format = "PNG"
    image.filepath_raw = str(temporary.resolve())
    image.save()
    digest = hashlib.sha256(temporary.read_bytes()).hexdigest()
    path = directory / f"armor-{channel}-{digest}.png"
    temporary.replace(path)
    return path.name


def texture(asset, filename):
    images, textures = asset.doc.setdefault("images", []), asset.doc.setdefault("textures", [])
    images.append({"uri": filename, "mimeType": "image/png"})
    textures.append({"source": len(images) - 1})
    return {"index": len(textures) - 1, "texCoord": 0}


def ao_objects(asset, resolution):
    """Only the actual geometry within this equipment item occludes its bake."""
    objects = []
    for mesh in asset.doc["meshes"]:
        for primitive in mesh["primitives"]:
            attributes = primitive["attributes"]
            positions = asset.array(attributes["POSITION"])
            faces = asset.array(primitive["indices"]).reshape(-1, 3)
            data = bpy.data.meshes.new(mesh["name"])
            data.from_pydata(positions.tolist(), [], faces.tolist())
            data.update()
            for face in data.polygons:
                face.use_smooth = True
            data.normals_split_custom_set_from_vertices(asset.array(attributes["NORMAL"]).tolist())
            uv = asset.array(attributes["TEXCOORD_0"])
            layer = data.uv_layers.new(name="armor_material")
            for loop in data.loops:
                u, v = uv[loop.vertex_index]
                layer.data[loop.index].uv = (float(u), float(1 - v))
            obj = bpy.data.objects.new(mesh["name"], data)
            bpy.context.collection.objects.link(obj)
            material = bpy.data.materials.new(mesh["name"])
            material.use_nodes = True
            image = bpy.data.images.new(mesh["name"] + " AO", resolution, resolution, alpha=False)
            image.colorspace_settings.name = "Non-Color"
            node = material.node_tree.nodes.new("ShaderNodeTexImage")
            node.image = image
            material.node_tree.nodes.active = node
            data.materials.append(material)
            objects.append((primitive, obj, image))
    return objects


def process(path, resolution, samples):
    if retains_body_uvs(path):
        return []
    asset = Asset(path)
    primitives = [p for m in asset.doc["meshes"] for p in m["primitives"]]
    if all("adventuresim_surface_bake" in p.get("extras", {}) for p in primitives):
        return []
    for primitive in primitives:
        if primitive.get("extras", {}).get("adventuresim_material_uv", {}).get("channel") != 0:
            raise ValueError(f"unwrap {path.name} before baking")
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = samples
    scene.render.bake.margin = 2
    scene.render.bake.use_clear = True
    scene.render.bake.use_selected_to_active = False
    objects = ao_objects(asset, resolution)
    result = []
    for primitive, obj, ao_image in objects:
        attributes = primitive["attributes"]
        positions = asset.array(attributes["POSITION"])
        original = asset.array(attributes["NORMAL"])
        faces = asset.array(primitive["indices"]).reshape(-1, 3)
        uv = asset.array(attributes["TEXCOORD_0"])
        fields = np.stack([original, *[
            unit(original + asset.array(target["NORMAL"])) for target in primitive.get("targets", [])
        ]], axis=1)
        smooth = carrier_normals(positions, faces, fields)
        tangent = frame_tangents(smooth[:, 0], asset.array(attributes["TANGENT"]))
        pixels, covered = normal_atlas(uv, faces, original, smooth[:, 0], tangent, resolution)
        detail = float(np.max(np.linalg.norm(pixels[covered, :2] - .5, axis=1)))
        pixels = dilate(pixels, covered.copy())
        normal_image = bpy.data.images.new(obj.name + " detail", resolution, resolution, alpha=False)
        normal_image.colorspace_settings.name = "Non-Color"
        # Blender image buffers start at the bottom; glTF atlas rows start at top.
        normal_image.pixels.foreach_set(pixels[::-1].flatten())
        normal_file = save_image(normal_image, path.parent, "normal")
        bpy.data.images.remove(normal_image)
        bpy.ops.object.select_all(action="DESELECT")
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.bake(type="AO")
        # Unused atlas space must be unoccluded, not black: mip filtering can
        # sample beyond a small chart's gutter at gameplay distances.
        ao_pixels = np.array(ao_image.pixels[:], dtype=np.float32).reshape(resolution, resolution, 4)[::-1].copy()
        gutter = covered.copy()
        dilate(np.zeros_like(ao_pixels), gutter)
        ao_pixels[~gutter] = 1
        ao_image.pixels.foreach_set(ao_pixels[::-1].flatten())
        ao_file = save_image(ao_image, path.parent, "ao")
        material = copy.deepcopy(asset.doc["materials"][primitive["material"]])
        material["normalTexture"] = texture(asset, normal_file)
        material["occlusionTexture"] = texture(asset, ao_file)
        primitive["material"] = len(asset.doc["materials"])
        asset.doc["materials"].append(material)
        attributes["NORMAL"] = asset.append(smooth[:, 0])
        attributes["TANGENT"] = asset.append(tangent)
        for index, target in enumerate(primitive.get("targets", [])):
            target["NORMAL"] = asset.append(smooth[:, index + 1] - smooth[:, 0])
        primitive.setdefault("extras", {})["adventuresim_surface_bake"] = {
            "resolution": resolution, "ao_samples": samples, "material_uv": 0,
            "normal_source": "detailed_geometry", "carrier": "smoothed_shading_normals",
        }
        result.append({"mesh": obj.name, "normal": normal_file, "ao": ao_file,
                       "maximum_tangent_detail": detail, "covered_texels": int(covered.sum())})
    asset.write(path)
    for _, obj, image in objects:
        data = obj.data
        materials = list(data.materials)
        bpy.data.objects.remove(obj, do_unlink=True)
        bpy.data.meshes.remove(data)
        for material in materials:
            bpy.data.materials.remove(material)
        bpy.data.images.remove(image)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--resolution", type=int, default=1024, choices=[256, 512, 1024, 2048])
    parser.add_argument("--samples", type=int, default=32)
    parser.add_argument("--only", action="append")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    if not 1 <= args.samples <= 256:
        parser.error("AO samples must be 1..256")
    results = {}
    for path in sorted(args.directory.glob("*.glb")):
        if args.only and path.stem not in args.only:
            continue
        results[path.name] = process(path, args.resolution, args.samples)
        print(path.name, results[path.name], flush=True)
    if not results:
        parser.error("no matching equipment assets")
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
