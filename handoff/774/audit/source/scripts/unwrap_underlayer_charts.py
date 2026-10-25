"""Unwrap the mail construction surface, preserving canonical body UVs as output.

blender --background --python scripts/unwrap_underlayer_charts.py -- MAIL_REVIEW_DIRECTORY CHARTS.json
Export every mail underlayer at maximum width and length into the directory.
"""
import heapq
import json
from pathlib import Path
import sys

import bpy
import numpy as np


def components(adjacency):
    remaining = set(adjacency)
    result = []
    while remaining:
        component, pending = set(), [remaining.pop()]
        while pending:
            vertex = pending.pop()
            component.add(vertex)
            for other in adjacency[vertex]:
                if other in remaining:
                    remaining.remove(other)
                    pending.append(other)
        result.append(component)
    return result


def sleeve_seams(mesh):
    """Connect sleeve cuff holes to the panel boundary on its upper surface."""
    edge_faces = {tuple(sorted(edge.vertices)): 0 for edge in mesh.edges}
    for face in mesh.polygons:
        for edge in face.edge_keys:
            edge_faces[tuple(sorted(edge))] += 1
    boundary, graph = {}, {}
    for edge in mesh.edges:
        a, b = edge.vertices
        graph.setdefault(a, []).append(b)
        graph.setdefault(b, []).append(a)
        if edge_faces[tuple(sorted((a, b)))] == 1:
            boundary.setdefault(a, []).append(b)
            boundary.setdefault(b, []).append(a)
    loops = components(boundary)
    edges = {tuple(sorted(edge.vertices)): edge for edge in mesh.edges}
    for surface in components(graph):
        rims = [rim for rim in loops if rim & surface]
        if len(rims) < 2:
            continue
        joined = rims.pop(max(range(len(rims)), key=lambda i: len(rims[i])))
        for rim in rims:
            # Highest point of a cuff favors a seam beneath the overlying plate.
            start = max(rim, key=lambda v: mesh.vertices[v].co.y)
            queue, costs, previous = [(0, start)], {start: 0}, {}
            finish = None
            while queue:
                cost, vertex = heapq.heappop(queue)
                if cost > costs[vertex]:
                    continue
                if vertex in joined:
                    finish = vertex
                    break
                for other in graph[vertex]:
                    step = (mesh.vertices[vertex].co - mesh.vertices[other].co).length
                    candidate = cost + step
                    if candidate < costs.get(other, float("inf")):
                        costs[other], previous[other] = candidate, vertex
                        heapq.heappush(queue, (candidate, other))
            if finish is None:
                raise RuntimeError("disconnected boundary path")
            while finish != start:
                prior = previous[finish]
                edges[tuple(sorted((prior, finish)))].use_seam = True
                finish = prior
            joined |= rim


def main():
    source, output = [Path(p).resolve() for p in sys.argv[sys.argv.index("--") + 1:]]
    paths = sorted(source.glob("mail*--*.json"))
    if not paths:
        raise ValueError("Mail review directory contains no mail meshes")
    positions, texcoords, faces = [], [], []
    for path in paths:
        row = json.loads(path.read_text())
        offset, count = len(positions), row["outer_vertex_count"]
        positions.extend(row["positions"][:count])
        texcoords.extend(row["texcoords"][:count])
        outer = np.asarray(row["indices"]).reshape(-1, 3)[:2 * row["outer_triangle_count"]:2]
        faces.extend((outer + offset).tolist())
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    mesh = bpy.data.meshes.new("Mail construction surface")
    mesh.from_pydata(positions, [], faces)
    mesh.update()
    canonical = mesh.uv_layers.new(name="mhr_body_v1")
    for loop in mesh.loops:
        canonical.data[loop.index].uv = texcoords[loop.vertex_index]
    obj = bpy.data.objects.new("Mail construction surface", mesh)
    bpy.context.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.remove_doubles(threshold=0.000001)
    bpy.ops.object.mode_set(mode="OBJECT")
    sleeve_seams(mesh)
    chart = mesh.uv_layers.new(name="Mail construction chart")
    mesh.uv_layers.active = chart
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.unwrap(method="ANGLE_BASED", margin=0.01, correct_aspect=False)
    bpy.ops.object.mode_set(mode="OBJECT")
    canonical = mesh.uv_layers["mhr_body_v1"]
    chart = mesh.uv_layers["Mail construction chart"]
    # Scale each separate panel by physical area. Unlike planar projection,
    # this preserves the metric of curved armpits and underside sleeves.
    adjacency = {v.index: [] for v in mesh.vertices}
    for edge in mesh.edges:
        a, b = edge.vertices
        adjacency[a].append(b)
        adjacency[b].append(a)
    scales = {}
    for group in components(adjacency):
        polygons = [face for face in mesh.polygons if face.vertices[0] in group]
        physical_area = sum(face.area for face in polygons)
        uv_area = 0
        for face in polygons:
            a, b, c = [np.array(chart.data[i].uv) for i in face.loop_indices]
            uv_area += abs(float(np.linalg.det(np.array([b-a, c-a])))) / 2
        if uv_area == 0:
            raise RuntimeError("mail unwrap has zero area")
        for face in polygons:
            scales[face.index] = (physical_area / uv_area)**0.5
    triangles = []
    for face in mesh.polygons:
        triangles.append({
            "positions": [list(mesh.vertices[v].co) for v in face.vertices],
            "normals": [list(mesh.vertices[v].normal) for v in face.vertices],
            "uv": [list(canonical.data[i].uv) for i in face.loop_indices],
            "chart": [(np.array(chart.data[i].uv) * scales[face.index]).tolist()
                      for i in face.loop_indices],
        })
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({"triangles": triangles, "source": str(source)}))


if __name__ == "__main__":
    main()
