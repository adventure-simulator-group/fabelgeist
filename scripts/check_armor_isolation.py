"""Check physical mesh islands in a posed museum assembly for isolation.

blender --background --python-exit-code 1 --python scripts/check_armor_isolation.py
-- PRESET.json REPORT.json [--max-gap-mm 25]

Evaluates node transforms and the saved pose. UV/normal seams are quantized to
a one-micrometre grid; each disconnected island is compared with all other geometry,
including the wearer. A passing proximity check does not prove attachment,
clearance, or collision-free articulation. Units are metres internally.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import render_museum_armor as renderer


def islands(points, faces):
    """Retain disconnected pieces even when they share one object/material."""
    _, inverse = np.unique(np.round(points, 6), axis=0, return_inverse=True)
    parent = list(range(int(inverse.max()) + 1))

    def root(i):
        while parent[i] != i:
            parent[i] = parent[parent[i]]
            i = parent[i]
        return i

    for face in inverse[faces]:
        for i in face[1:]:
            parent[root(int(i))] = root(int(face[0]))
    groups = {}
    for face in faces:
        groups.setdefault(root(int(inverse[face[0]])), []).append(face)
    for selected in groups.values():
        selected = np.asarray(selected)
        used, remap = np.unique(selected, return_inverse=True)
        yield points[used], remap.reshape(-1, 3)


class Surface:
    def __init__(self, points, faces):
        self.points = np.asarray(points, dtype=float)
        self.faces = np.asarray(faces, dtype=int)
        if not len(self.faces) or not np.isfinite(self.points).all():
            raise ValueError("empty or nonfinite mesh")
        self.tree = BVHTree.FromPolygons(self.points.tolist(), self.faces.tolist(), all_triangles=True)
        edges = np.concatenate([self.faces[:, [0, 1]], self.faces[:, [1, 2]], self.faces[:, [2, 0]]])
        self.edges = self.points[np.unique(np.sort(edges, axis=1), axis=0)]
        self.low, self.high = self.points.min(axis=0), self.points.max(axis=0)


def segment_distances(edge, others):
    """Closest distances between closed segments, including parallel edges."""
    p, q = edge
    u = q - p
    v = others[:, 1] - others[:, 0]
    r = p - others[:, 0]
    a = u @ u
    e = np.einsum("ij,ij->i", v, v)
    valid = e > 1e-24
    v, r, e = v[valid], r[valid], e[valid]
    if a <= 1e-24 or not len(e):
        return np.asarray([np.inf])
    b, c = v @ u, r @ u
    f = np.einsum("ij,ij->i", v, r)
    denominator = a * e - b * b
    s = np.clip(np.divide(b * f - c * e, denominator,
                         out=np.zeros_like(e), where=denominator > 1e-24), 0, 1)
    t = (b * s + f) / e
    s = np.where(t < 0, np.clip(-c / a, 0, 1), s)
    s = np.where(t > 1, np.clip((b - c) / a, 0, 1), s)
    t = np.clip(t, 0, 1)
    return np.linalg.norm(r + s[:, None] * u - t[:, None] * v, axis=1)


def separation(a, b, threshold):
    """Exact triangle distance for failures; early exit once proximity is proven."""
    best = float("inf")
    for source, target in [(a, b), (b, a)]:
        for p in source.points:
            best = min(best, target.tree.find_nearest(Vector(p))[3])
            if best <= threshold:
                return best
    # Vertex-face distance alone misses closest edge interiors and piercings.
    other_low, other_high = b.edges.min(axis=1), b.edges.max(axis=1)
    for edge in a.edges:
        direction = Vector(edge[1] - edge[0])
        length = direction.length
        if length and b.tree.ray_cast(Vector(edge[0]), direction / length, length)[0] is not None:
            return 0.0
        gap = np.maximum(np.maximum(other_low - edge.max(axis=0), edge.min(axis=0) - other_high), 0)
        candidates = b.edges[np.einsum("ij,ij->i", gap, gap) < best * best]
        if len(candidates):
            best = min(best, float(segment_distances(edge, candidates).min()))
            if best <= threshold:
                return best
    # A triangle of b can pierce the interior of a without an a-edge crossing.
    for edge in b.edges:
        direction = Vector(edge[1] - edge[0])
        length = direction.length
        if length and a.tree.ray_cast(Vector(edge[0]), direction / length, length)[0] is not None:
            return 0.0
    return best


def audit(named, threshold):
    rows = []
    for i, (name, surface) in enumerate(named):
        best, neighbor = float("inf"), None
        candidates = []
        for j, (_, other) in enumerate(named):
            if i != j:
                gap = np.maximum(np.maximum(other.low - surface.high, surface.low - other.high), 0)
                candidates.append((float(np.linalg.norm(gap)), j))
        for bound, j in sorted(candidates):
            if bound >= best:
                break
            distance = separation(surface, named[j][1], threshold)
            if distance < best:
                best, neighbor = distance, named[j][0]
            if best <= threshold:
                break
        rows.append(dict(island=name, neighbor=neighbor,
                         gap_mm=best * 1000 if np.isfinite(best) else None,
                         isolated=best > threshold))
    return rows


def load_surfaces(preset):
    """Evaluate real exported meshes, transforms, skin and disconnected shells."""
    import bpy
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    objects = []
    names = {}
    for item in preset.items:
        imported = renderer.import_glb(item.path) if item.path.suffix == ".glb" else renderer.import_review(item.path, preset)
        objects.extend(imported)
        names.update({o: item.name + "/" + o.name for o in imported})
    renderer.apply_pose(objects, preset.pose)
    bpy.context.view_layer.update()
    graph = bpy.context.evaluated_depsgraph_get()
    named = []
    for obj in objects:
        if obj.type != "MESH":
            continue
        evaluated = obj.evaluated_get(graph)
        mesh = evaluated.to_mesh()
        mesh.calc_loop_triangles()
        points = np.asarray([tuple(evaluated.matrix_world @ v.co) for v in mesh.vertices])
        faces = np.asarray([tuple(t.vertices) for t in mesh.loop_triangles])
        for i, (p, f) in enumerate(islands(points, faces)):
            named.append((f"{names[obj]}#{i}", Surface(p, f)))
        evaluated.to_mesh_clear()
    if not named:
        raise ValueError("assembly has no mesh islands")
    return named


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("preset", type=Path)
    parser.add_argument("report", type=Path)
    parser.add_argument("--source-root", type=Path)
    parser.add_argument("--max-gap-mm", type=float, default=25.0)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    if not 0 < args.max_gap_mm < float("inf"):
        parser.error("max-gap-mm must be positive and finite")
    preset = renderer.Preset.load(args.preset, args.source_root)
    named = load_surfaces(preset)
    rows = audit(named, args.max_gap_mm / 1000)
    passed = not any(r["isolated"] for r in rows)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(dict(passed=passed, max_gap_mm=args.max_gap_mm,
        preset=dict(path=str(args.preset), sha256=hashlib.sha256(args.preset.read_bytes()).hexdigest(), pose=preset.pose),
        interpretation="Pass distances are upper bounds; isolated distances are surface minima.",
        inputs=[dict(path=str(i.path), sha256=hashlib.sha256(i.path.read_bytes()).hexdigest()) for i in preset.items],
        islands=rows), indent=2) + "\n")
    print(json.dumps(dict(passed=passed, islands=len(rows), isolated=[r for r in rows if r["isolated"]])))
    if not passed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
