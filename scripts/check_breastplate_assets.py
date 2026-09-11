"""Check armor shells, component intersections and body clearance in Blender.

blender --background --python-exit-code 1 --python scripts/check_breastplate_assets.py -- ARMOR BODY REPORT
ARMOR/BODY may be review JSON for a neutral check or exported GLBs for a morph sweep.
Generated reports belong under target, not in Git.
"""
import argparse
import hashlib
import json
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_parametric_armor_assets import EXPECTED_TARGETS, Glb, audit

IDENTITY_BOUND = .35
PENETRATION_TOLERANCE_M = .001
RANDOM_SEED = 1640


def load(path):
    if path.suffix == '.json':
        row = json.loads(path.read_text())
        faces = np.asarray(row['faces'] if 'faces' in row else row['indices']).reshape(-1, 3)
        return np.asarray(row['positions']), faces, None
    glb = Glb(path)
    positions, faces, morphs = [], [], []
    offset = 0
    for node in glb.doc['nodes']:
        if 'mesh' in node:
            assert not any(key in node for key in ('matrix', 'translation', 'rotation', 'scale')), 'Expected reference-body component coordinates'
    for mesh in glb.doc['meshes']:
        assert mesh['extras']['targetNames'] == EXPECTED_TARGETS
        for primitive in mesh['primitives']:
            points = glb.array(primitive['attributes']['POSITION'])
            positions.append(points)
            faces.append(glb.array(primitive['indices']).reshape(-1, 3) + offset)
            morphs.append(np.stack([glb.array(t['POSITION']) for t in primitive['targets']]))
            offset += len(points)
    return np.concatenate(positions), np.concatenate(faces), np.concatenate(morphs, axis=1)


def configurations(morphs):
    zero = np.zeros(len(EXPECTED_TARGETS))
    yield 'neutral', zero.copy()
    if morphs is None:
        return
    for channel in range(45):
        for sign in [-1, 1]:
            weights = zero.copy()
            weights[channel] = sign * IDENTITY_BOUND
            yield f'identity-{channel:02}-{sign:+}', weights
    for name, values in [('positive', np.full(45, IDENTITY_BOUND)),
                         ('negative', np.full(45, -IDENTITY_BOUND)),
                         ('mixed', np.array([IDENTITY_BOUND if i % 2 else -IDENTITY_BOUND for i in range(45)]))]:
        yield name, np.r_[values, 0, 0]
    rng = np.random.default_rng(RANDOM_SEED)
    for index in range(16):
        yield f'corner-{index:02}', np.r_[rng.choice([-IDENTITY_BOUND, IDENTITY_BOUND], 45), 0, 0]


def tree(points, faces):
    return BVHTree.FromPolygons([Vector(p) for p in points], faces.tolist(), all_triangles=True)


def check(points, faces, welded_faces, body_points, body_faces):
    mesh_tree, body_tree = tree(points, faces), tree(body_points, body_faces)
    intersections = [(a, b) for a, b in mesh_tree.overlap(mesh_tree)
                     if a < b and not set(welded_faces[a]).intersection(welded_faces[b])]
    edges = np.unique(np.sort(np.concatenate([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]]), axis=1), axis=0)
    triangles = points[faces]
    samples = np.concatenate([points, triangles.mean(axis=1), points[edges].mean(axis=1)])
    minimum, inside, worst = float('inf'), 0, None
    for point in samples:
        hit, normal, _, distance = body_tree.find_nearest(Vector(point))
        signed = distance if (Vector(point) - hit).dot(normal) >= 0 else -distance
        if signed < minimum:
            minimum, worst = signed, point.tolist()
        inside += signed < -PENETRATION_TOLERANCE_M
    area = float(np.linalg.norm(np.cross(triangles[:, 1]-triangles[:, 0], triangles[:, 2]-triangles[:, 0]), axis=1).min() * .5)
    return {'passed': not intersections and not inside and area > 1e-12,
            'self_intersections': len(intersections), 'intersection_examples': intersections[:8],
            'body_triangle_intersections': len(mesh_tree.overlap(body_tree)),
            'samples_inside_by_over_1mm': inside, 'minimum_signed_distance_m': minimum,
            'minimum_triangle_area_m2': area, 'worst_point': worst}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('armor', type=Path)
    parser.add_argument('body', type=Path)
    parser.add_argument('report', type=Path)
    parser.add_argument('--only')
    args = parser.parse_args(sys.argv[sys.argv.index('--')+1:])
    if args.armor.suffix == '.glb':
        audit(args.armor)
    positions, faces, morphs = load(args.armor)
    body_positions, body_faces, body_morphs = load(args.body)
    assert morphs is None or body_morphs is not None
    _, welded = np.unique(np.round(positions, 6), axis=0, return_inverse=True)
    welded_faces = welded[faces]
    edges = defaultdict(list)
    for a, b, c in welded_faces:
        for pair in [(a,b), (b,c), (c,a)]:
            edges[tuple(sorted(pair))].append(pair)
    assert all(len(uses)==2 and uses[0]==uses[1][::-1] for uses in edges.values()), 'Physical closure/winding'
    rows = []
    for name, weights in configurations(morphs):
        if args.only and name != args.only:
            continue
        points = positions if morphs is None else positions + np.einsum('i,ijk->jk', weights, morphs)
        body = body_positions if body_morphs is None else body_positions + np.einsum('i,ijk->jk', weights, body_morphs)
        result = check(points, faces, welded_faces, body, body_faces)
        rows.append({'name': name, 'weights': weights.tolist(), **result})
        print(f"{name}: {result}", flush=True)
    assert rows, 'No configurations selected'
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps({'armor_sha256': hashlib.sha256(args.armor.read_bytes()).hexdigest(),
        'body_sha256': hashlib.sha256(args.body.read_bytes()).hexdigest(), 'seed': RANDOM_SEED,
        'limitations': 'Static identity morphs only; no skeletal poses/animation. Nearest-surface signs and finite samples do not prove continuous containment.',
        'configurations': len(rows), 'failures': sum(not r['passed'] for r in rows), 'samples': rows}, indent=2))
    assert all(r['passed'] for r in rows), 'Breastplate geometry/clearance failure; see report'


if __name__ == '__main__':
    main()
