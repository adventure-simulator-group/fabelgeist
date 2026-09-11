"""Author material UVs on exported equipment using Blender angle-based unwrap.

blender --background --python-exit-code 1 --python scripts/unwrap_armor.py -- DIRECTORY
Canonical anatomical TEXCOORD_0 remains untouched. The material atlas occupies
TEXCOORD_1. Textured body-conforming garments retain their authored UV layout.
"""
import argparse
import json
import math
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bpy
import numpy as np
from armor_glb import Asset, retains_body_uvs
from check_armor_uvs import overlap_pairs

RIM_ANGLE = math.radians(65)
ATLAS_MARGIN = 0.004


def seams(mesh):
    """Separate sharp plate rims, then slit curved panels on their rear half.

    The sagittal slit is +Z (rear) in the generator's Y-up, -Z-forward frame.
    Cylindrical arm pieces instead receive an underside slit along their axis.
    Disconnected lames and independent helmet components unwrap independently.
    """
    edges = {tuple(sorted(edge.vertices)): edge for edge in mesh.edges}
    adjacent = {key: [] for key in edges}
    positions = np.array([vertex.co[:] for vertex in mesh.vertices])
    lower, upper = positions.min(axis=0), positions.max(axis=0)
    center = (lower + upper) / 2
    arm = (upper - lower)[0] > (upper - lower)[1] * 1.4
    cut_axis, hidden_axis, hidden_sign = (2, 1, -1) if arm else (0, 2, 1)
    for face in mesh.polygons:
        for key in face.edge_keys:
            adjacent[tuple(sorted(key))].append(face)
    for key, edge in edges.items():
        faces = adjacent[key]
        sharp = len(faces) != 2 or faces[0].normal.angle(faces[1].normal, 0) > RIM_ANGLE
        # Separate front/back, lateral, and rim-facing zones. In particular a
        # rounded gorget rim must not remain a closed annulus inside one chart.
        zones = [(int(np.argmax(np.abs(face.normal))),
                  face.normal[int(np.argmax(np.abs(face.normal)))] > 0) for face in faces]
        transition = len(zones) == 2 and zones[0] != zones[1]
        # Cut the dual graph at the hidden meridian, following actual mesh edges.
        slit = len(faces) == 2 and (
            (faces[0].center[cut_axis] - center[cut_axis]) *
            (faces[1].center[cut_axis] - center[cut_axis]) <= 0
        ) and hidden_sign * (sum(positions[list(key), hidden_axis]) / 2 - center[hidden_axis]) >= 0
        edge.use_seam = bool(sharp or transition or slit)


