"""Check actual skinned closures in unposed base and morph configurations.

blender --background --python-exit-code 1 --python scripts/check_fastener_assets.py
-- ASSETS BODY.glb REPORT.json [--only neutral] [--item cuisse--left]
Metal frames and tongues intentionally meet. Their internal metal/metal contacts
are not reported as cloth folds; leather must not intersect itself or any plate.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_underlayer_assets import Surface, configurations
from check_parametric_armor_assets import Glb, audit

HARDWARE = {"leather_straps", "buckles"}
NEIGHBORS = {"pauldron": ["rerebrace"], "spaulder": ["rerebrace"],
             "couter": ["rerebrace", "vambrace"], "poleyn": ["cuisse", "greave"],
             "sabaton": ["greave"], "mitten_gauntlet": ["vambrace"]}


def topology(surface):
    faces = surface.physical_faces
    edges = np.concatenate([faces[:, [0,1]], faces[:, [1,2]], faces[:, [2,0]]])
    directed, counts = np.unique(edges, axis=0, return_counts=True)
    assert np.array_equal(directed, np.unique(edges[:, ::-1], axis=0))
    assert np.all(counts == 1), "Open or inconsistently wound closure wall"


def check(state, body, pieces, selected=None):
    name, weights, proportions, pose = state
    assert not pose, "Fastener fit acceptance is unposed"
    body_tree = body.tree(body.deform(weights, proportions, {}))
    plates = {key: row["plate"].tree(row["plate"].deform(weights, proportions, {})) for key,row in pieces.items()}
    result = {}
    for key, row in pieces.items():
        if selected and key not in selected:
            continue
        leather = row["leather"]
        cloth = leather.tree(leather.deform(weights, proportions, {}))
        hardware = row["hardware"]
        tree = hardware.tree(hardware.deform(weights, proportions, {}))
        folds = sum(a < b and not np.intersect1d(leather.physical_faces[a], leather.physical_faces[b]).size
                    for a,b in cloth.overlap(cloth))
        item, side = key.split("--")
        neighbors = {other: len(tree.overlap(plates[f"{other}--{side}"]))
                     for other in NEIGHBORS.get(item, []) if f"{other}--{side}" in plates}
        result[key] = dict(body_intersections=len(tree.overlap(body_tree)),
                           leather_self_intersections=int(folds),
                           own_plate_intersections=len(tree.overlap(plates[key])),
                           neighbor_intersections=neighbors)
        if item in {"tassets", "spaulder"}:
            plate = row["plate"]
            attached = plates[key]
            result[key]["plate_body_intersections"] = len(attached.overlap(body_tree))
            result[key]["plate_self_intersections"] = int(sum(
                a < b and not np.intersect1d(plate.physical_faces[a], plate.physical_faces[b]).size
                for a,b in attached.overlap(attached)))
    passed = all(not r["body_intersections"] and not r["leather_self_intersections"]
                 and not r["own_plate_intersections"] and not any(r["neighbor_intersections"].values())
                 and not r.get("plate_body_intersections", 0) and not r.get("plate_self_intersections", 0)
                 for r in result.values())
    return dict(name=name, passed=passed, pieces=result)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("body", type=Path)
    parser.add_argument("report", type=Path)
    parser.add_argument("--only", action="append")
    parser.add_argument("--item", action="append")
    args = parser.parse_args(sys.argv[sys.argv.index("--")+1:])
    body = Surface(args.body)
    pieces = {}
    for path in sorted(args.assets.glob("*.glb")):
        names = {m["name"] for m in Glb(path).doc["meshes"]}
        if not HARDWARE <= names:
            continue
        audit(path)
        leather = Surface(path, {"leather_straps"})
        topology(leather)
        topology(Surface(path, {"buckles"}))
        pieces[path.stem] = dict(leather=leather, hardware=Surface(path, HARDWARE), plate=Surface(path, names-HARDWARE))
    assert pieces, "No closure assets"
    assert not args.item or set(args.item) <= pieces.keys(), "Unknown closure item"
    results = []
    for state in configurations():
        if args.only and state[0] not in args.only:
            continue
        row = check(state, body, pieces, args.item)
        results.append(row)
        print(f"{'PASS' if row['passed'] else 'FAIL'} {row['name']}", flush=True)
    assert results, "No matching unposed configurations"
    hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
              for p in [args.body, *sorted(args.assets.glob("*.glb"))]}
    args.report.write_text(json.dumps(dict(passed=all(r["passed"] for r in results), hashes=hashes,
        scope="Unposed sampled bodies; exact triangle tests, not a continuous parameter guarantee.",
        results=results), indent=2)+"\n")
    if not all(row["passed"] for row in results):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
