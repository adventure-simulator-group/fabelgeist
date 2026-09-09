"""Objective triangle and sampled body-clearance evidence from actual candidates.

Run with Blender --background --python ... -- CANDIDATE_DIRECTORY.
Nearest-surface signs are diagnostics, not continuous collision guarantees.
"""
import json
import sys
from collections import defaultdict
from pathlib import Path

from mathutils import Vector
from mathutils.bvhtree import BVHTree

root = Path(sys.argv[sys.argv.index("--") + 1]).resolve()
body = json.loads((root / "body.json").read_text())
tree = BVHTree.FromPolygons([Vector(p) for p in body["positions"]], body["faces"], all_triangles=True)
results = []
for path in sorted(root.glob("*--*.json")):
    mesh = json.loads(path.read_text())
    positions = [Vector(p) for p in mesh["positions"]]
    indices = mesh["indices"]
    edges = defaultdict(list)
    volume = 0.0
    minimum_area = float("inf")
    for i in range(0, len(indices), 3):
        a, b, c = indices[i:i + 3]
        p, q, r = [positions[v] for v in (a, b, c)]
        minimum_area = min(minimum_area, (q - p).cross(r - p).length * .5)
        volume += p.dot(q.cross(r)) / 6
        for edge in ((a, b), (b, c), (c, a)):
            edges[tuple(sorted(edge))].append(edge)
    wrong_edges = sum(len(uses) != 2 or uses[0] != uses[1][::-1] for uses in edges.values())
    # Duplicate vertices at a UV/hard-normal seam still form one physical wall.
    welded = {}
    mapping = []
    for point in positions:
        key = tuple(round(value, 6) for value in point)
        mapping.append(welded.setdefault(key, len(welded)))
    physical_edges = defaultdict(list)
    for i in range(0, len(indices), 3):
        a, b, c = [mapping[index] for index in indices[i:i + 3]]
        for edge in ((a, b), (b, c), (c, a)):
            physical_edges[tuple(sorted(edge))].append(edge)
    wrong_physical_edges = sum(len(uses) != 2 or uses[0] != uses[1][::-1]
                               for uses in physical_edges.values())
    distances = []
    for p in positions:
        hit, normal, _, distance = tree.find_nearest(p)
        distances.append(distance if (p - hit).dot(normal) >= 0 else -distance)
    # Vertices alone can miss a body bulge through the middle of a coarse face.
    # Add face centroids and unique edge midpoints as bounded diagnostics.
    interior_samples = [(positions[indices[i]] + positions[indices[i + 1]] + positions[indices[i + 2]]) / 3
                        for i in range(0, len(indices), 3)]
    interior_samples.extend((positions[a] + positions[b]) / 2 for a, b in edges)
    interior_distances = []
    for point in interior_samples:
        hit, normal, _, distance = tree.find_nearest(point)
        interior_distances.append(distance if (point - hit).dot(normal) >= 0 else -distance)
    results.append({"id": path.stem, "vertices": len(positions), "triangles": len(indices) // 3,
                    "invalid_closed_edges": wrong_edges, "invalid_physical_edges": wrong_physical_edges, "minimum_area_m2": minimum_area,
                    "signed_material_volume_m3": volume,
                    "sampled_vertices_inside_body_by_over_1mm": sum(d < -.001 for d in distances),
                    "minimum_sampled_signed_distance_m": min(distances),
                    "interior_samples": len(interior_samples),
                    "interior_samples_inside_body_by_over_1mm": sum(d < -.001 for d in interior_distances),
                    "minimum_interior_sampled_signed_distance_m": min(interior_distances)})
(root / "mesh-checks.json").write_text(json.dumps({
    "method": "Every generated vertex, face centroid and edge midpoint nearest to full body triangles; signs are local diagnostics, not continuous containment proof.",
    "items": results}, indent=2))
print(json.dumps(results, indent=2))
