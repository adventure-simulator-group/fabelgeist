"""Bake square armor portraits from finished meshes with the shared studio HDR.

blender --background --python-exit-code 1 --python scripts/render_equipment_icons.py -- DIRECTORY
"""
import argparse
import json
from pathlib import Path
import shutil
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from render_armor_materials import build

ICON_SIZE = 128
FRAME_MARGIN = 1.12
ROOT = Path(__file__).resolve().parent.parent


def render(path, output, environment):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    objects = build(path, detail=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 32
    scene.cycles.seed = 0
    scene.cycles.use_denoising = True
    scene.render.resolution_x = scene.render.resolution_y = ICON_SIZE
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.film_transparent = True
    scene.view_settings.view_transform = "AgX"
    world = bpy.data.worlds.new("Equipment studio")
    scene.world = world
    world.use_nodes = True
    nodes, links = world.node_tree.nodes, world.node_tree.links
    texture = nodes.new("ShaderNodeTexEnvironment")
    texture.image = bpy.data.images.load(str(environment))
    links.new(texture.outputs["Color"], nodes.get("Background").inputs["Color"])
    points = [obj.matrix_world @ vertex.co for obj in objects for vertex in obj.data.vertices]
    if not points:
        raise ValueError(f"No equipment geometry in {path}")
    lower = Vector(tuple(min(p[axis] for p in points) for axis in range(3)))
    upper = Vector(tuple(max(p[axis] for p in points) for axis in range(3)))
    center = (lower + upper) * .5
    camera = bpy.data.objects.new("Portrait camera", bpy.data.cameras.new("Portrait camera"))
    scene.collection.objects.link(camera)
    camera.location = center + Vector((1.5, -4, 1.0))
    camera.rotation_euler = (center - camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.type = "ORTHO"
    scene.camera = camera
    rotation = camera.rotation_euler.to_matrix().transposed()
    projected = [rotation @ (point - center) for point in points]
    bounds = [(min(p[axis] for p in projected), max(p[axis] for p in projected)) for axis in range(2)]
    offset = camera.rotation_euler.to_matrix() @ Vector(((bounds[0][0] + bounds[0][1]) / 2, (bounds[1][0] + bounds[1][1]) / 2, 0))
    camera.location += offset
    camera.data.ortho_scale = max(high - low for low, high in bounds) * FRAME_MARGIN
    print(f"Portrait {path.name}: bounds={lower[:]}..{upper[:]}, scale={camera.data.ortho_scale}, camera={camera.location[:]}", flush=True)
    # Composite onto opaque black while retaining environment illumination.
    scene.use_nodes = True
    tree = scene.compositing_node_group
    if tree is None:
        tree = bpy.data.node_groups.new("Portrait black background", "CompositorNodeTree")
        scene.compositing_node_group = tree
        tree.interface.new_socket(name="Image", in_out="OUTPUT", socket_type="NodeSocketColor")
    tree.nodes.clear()
    render_layer = tree.nodes.new("CompositorNodeRLayers")
    over = tree.nodes.new("CompositorNodeAlphaOver")
    over.inputs["Background"].default_value = (0, 0, 0, 1)
    tree.links.new(render_layer.outputs["Image"], over.inputs["Foreground"])
    output_node = tree.nodes.new("NodeGroupOutput")
    tree.links.new(over.outputs[0], output_node.inputs[0])
    scene.render.filepath = str(output)
    bpy.ops.render.render(write_still=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--web-output", type=Path)
    parser.add_argument("--item", help="Render just one catalog item for visual review")
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    directory = args.directory.resolve()
    output = directory / "icons"
    output.mkdir(parents=True, exist_ok=True)
    environment = ROOT / "assets/equipment/icon-studio.hdr"
    manifest = json.loads((directory / "manifest.json").read_text())
    for asset in manifest["assets"]:
        if args.item and asset["item_id"] != args.item:
            continue
        filename = Path(asset["file"]).with_suffix(".png").name
        render(directory / asset["file"], output / filename, environment)
        if args.web_output:
            args.web_output.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(output / filename, args.web_output / filename)


if __name__ == "__main__":
    main()
