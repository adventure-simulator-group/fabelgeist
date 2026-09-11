"""Bake separate interlinked round-wire rings as a periodic construction chart.

blender --background --python-exit-code 1 --python scripts/bake_mail_weave.py -- OUTPUT_DIRECTORY
Met 27.183.35 supplies the 6.2 mm outside / 4.0 mm inside link diameters.
The atlas baker carries this construction chart across the source body's UVs.
"""
import math
from pathlib import Path
import sys


RING_RADIUS_M = .00255
WIRE_RADIUS_M = .00055
PITCH_X_M = .0063
PITCH_Y_M = .0015
INCLINATION_RADIANS = math.radians(34)
STEEL_BASE_COLOR_SRGB = .71


def main():
    import bpy
    output = Path(sys.argv[sys.argv.index("--") + 1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 64
    # Denoising is appropriate for lit beauty renders, not independent data
    # channels: it can contaminate constant RGB using neighboring apertures.
    scene.cycles.use_denoising = False
    scene.render.dither_intensity = 0
    scene.render.resolution_x = 512
    scene.render.resolution_y = round(512 * 2 * PITCH_Y_M / PITCH_X_M)
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = True
    scene.view_settings.view_transform = "Raw"
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    material = bpy.data.materials.new("Unlit bake channels")
    material.use_nodes = True
    nodes = material.node_tree.nodes
    nodes.clear()
    output_node = nodes.new("ShaderNodeOutputMaterial")
    emission = nodes.new("ShaderNodeEmission")
    material.node_tree.links.new(emission.outputs[0], output_node.inputs["Surface"])
    for row in range(-4, 7):
        for column in range(-2, 4):
            bpy.ops.mesh.primitive_torus_add(major_segments=64, minor_segments=20,
                major_radius=RING_RADIUS_M, minor_radius=WIRE_RADIUS_M,
                location=((column + (row % 2) * .5) * PITCH_X_M, row * PITCH_Y_M, 0),
                rotation=(INCLINATION_RADIANS * (-1 if row % 2 else 1), 0, 0))
            obj = bpy.context.object
            obj.data.materials.append(material)
            for face in obj.data.polygons:
                face.use_smooth = True
    camera = bpy.data.objects.new("Orthographic construction chart", bpy.data.cameras.new("Camera"))
    bpy.context.collection.objects.link(camera)
    scene.camera = camera
    camera.location = (PITCH_X_M / 2, PITCH_Y_M, .03)
    camera.data.type = "ORTHO"
    camera.data.ortho_scale = PITCH_X_M
    camera.data.clip_start = .0001
    # Emission provides reflectance and silhouette only: no light or AO enters
    # the base-color channel. Raw preserves data values for all three passes.
    emission.inputs["Color"].default_value = (STEEL_BASE_COLOR_SRGB,) * 3 + (1,)
    scene.render.filepath = str(output / "color.png")
    bpy.ops.render.render(write_still=True)
    ao = nodes.new("ShaderNodeAmbientOcclusion")
    ao.inputs["Distance"].default_value = WIRE_RADIUS_M * 3
    material.node_tree.links.new(ao.outputs["AO"], emission.inputs["Color"])
    scene.render.filepath = str(output / "occlusion.png")
    bpy.ops.render.render(write_still=True)
    geometry = nodes.new("ShaderNodeNewGeometry")
    scale = nodes.new("ShaderNodeVectorMath")
    scale.operation = "MULTIPLY_ADD"
    scale.inputs[1].default_value = (.5, .5, .5)
    scale.inputs[2].default_value = (.5, .5, .5)
    material.node_tree.links.new(geometry.outputs["Normal"], scale.inputs[0])
    material.node_tree.links.new(scale.outputs[0], emission.inputs["Color"])
    scene.render.filepath = str(output / "normal.png")
    bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    main()
