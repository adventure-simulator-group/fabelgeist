"""Check generated review JSON meshes against their saved, unposed body.

blender --background --python-exit-code 1 --python scripts/check_armor_construction.py
-- DIRECTORY REPORT.json
Every *--*.json in DIRECTORY is an independent candidate on body.json.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import numpy as np
from mathutils.bvhtree import BVHTree


def audit(path, body_tree):
    row = json.loads(path.read_text())
    points = np.asarray(row["positions"], dtype=float)
    faces = np.asarray(row["indices"], dtype=int).reshape(-1, 3)
    assert np.isfinite(points).all() and len(faces), "empty/nonfinite mesh"
    assert faces.min() >= 0 and faces.max() < len(points), "invalid indices"
    area = np.linalg.norm(np.cross(points[faces[:, 1]] - points[faces[:, 0]],
                                   points[faces[:, 2]] - points[faces[:, 0]]), axis=1)
    assert area.min() > 1e-12, "degenerate triangle"
    # Normal discontinuities split vertices; exact coincident positions retain
    # the physical wall connectivity without a tolerance that hides cracks.
    _, welded = np.unique(points, axis=0, return_inverse=True)
    physical = welded[faces]
    edges = np.concatenate([physical[:, [0, 1]], physical[:, [1, 2]], physical[:, [2, 0]]])
    directed, counts = np.unique(edges, axis=0, return_counts=True)
    reverse = np.unique(edges[:, ::-1], axis=0)
    assert np.array_equal(directed, reverse) and np.all(counts == 1), "open or inconsistently wound wall"
    tree = BVHTree.FromPolygons(points.tolist(), faces.tolist(), all_triangles=True)
    intersections = sum(a < b and not np.intersect1d(physical[a], physical[b]).size
                        for a, b in tree.overlap(tree))
    return dict(body_intersections=len(tree.overlap(body_tree)),
                self_intersections=int(intersections), triangles=len(faces),
                sha256=hashlib.sha256(path.read_bytes()).hexdigest())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("report", type=Path)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    body = json.loads((args.directory / "body.json").read_text())
    tree = BVHTree.FromPolygons(body["positions"], body["faces"], all_triangles=True)
    rows = {}
    for path in sorted(args.directory.glob("*--*.json")):
        rows[path.name] = audit(path, tree)
        print(path.name, rows[path.name], flush=True)
    assert rows, "no generated review meshes"
    passed = all(not r["body_intersections"] and not r["self_intersections"] for r in rows.values())
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(dict(passed=passed, candidates=rows), indent=2))
    if not passed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
