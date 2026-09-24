"""Render actual recipe triangles against the exported MHR body with Blender.

blender --background --python scripts/render_armor_review.py -- INPUT_DIR OUTPUT_DIR [ID] [--bare-body]
"""
import json
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector


def mesh_object(name, positions, faces, color, normals):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([(p[0], -p[2], p[1]) for p in positions], [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.color = color
    for polygon in mesh.polygons:
        polygon.use_smooth = True
    mesh.normals_split_custom_set_from_vertices([(n[0], -n[2], n[1]) for n in normals])
    return obj


def main():
    args = sys.argv[sys.argv.index("--") + 1:]
    bare_body = "--bare-body" in args
    args = [arg for arg in args if arg != "--bare-body"]
    source, output = [Path(p).resolve() for p in args[:2]]
    output.mkdir(parents=True, exist_ok=True)
    selected = args[2] if len(args) > 2 else None
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = 600
    scene.render.resolution_y = 700
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    shading = scene.display.shading
    shading.light = "STUDIO"
    shading.studiolight_rotate_z = math.radians(25)
    shading.color_type = "OBJECT"
    shading.show_shadows = True
    shading.show_cavity = True
    shading.cavity_type = "BOTH"
    shading.show_specular_highlight = True
    shading.show_backface_culling = True
    shading.background_type = "WORLD"
    scene.world.color = (0.12, 0.13, 0.15)
    body = json.loads((source / "body.json").read_text())
    mesh_object("MHR body", body["positions"], body["faces"], (0.49, 0.32, 0.25, 1), body["normals"])
    camera_data = bpy.data.cameras.new("Review camera")
    camera = bpy.data.objects.new("Review camera", camera_data)
    bpy.context.collection.objects.link(camera)
    scene.camera = camera
    camera_data.type = "ORTHO"
    for path in sorted(source.glob("*--*.json")):
        row = json.loads(path.read_text())
        if selected and row["id"] not in selected.split(","):
            continue
        positions = row["positions"]
        faces = [row["indices"][i:i + 3] for i in range(0, len(row["indices"]), 3)]
        components = row.get("components", [])
        if components:
            armor = []
            for part in components:
                start, end = part["vertices"]["start"], part["vertices"]["end"]
                first, last = part["indices"]["start"] // 3, part["indices"]["end"] // 3
                component_faces = [[index - start for index in face] for face in faces[first:last]]
                surface = part.get("material")
                color = surface["base_color"] if surface else (0.56, 0.62, 0.68, 1)
                obj = mesh_object(f"{row['id']}.{part['role']}", positions[start:end], component_faces,
                                  color, row["normals"][start:end])
                if part.get("hinge"):
                    obj["reference_body_hinge"] = json.dumps(part["hinge"])
                armor.append(obj)
        else:
            armor = [mesh_object(row["id"], positions, faces, (0.56, 0.62, 0.68, 1), row["normals"])]
        low = Vector([min(p[i] for p in positions) for i in range(3)])
        high = Vector([max(p[i] for p in positions) for i in range(3)])
        center = (low + high) * 0.5
        extent = high - low
        # Keep surrounding anatomy visible while framing the equipment large enough.
        scale = max(extent.y * 1.45, extent.x * 1.6, extent.z * 1.6, 0.32)
        lateral = -1 if row["placement"] == "right" else 1
        for name, direction in [("front", (0, .05, 1)), ("side", (lateral, .06, 0)),
                                ("quarter", (.7, .24, 1)), ("rear", (0, .08, -1))]:
            target = Vector((center.x, -center.z, center.y))
            camera.location = target + Vector((direction[0], -direction[2], direction[1])).normalized() * 3
            camera.rotation_euler = (target - camera.location).to_track_quat("-Z", "Y").to_euler()
            camera_data.ortho_scale = scale
            scene.render.filepath = str(output / f"{path.stem}-{name}.png")
            bpy.ops.render.render(write_still=True)
            if bare_body:
                for obj in armor:
                    obj.hide_render = True
                scene.render.filepath = str(output / f"{path.stem}-{name}-body.png")
                bpy.ops.render.render(write_still=True)
                for obj in armor:
                    obj.hide_render = False
        for obj in armor:
            bpy.data.objects.remove(obj, do_unlink=True)
    (output / "render-settings.json").write_text(json.dumps({
        "engine": scene.render.engine, "size": [600, 700], "body": str(source / "body.json"),
        "views": ["front", "side", "quarter", "rear"], "body_included": True, "normals": "preserved exported runtime vertex normals",
        "matched_bare_body_views": bare_body,
        "stage": "ordinary generator output; external static render"
    }, indent=2))


if __name__ == "__main__":
    main()
