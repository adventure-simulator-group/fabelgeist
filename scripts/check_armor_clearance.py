"""Audit selected armor against its body and neighbors in unposed morph states.

Run in Blender: --python-exit-code 1 --python scripts/check_armor_clearance.py --
ASSETS BODY.glb REPORT.json --item pauldron--left --item pauldron--right
Optional --neighbor NAME uses another asset from ASSETS; --only neutral limits
the run to one named configuration. Every default configuration is unposed.
"""
import argparse
import json
from pathlib import Path
import sys

import numpy as np
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_underlayer_assets import Surface, configurations, PENETRATION_TOLERANCE_M


def check(configuration, armor, body, neighbors):
    name, weights, proportions, pose = configuration
    assert not pose, "This audit accepts only unposed configurations"
    body_tree = body.tree(body.deform(weights, proportions, {}))
    trees, results = {}, {}
    for item, surface in armor.items():
        points = surface.deform(weights, proportions, {})
        tree = trees[item] = surface.tree(points)
        crossing = sum(a < b and not np.intersect1d(surface.physical_faces[a], surface.physical_faces[b]).size
                       for a, b in tree.overlap(tree))
        inside, minimum = 0, float("inf")
        for point in surface.samples(points):
            point = Vector(point)
            hit, normal, _, distance = body_tree.find_nearest(point)
            signed = distance if (point - hit).dot(normal) >= 0 else -distance
            minimum = min(minimum, signed)
            inside += signed < -PENETRATION_TOLERANCE_M
        results[item] = dict(body_intersections=len(tree.overlap(body_tree)),
                             self_intersections=int(crossing),
                             samples_inside_by_over_1mm=inside,
                             minimum_signed_distance_m=minimum)
    other_trees = {name: surface.tree(surface.deform(weights, proportions, {}))
                   for name, surface in neighbors.items()}
    pairs = {f"{item}/{other}": len(tree.overlap(other_tree))
             for item, tree in trees.items() for other, other_tree in other_trees.items()}
    items = list(trees)
    pairs.update({f"{a}/{b}": len(trees[a].overlap(trees[b]))
                  for i, a in enumerate(items) for b in items[i+1:]})
    passed = not any(pairs.values()) and not any(
        r["body_intersections"] or r["self_intersections"] or r["samples_inside_by_over_1mm"]
        for r in results.values())
    return dict(name=name, passed=passed, pieces=results, neighbor_intersections=pairs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("body", type=Path)
    parser.add_argument("report", type=Path)
    parser.add_argument("--item", action="append", required=True)
    parser.add_argument("--neighbor", action="append", default=[])
    parser.add_argument("--only", action="append")
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    armor = {item: Surface(args.assets / f"{item}.glb") for item in args.item}
    neighbors = {item: Surface(args.assets / f"{item}.glb") for item in args.neighbor}
    body = Surface(args.body)
    samples = [config for config in configurations() if not args.only or config[0] in args.only]
    if args.only and set(args.only) != {config[0] for config in samples}:
        parser.error("Unknown requested configuration")
    results = []
    for config in samples:
        row = check(config, armor, body, neighbors)
        results.append(row)
        print(config[0], "PASS" if row["passed"] else "FAIL", flush=True)
    report = dict(passed=all(row["passed"] for row in results), configurations=results,
                  scope="Sampled unposed identity and skeletal configurations; no continuous guarantee",
                  assets=str(args.assets.resolve()), body=str(args.body.resolve()))
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2))
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
