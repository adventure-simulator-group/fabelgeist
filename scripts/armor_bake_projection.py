"""Project a matching dense recipe mesh onto an unchanged native LOD surface."""
import hashlib
import json

import bpy
import numpy as np


def project(source_path, name, target, image):
    source = json.loads(source_path.read_text(encoding="utf-8"))
    components = source.get("components", [])
    if components:
        matches = [part for part in components if part["role"] == name]
        if len(matches) != 1:
            raise ValueError(f"{source_path}: expected one dense component {name}")
        part = matches[0]
        first, last = part["vertices"]["start"], part["vertices"]["end"]
        start, end = part["indices"]["start"], part["indices"]["end"]
    else:
        first, last = 0, len(source["positions"])
        start, end = 0, len(source["indices"])
    # Only the dense outer sheet carries surface detail. Interior faces and
    # narrow construction returns can intercept a coarse surface's bake rays.
    exterior = np.ones(len(source["indices"]) // 3, dtype=bool)
    for span in source["construction_faces"]:
        exterior[span["start"] // 3:span["end"] // 3] = False
    faces = np.asarray(source["indices"][start:end]).reshape(-1, 3) - first
    faces = faces[exterior[start // 3:end // 3]]
    mesh = bpy.data.meshes.new("Dense recipe bake source")
    mesh.from_pydata(source["positions"][first:last], [], faces.tolist())
    mesh.update()
    for polygon in mesh.polygons:
        polygon.use_smooth = True
    mesh.normals_split_custom_set_from_vertices(source["normals"][first:last])
    obj = bpy.data.objects.new(mesh.name, mesh)
    bpy.context.collection.objects.link(obj)
    material = target.data.materials[0]
    material.node_tree.nodes.active.image = image
    scene = bpy.context.scene
    bake = scene.render.bake
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    target.select_set(True)
    bpy.context.view_layer.objects.active = target
    bake.use_selected_to_active = True
    # Millimetre relief plus chord error on a native coarse plate. Each bake
    # source contains only the corresponding item component, never its neighbors.
    bake.cage_extrusion = 0.03
    bake.max_ray_distance = 0.06
    bake.normal_space = "TANGENT"
    try:
        bpy.ops.object.bake(type="NORMAL")
        pixels = np.asarray(image.pixels[:], dtype=np.float32).reshape(
            image.size[1], image.size[0], 4
        )[::-1].copy()
        return pixels, {
            "source_sha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
            "source_triangles": len(faces),
            "runtime_triangles": len(target.data.polygons),
            "projection_distance_m": bake.max_ray_distance,
        }
    finally:
        bake.use_selected_to_active = False
        bpy.data.objects.remove(obj, do_unlink=True)
        bpy.data.meshes.remove(mesh)