def unwrap(positions, faces, normals):
    mesh = bpy.data.meshes.new("armor material chart")
    mesh.from_pydata(positions.tolist(), [], faces.tolist())
    mesh.update()
    seams(mesh)
    obj = bpy.data.objects.new(mesh.name, mesh)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    mesh.uv_layers.new(name="armor_material")
    # Each retry adds seams; the bounded solve fails rather than accepting an
    # atlas with folds. Changing a boundary can expose a neighboring fold.
    for attempt in range(8):
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.select_all(action="SELECT")
        bpy.ops.uv.unwrap(method="ANGLE_BASED", margin=ATLAS_MARGIN)
        bpy.ops.uv.average_islands_scale()
        bpy.ops.uv.pack_islands(rotate=True, margin=ATLAS_MARGIN)
        bpy.ops.object.mode_set(mode="OBJECT")
        coords = np.array([row.uv[:] for row in mesh.uv_layers.active.data]).reshape(-1, 3, 2)
        a, b = coords[:, 1] - coords[:, 0], coords[:, 2] - coords[:, 0]
        collapsed = np.abs(a[:, 0] * b[:, 1] - a[:, 1] * b[:, 0]) <= 2e-14
        collisions = overlap_pairs(coords)
        if not collapsed.any() and not collisions:
            break
        invalid = set(np.flatnonzero(collapsed)) | {face for pair in collisions for face in pair}
        print("splitting folded or collapsed rim charts", attempt, len(invalid), flush=True)
        # A thin corner/rim may collapse in the enclosing curved chart. Give
        # that physical face its own boundary and let the solver unwrap again.
        edges = {tuple(sorted(edge.vertices)): edge for edge in mesh.edges}
        for index in invalid:
            for key in mesh.polygons[int(index)].edge_keys:
                edges[tuple(sorted(key))].use_seam = True
    else:
        raise ValueError("material chart still folds after splitting invalid rim faces")
    for polygon in mesh.polygons:
        polygon.use_smooth = True
    mesh.normals_split_custom_set_from_vertices(normals.tolist())
    mesh.calc_tangents(uvmap="armor_material")
    sources, uv, tangents, indices, vertices = [], [], [], [], {}
    for loop in mesh.loops:
        coord = tuple(mesh.uv_layers.active.data[loop.index].uv)
        # Blender compresses custom normals internally. Orthogonalize to the
        # exact source normal exported to glTF, not that quantized surrogate.
        normal = normals[loop.vertex_index].astype(np.float64)
        normal /= np.linalg.norm(normal)
        direction = np.array(loop.tangent)
        direction -= normal * np.dot(normal, direction)
        direction /= np.linalg.norm(direction)
        tangent = (*direction, loop.bitangent_sign)
        key = (loop.vertex_index, coord, tangent)
        if key not in vertices:
            vertices[key] = len(sources)
            sources.append(loop.vertex_index)
            uv.append(coord)
            tangents.append(tangent)
        indices.append(vertices[key])
    bpy.data.objects.remove(obj, do_unlink=True)
    bpy.data.meshes.remove(mesh)
    return np.array(sources), np.array(indices), np.array(uv), np.array(tangents)


def process(path):
    if retains_body_uvs(path):
        return []
    asset, counts = Asset(path), []
    for mesh in asset.doc["meshes"]:
        for primitive in mesh["primitives"]:
            material = asset.doc.get("materials", [])[primitive["material"]]
            pbr = material.get("pbrMetallicRoughness", {})
            if "baseColorTexture" in pbr:
                continue  # Body-conforming mail/padding already has authored maps.
            attributes = primitive["attributes"]
            positions = asset.array(attributes["POSITION"])
            normals = asset.array(attributes["NORMAL"])
            faces = asset.array(primitive["indices"]).reshape(-1, 3)
            source, indices, uv, tangents = unwrap(positions, faces, normals)
            if not np.isfinite(uv).all() or uv.min() < -1e-5 or uv.max() > 1.00001:
                raise ValueError(f"invalid material UVs: {path} {mesh['name']}")
            triangles = uv[indices.reshape(-1, 3)]
            e, f = triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
            area = np.abs(e[:, 0] * f[:, 1] - e[:, 1] * f[:, 0]) / 2
            if area.min() <= 1e-14:
                raise ValueError(f"collapsed chart triangle: {path} {mesh['name']}")
            asset.remap(primitive, source, indices)
            attributes["TEXCOORD_1"] = asset.append(uv)
            attributes["TANGENT"] = asset.append(tangents)
            primitive.setdefault("extras", {})["adventuresim_material_uv"] = {
                "channel": 1, "unwrap": "ANGLE_BASED", "margin": ATLAS_MARGIN,
            }
            counts.append({"mesh": mesh["name"], "vertices": len(source),
                           "triangles": len(faces), "uv_area": float(area.sum())})
    if counts:
        asset.write(path)
    return counts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--only", action="append", help="Exact asset stem; repeatable")
    parser.add_argument("--report", type=Path)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    result = {}
    for path in sorted(args.directory.glob("*.glb")):
        if args.only and path.stem not in args.only:
            continue
        result[path.name] = process(path)
        print(path.name, result[path.name], flush=True)
    if not result:
        raise ValueError("no matching equipment GLBs")
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
