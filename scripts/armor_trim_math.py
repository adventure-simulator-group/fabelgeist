"""Physical plate-boundary distance and continuous pattern coordinates."""
from collections import defaultdict
import numpy as np


def boundary_field(positions, faces, segments):
    # Joining equal positions repairs UV/normal splits, never infers a rim.
    keys, inverse = np.unique(np.rint(np.asarray(positions, dtype=np.float64) * 1e6).astype(np.int64), axis=0, return_inverse=True)
    unique = keys / 1e6
    parent = np.arange(len(unique))
    def root(i):
        while parent[i] != i:
            parent[i] = parent[parent[i]]
            i = parent[i]
        return int(i)
    for face in inverse[faces]:
        a = root(face[0])
        for b in face[1:]:
            parent[root(b)] = a
    components = np.array([root(i) for i in range(len(unique))])
    lookup = {tuple(p): i for i, p in enumerate(keys)}
    ends = np.array([[lookup[tuple(p)] for p in edge] for edge in np.rint(np.asarray(segments, dtype=np.float32).astype(np.float64) * 1e6).astype(np.int64)])
    adjacency = defaultdict(list)
    for i, (a, b) in enumerate(ends):
        adjacency[a].append((b, i)); adjacency[b].append((a, i))
    if any(len(edges) != 2 for edges in adjacency.values()):
        raise ValueError("authored plate edges must form closed nonbranching rims")
    used, ordered, starts, lengths = set(), [], [], []
    for seed in range(len(ends)):
        if seed in used:
            continue
        first = int(ends[seed, 0]); vertex = first; loop = []
        while True:
            candidates = [(b, i) for b, i in adjacency[vertex] if i not in used]
            if not candidates:
                break
            next_vertex, index = candidates[0]
            used.add(index); loop.append((vertex, next_vertex)); vertex = next_vertex
        if vertex != first:
            raise ValueError("open plate rim")
        measures = [np.linalg.norm(unique[b] - unique[a]) for a, b in loop]
        total = sum(measures)
        distance = 0
        for edge, measure in zip(loop, measures):
            ordered.append(edge); starts.append(distance / total); lengths.append(measure / total)
            distance += measure
    ordered = np.array(ordered)
    return (unique[ordered], np.array(starts), np.array(lengths),
            components[ordered[:, 0]], components[inverse[faces[:, 0]]])


def pattern_mask(distance, phase, width, pattern):
    band = (distance >= 0) & (distance <= width)
    t = distance / width
    phase = phase % 1
    if pattern == "plain":
        return band
    if pattern == "double":
        return band & ((t < .25) | (t > .75))
    if pattern == "chevron":
        center = .2 + .6 * (1 - np.abs(phase * 2 - 1))
        return band & (np.abs(t - center) < .14)
    if pattern == "scallop":
        center = .2 + .6 * np.sqrt(np.maximum(0, 1 - (phase * 2 - 1) ** 2))
        return band & (np.abs(t - center) < .12)
    if pattern == "vine":
        stem = .5 + .13 * np.sin(phase * np.pi * 2)
        leaf_a = ((phase - .28) / .18) ** 2 + ((t - .27) / .19) ** 2 < 1
        leaf_b = ((phase - .73) / .18) ** 2 + ((t - .73) / .19) ** 2 < 1
        return band & ((np.abs(t - stem) < .05) | leaf_a | leaf_b)
    raise ValueError(f"unknown plate trim pattern {pattern}")


def surface_band_mask(points, bounds, band):
    """Evaluate a physical slab in the original mesh coordinate system.

    Signed transverse distance maps the whole stripe to the existing pattern,
    avoiding a mirrored/reset pattern at its center. Phase uses a perpendicular
    object-space axis, so neither coordinate resets at an atlas seam or lame.
    """
    axis = 'xyz'.index(band['axis'])
    phase_axis = 'xyz'.index(band.get('phase_axis', 'y' if axis != 1 else 'x'))
    lower, upper = bounds
    center = lower[axis] + band['position'] * (upper[axis] - lower[axis])
    width = band['width_mm'] / 1000
    distance = points[:, axis] - center + width / 2
    span = upper[phase_axis] - lower[phase_axis]
    phase = (points[:, phase_axis] - lower[phase_axis]) / max(float(span), 1e-12)
    return pattern_mask(distance, phase * band['repeats'], width, band['pattern'])
