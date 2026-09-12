"""Automated exported-helmet morph checks, without rendering.

blender --background --python-exit-code 1 --python scripts/check_close_helmet_assets.py -- HELMET.glb BODY.glb REPORT.json
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_parametric_armor_assets import EXPECTED_TARGETS, SKELETAL_FIT_TARGETS, Glb, audit

IDENTITY_COUNT = 45
RUNTIME_BOUND = .35  # character_morph.rs: MAX_IDENTITY_VARIATION
PENETRATION_TOLERANCE_M = .001
RANDOM_SEED = 25397


class Mesh:
    def __init__(self, glb, mesh):
        assert mesh['extras']['targetNames'] == EXPECTED_TARGETS
        primitive, = mesh['primitives']
        self.positions = glb.array(primitive['attributes']['POSITION'])
        self.targets = np.stack([glb.array(t['POSITION']) for t in primitive['targets']])
        self.faces = glb.array(primitive['indices']).reshape(-1, 3)
        _, welded = np.unique(np.round(self.positions, 6), axis=0, return_inverse=True)
        self.physical_faces = welded[self.faces]
        self.edges = np.unique(np.sort(np.concatenate([
            self.faces[:, [0, 1]], self.faces[:, [1, 2]], self.faces[:, [2, 0]]]), axis=1), axis=0)
        parent = np.arange(len(self.positions))
        def root(i):
            while parent[i] != i:
                parent[i] = parent[parent[i]]
                i = parent[i]
            return i
        for a, b, c in self.physical_faces:
            parent[root(b)] = root(a)
            parent[root(c)] = root(a)
        labels = [root(face[0]) for face in self.physical_faces]
        mapping = {v: i for i, v in enumerate(dict.fromkeys(labels))}
        self.shells = np.array([mapping[v] for v in labels])

    def deform(self, weights):
        return self.positions + np.einsum('i,ijk->jk', weights, self.targets)

    def tree(self, points):
        return BVHTree.FromPolygons(points.tolist(), self.faces.tolist(), all_triangles=True)

    def samples(self, points):
        return np.concatenate([points, points[self.faces].mean(axis=1), points[self.edges].mean(axis=1)])

    def self_intersections(self, tree, role):
        counts = {'within_shell': 0, 'between_shells': 0, 'intentional_comb_bowl_join': 0}
        for a, b in tree.overlap(tree):
            if a >= b or np.intersect1d(self.physical_faces[a], self.physical_faces[b]).size:
                continue
            sa, sb = int(self.shells[a]), int(self.shells[b])
            # Catalog construction: bowl, embedded comb, then three nape lames.
            key = ('intentional_comb_bowl_join' if role == 'skull' and {sa, sb} == {0, 1}
                   else 'within_shell' if sa == sb else 'between_shells')
            counts[key] += 1
        return counts


def configurations():
    zero = np.zeros(len(EXPECTED_TARGETS))
    yield 'neutral', zero.copy(), 'runtime'
    for i in range(IDENTITY_COUNT):
        for sign in [-1, 1]:
            w = zero.copy()
            w[i] = sign * RUNTIME_BOUND
            yield f'identity-{i:02}-{sign:+}-bound', w, 'runtime'
    for i in range(len(EXPECTED_TARGETS)):
        w = zero.copy()
        w[i] = 1
        yield f'endpoint-{i:02}', w, 'runtime' if i >= IDENTITY_COUNT else 'basis'
    for i in range(IDENTITY_COUNT):
        w = zero.copy()
        w[i] = -1
        yield f'negative-unit-{i:02}', w, 'stress'
    corners = [np.full(45, RUNTIME_BOUND), np.full(45, -RUNTIME_BOUND),
               np.array([RUNTIME_BOUND if i % 2 else -RUNTIME_BOUND for i in range(45)])]
    for name, values in zip(['positive', 'negative', 'mixed'], corners):
        yield name, np.r_[values, np.zeros(len(SKELETAL_FIT_TARGETS))], 'runtime'
        for index, (target, _, _) in enumerate(SKELETAL_FIT_TARGETS, IDENTITY_COUNT):
            w = np.r_[values, np.zeros(len(SKELETAL_FIT_TARGETS))]
            w[index] = 1
            yield f'{name}-{target}', w, 'runtime'
    rng = np.random.default_rng(RANDOM_SEED)
    for i in range(32):
        yield f'random-{i:02}', np.r_[rng.uniform(-RUNTIME_BOUND, RUNTIME_BOUND, 45), np.zeros(len(SKELETAL_FIT_TARGETS))], 'runtime'
    for i in range(16):
        yield f'corner-{i:02}', np.r_[rng.choice([-RUNTIME_BOUND, RUNTIME_BOUND], 45), np.zeros(len(SKELETAL_FIT_TARGETS))], 'runtime'


def body_clearance(mesh, points, tree):
    samples = mesh.samples(points)
    minimum, inside, worst = float('inf'), 0, None
    for point in samples:
        p = Vector(point)
        hit, normal, _, distance = tree.find_nearest(p)
        signed = distance if (p - hit).dot(normal) >= 0 else -distance
        if signed < minimum:
            minimum, worst = signed, point.tolist()
        inside += signed < -PENETRATION_TOLERANCE_M
    return {'sample_count': len(samples), 'minimum_signed_distance_m': minimum,
            'samples_inside_by_over_tolerance': inside, 'worst_point': worst}


def check(name, weights, scope, plates, body):
    points = {role: mesh.deform(weights) for role, mesh in plates.items()}
    trees = {role: mesh.tree(points[role]) for role, mesh in plates.items()}
    pairs = {f'{a} vs {b}': len(trees[a].overlap(trees[b]))
             for a, b in [('skull', 'bevor'), ('skull', 'visor'), ('bevor', 'visor')]}
    internal = {role: mesh.self_intersections(trees[role], role) for role, mesh in plates.items()}
    # Body skeletal targets are zero: residual equipment morphs alone are not a posed wearer.
    body_tree = body.tree(body.deform(weights)) if not np.any(weights[45:]) else None
    clearance = {role: body_clearance(mesh, points[role], body_tree)
                 for role, mesh in plates.items()} if body_tree else None
    if clearance is not None:
        for role in plates:
            clearance[role]['body_triangle_intersections'] = len(trees[role].overlap(body_tree))
    areas = {}
    for role, mesh in plates.items():
        a, b, c = (points[role][mesh.faces[:, i]] for i in range(3))
        areas[role] = float(np.linalg.norm(np.cross(b - a, c - a), axis=1).min() * .5)
    failed = (any(pairs.values()) or any(v['within_shell'] or v['between_shells'] for v in internal.values())
              or min(areas.values()) <= 1e-12
              or (clearance is not None and any(v['samples_inside_by_over_tolerance'] for v in clearance.values())))
    return {'sample': name, 'scope': scope, 'weights': weights.tolist(), 'passed': not failed,
            'triangle_intersections': pairs, 'self_intersections': internal,
            'body_clearance': clearance, 'minimum_triangle_area_m2': areas}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('helmet', type=Path)
    parser.add_argument('body', type=Path)
    parser.add_argument('report', type=Path)
    parser.add_argument('--only', help='Run one named configuration for diagnosis')
    args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    audit(args.helmet)  # Physical closure, winding, morph seams, and skin bindings.
    glb, body_glb = Glb(args.helmet), Glb(args.body)
    plates = {m['name']: Mesh(glb, m) for m in glb.doc['meshes']}
    assert set(plates) == {'skull', 'bevor', 'visor'}
    skull = plates['skull']
    assert len(set(skull.shells)) == 5, 'Update shell-role expectations if catalog construction changes'
    comb_vertices = np.unique(skull.faces[skull.shells == 1])
    assert np.ptp(skull.positions[comb_vertices, 0]) < .01, 'Expected narrow sagittal comb'
    body_mesh, = body_glb.doc['meshes']
    body = Mesh(body_glb, body_mesh)
    results = []
    for name, weights, scope in configurations():
        if args.only and name != args.only:
            continue
        row = check(name, weights, scope, plates, body)
        results.append(row)
        print(f"{'PASS' if row['passed'] else 'FAIL'} {name} ({scope})", flush=True)
    assert results, 'No configurations selected'
    report = {'source': str(args.helmet), 'sha256': hashlib.sha256(args.helmet.read_bytes()).hexdigest(),
              'body': str(args.body), 'body_sha256': hashlib.sha256(args.body.read_bytes()).hexdigest(),
              'seed': RANDOM_SEED, 'runtime_identity_bound': RUNTIME_BOUND,
              'body_penetration_tolerance_m': PENETRATION_TOLERANCE_M,
              'limitations': 'Reference space, not skeletal animation. Body tests omit skeletal residual configurations. '
              'Body signs use nearest triangles at vertices, edge midpoints and face centroids, not continuous containment. '
              'Comb/bowl intersection is an intentional construction join; other shell pairs are tested. '
              'Basis +/-1 exceeds game identity variation bounds; failures there are diagnostic stress results.',
              'summary': {
                  'configurations': len(results),
                  'body_tested_configurations': sum(r['body_clearance'] is not None for r in results),
                  'runtime_configurations': sum(r['scope'] == 'runtime' for r in results),
                  'runtime_failures': [r['sample'] for r in results if r['scope'] == 'runtime' and not r['passed']],
                  'outside_runtime_failures': [r['sample'] for r in results if r['scope'] != 'runtime' and not r['passed']],
              },
              'samples': results}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    failures = [r['sample'] for r in results if not r['passed'] and r['scope'] == 'runtime']
    assert not failures, f'Runtime-range failures: {failures}'
    print(f'Completed {len(results)} configurations; runtime failures: {len(failures)}', flush=True)


if __name__ == '__main__':
    main()
