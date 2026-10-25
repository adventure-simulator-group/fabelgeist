"""Check exported boots against the supported leg garments in reference space.

blender --background --python scripts/check_boot_layering.py -- ASSETS OUTPUT.json
Checks neutral, all morph endpoints and three identity blends. Triangle
intersection tests do not establish clearance under skeletal animation.
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

import numpy as np
from mathutils.bvhtree import BVHTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_parametric_armor_assets import EXPECTED_TARGETS, SKELETAL_FIT_TARGETS, Glb


def read(path):
    glb = Glb(path)
    mesh, = glb.doc["meshes"]
    primitive, = mesh["primitives"]
    assert mesh["extras"]["targetNames"] == EXPECTED_TARGETS, path
    return {
        "positions": glb.array(primitive["attributes"]["POSITION"]),
        "faces": glb.array(primitive["indices"]).reshape(-1, 3),
        "targets": np.stack([glb.array(t["POSITION"]) for t in primitive["targets"]]),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    rows = {
        f"{kind}--{side}": read(args.assets / f"{kind}--{side}.glb")
        for side in ("left", "right")
        for kind in ("leather_boot", "mail_chausses", "padded_chausses")
    }
    samples = [("neutral", np.zeros(len(EXPECTED_TARGETS)))]
    samples.extend((f"endpoint-{i}", weights) for i, weights in enumerate(np.eye(len(EXPECTED_TARGETS))))
    samples.extend([
        ("positive", np.r_[np.full(45, .35), np.zeros(len(SKELETAL_FIT_TARGETS))]),
        ("negative", np.r_[np.full(45, -.35), np.zeros(len(SKELETAL_FIT_TARGETS))]),
        ("mixed", np.r_[[.35 if i % 2 else -.35 for i in range(45)], np.zeros(len(SKELETAL_FIT_TARGETS))]),
    ])
    results = []
    for name, weights in samples:
        trees = {}
        for key, row in rows.items():
            positions = row["positions"] + np.einsum("i,ijk->jk", weights, row["targets"])
            trees[key] = BVHTree.FromPolygons(
                positions.tolist(), row["faces"].tolist(), all_triangles=True)
        pairs = {}
        for side in ("left", "right"):
            boot = trees[f"leather_boot--{side}"]
            for garment in ("mail_chausses", "padded_chausses"):
                pairs[f"{side} vs {garment}"] = len(boot.overlap(trees[f"{garment}--{side}"]))
        results.append({"sample": name, "weights": weights.tolist(), "pairs": pairs})
        print(name, pairs, flush=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({
        "stage": "Actual exported reference-space morph triangles; no skeletal animation",
        "hashes": {key: row["sha256"] for key, row in rows.items()},
        "samples": results,
    }, indent=2))
    assert all(not count for result in results for count in result["pairs"].values()), \
        f"Boot/legging intersections found; see {args.output}"


if __name__ == "__main__":
    main()
