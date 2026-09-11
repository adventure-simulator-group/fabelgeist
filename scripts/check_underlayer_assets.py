"""Check actual underlayer GLBs against the body and each other across morphs.

blender --background --python-exit-code 1 --python scripts/check_underlayer_assets.py -- ASSETS BODY.glb REPORT.json
Uses every vertex, edge midpoint and triangle centroid for signed clearance,
plus triangle intersections. These are sampled configurations, not continuous
guarantees for all possible bodies or motions. Skeletal residuals are evaluated
together with their authored bone translations, never against an unmoved body.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import numpy as np
from mathutils import Matrix, Quaternion, Vector
from mathutils.bvhtree import BVHTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_parametric_armor_assets import EXPECTED_TARGETS, Glb, audit

IDENTITY_BOUND = 0.35
# CharacterProportions' generated range is narrower than its editor limits.
GENERATED_PROPORTION_FRACTION = 0.35
PENETRATION_TOLERANCE_M = 0.001
ITEMS = ["arming_doublet--worn", "padded_chausses--left",
         "padded_chausses--right", "mail_voiders--worn", "mail_brayette--worn",
         "mail_knee_voider--left", "mail_knee_voider--right", "mail_standard--worn"]


class Surface:
    def __init__(self, path):
        self.glb = glb = Glb(path)
        mesh, = glb.doc["meshes"]
        primitive, = mesh["primitives"]
        assert mesh["extras"]["targetNames"] == EXPECTED_TARGETS
        attributes = primitive["attributes"]
        self.positions = glb.array(attributes["POSITION"])
        self.faces = glb.array(primitive["indices"]).reshape(-1, 3)
        self.targets = np.stack([glb.array(t["POSITION"]) for t in primitive["targets"]])
        self.joints = glb.array(attributes["JOINTS_0"])
        self.weights = glb.array(attributes["WEIGHTS_0"])
        self.edges = np.unique(np.sort(np.concatenate([self.faces[:, [0, 1]],
            self.faces[:, [1, 2]], self.faces[:, [2, 0]]]), axis=1), axis=0)
        _, welded = np.unique(np.round(self.positions, 6), axis=0, return_inverse=True)
        self.physical_faces = welded[self.faces]

    def deform(self, weights, proportions, pose):
        points = self.positions + np.einsum("i,ijk->jk", weights, self.targets)
        skin = self.glb.doc["skins"][0]
        inverse = self.glb.array(skin["inverseBindMatrices"]).reshape(-1, 4, 4).transpose(0, 2, 1)
        parents = {child: index for index, node in enumerate(self.glb.doc["nodes"])
                   for child in node.get("children", [])}
        globals = {}
        for index in skin["joints"]:
            node = self.glb.doc["nodes"][index]
            translation = np.array(node.get("translation", [0, 0, 0]), dtype=float)
            basis = node.get("extras", {}).get("adventuresim_proportions")
            if basis:
                translation += (proportions - np.array(basis["reference"])) @ np.array(basis["translation_metres"])
            x, y, z, w = node.get("rotation", [0, 0, 0, 1])
            rotation = Quaternion((w, x, y, z))
            if node["name"] in pose:
                axis, degrees = pose[node["name"]]
                rotation = rotation @ Quaternion(Vector(axis), np.radians(degrees))
            local = Matrix.LocRotScale(Vector(translation), rotation, Vector(node.get("scale", [1, 1, 1])))
            globals[index] = globals.get(parents.get(index), Matrix.Identity(4)) @ local
        transforms = np.array([globals[index] for index in skin["joints"]]) @ inverse
        if "global_frame" in pose:
            observed = {bone["name"]: bone for bone in pose["global_frame"]["bones"]}
            matrices = []
            for index in skin["joints"]:
                bone = observed[self.glb.doc["nodes"][index]["name"]]
                x, y, z, w = bone["rotation_xyzw"]
                matrices.append(Matrix.LocRotScale(Vector(bone["translation"]),
                    Quaternion((w, x, y, z)), Vector(bone["scale"])))
            transforms = np.array(matrices) @ inverse
        homogeneous = np.column_stack((points, np.ones(len(points))))
        result = np.zeros_like(homogeneous)
        for influence in range(4):
            result += np.einsum("nij,nj->ni", transforms[self.joints[:, influence]], homogeneous) * self.weights[:, influence, None]
        # Bevy passes world_position.xyz to projection with a fresh w=1.
        # Do not normalize here: that would hide under-summed exported weights.
        return result[:, :3]

    def tree(self, points):
        return BVHTree.FromPolygons(points.tolist(), self.faces.tolist(), all_triangles=True)

    def samples(self, points):
        return np.concatenate([points, points[self.faces].mean(axis=1), points[self.edges].mean(axis=1)])


def configurations():
    zero = np.zeros(47)
    proportions = np.zeros(9)
    yield "neutral", zero.copy(), proportions.copy(), {}
    for i in range(45):
        for sign in [-1, 1]:
            weights = zero.copy()
            weights[i] = sign * IDENTITY_BOUND
            yield f"identity-{i:02}-{sign:+}", weights, proportions.copy(), {}
    for i in range(47):
        weights, shape = zero.copy(), proportions.copy()
        weights[i] = 1
        if i >= 45:
            shape[6] = -1.1 if i == 45 else 1.1
        yield f"basis-{i:02}", weights, shape, {}
    for name, values in [("positive", np.full(45, IDENTITY_BOUND)),
                         ("negative", np.full(45, -IDENTITY_BOUND)),
                         ("mixed", np.array([IDENTITY_BOUND if i % 2 else -IDENTITY_BOUND for i in range(45)]))]:
        yield name, np.r_[values, 0, 0], proportions.copy(), {}
    rng = np.random.default_rng(71390)
    for i in range(12):
        yield f"blend-{i:02}", np.r_[rng.uniform(-IDENTITY_BOUND, IDENTITY_BOUND, 45), 0, 0], proportions.copy(), {}
    for index, limit in enumerate([.5, .2, 1, 1, .5, 1, 1.1, .4, .1]):
        for sign in [-1, 1]:
            weights, shape = zero.copy(), proportions.copy()
            shape[index] = sign * limit * GENERATED_PROPORTION_FRACTION
            if index == 6:
                weights[45 if sign < 0 else 46] = GENERATED_PROPORTION_FRACTION
            yield f"generated-proportion-{index}-{sign:+}", weights, shape, {}
            weights, shape = zero.copy(), proportions.copy()
            shape[index] = sign * limit
            if index == 6:
                weights[45 if sign < 0 else 46] = 1
            yield f"proportion-{index}-{sign:+}", weights, shape, {}


def check(name, weights, proportions, pose, surfaces):
    points = {key: surface.deform(weights, proportions, pose) for key, surface in surfaces.items()}
    trees = {key: surface.tree(points[key]) for key, surface in surfaces.items()}
    results = {}
    touched_body_faces = set()
    for key in ITEMS:
        surface = surfaces[key]
        inside, minimum = 0, float("inf")
        for point in surface.samples(points[key]):
            point = Vector(point)
            hit, normal, _, distance = trees["body"].find_nearest(point)
            signed = distance if (point - hit).dot(normal) >= 0 else -distance
            minimum = min(minimum, signed)
            inside += signed < -PENETRATION_TOLERANCE_M
        crossing = sum(a < b and not np.intersect1d(surface.physical_faces[a], surface.physical_faces[b]).size
                       for a, b in trees[key].overlap(trees[key]))
        body_crossings = trees[key].overlap(trees["body"])
        touched_body_faces.update(b for _, b in body_crossings)
        results[key] = {"body_intersections": len(body_crossings),
                        "self_intersections": int(crossing), "minimum_signed_distance_m": minimum,
                        "samples_inside_by_over_1mm": inside}
    pairs = {f"{a}/{b}": len(trees[a].overlap(trees[b]))
             for i, a in enumerate(ITEMS) for b in ITEMS[i+1:]}
    passed = not any(pairs.values()) and not any(r["self_intersections"] or r["body_intersections"]
              or r["samples_inside_by_over_1mm"] for r in results.values())
    body_faces = surfaces["body"].physical_faces
    body_crossings = [(a, b) for a, b in trees["body"].overlap(trees["body"])
                     if a < b and not np.intersect1d(body_faces[a], body_faces[b]).size]
    # Source defects explain fitting failures; they never waive garment checks.
    source_body = {"self_intersections": len(body_crossings),
                   "self_intersections_touching_garment_crossings": sum(
                       a in touched_body_faces or b in touched_body_faces
                       for a, b in body_crossings)}
    return {"name": name, "passed": passed, "pieces": results, "layer_intersections": pairs,
            "source_body": source_body}


def capture_configuration(pose_trace):
    """Read the actual wearer shape recorded beside a runtime bone trace."""
    readiness = json.loads((pose_trace.parent / "armor-readiness.json").read_text())
    observed = [np.asarray(part["morph_weights"], dtype=float)
                for row in readiness for part in row["parts"]]
    if not observed or any(weights.shape != (len(EXPECTED_TARGETS),)
                           or not np.isfinite(weights).all() for weights in observed):
        raise ValueError("Capture readiness must contain complete finite morph weights")
    if any(not np.array_equal(weights, observed[0]) for weights in observed[1:]):
        raise ValueError("Capture pieces disagree on the wearer's morph weights")
    proportions = np.asarray(json.loads(
        (pose_trace.parent / "body-proportions.json").read_text()), dtype=float)
    if proportions.shape != (9,) or not np.isfinite(proportions).all():
        raise ValueError("Capture must provide nine finite body proportions")
    return observed[0], proportions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("body", type=Path)
    parser.add_argument("report", type=Path)
    parser.add_argument("--only", action="append", help="Named configuration; repeat to check a focused set")
    parser.add_argument("--pose-trace", type=Path, help="Actual runtime global-bone-transforms.jsonl")
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    surfaces = {"body": Surface(args.body)}
    for name in ITEMS:
        path = args.assets / f"{name}.glb"
        audit(path)
        surfaces[name] = Surface(path)
    results = []
    samples = configurations()
    if args.pose_trace:
        weights, proportions = capture_configuration(args.pose_trace)
        frames = [json.loads(line) for line in args.pose_trace.read_text().splitlines()]
        selected = np.unique(np.linspace(0, len(frames)-1, 8).round().astype(int))
        samples = [(f"runtime-frame-{i}", weights, proportions, {"global_frame": frames[i]}) for i in selected]
    for name, weights, proportions, pose in samples:
        if args.only and name not in args.only:
            continue
        row = check(name, weights, proportions, pose, surfaces)
        results.append(row)
        print(f"{'PASS' if row['passed'] else 'FAIL'} {name}", flush=True)
    assert results
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps({"body": str(args.body),
        "assets": {key: hashlib.sha256((args.assets / f"{key}.glb").read_bytes()).hexdigest() for key in ITEMS},
        "penetration_tolerance_m": PENETRATION_TOLERANCE_M,
        "pose_trace": str(args.pose_trace) if args.pose_trace else None,
        "skin_influences": 4,
        "limitations": "Sampled static bodies and authored bone proportions; no continuous all-pose guarantee. Triangle overlap is a conservative intersection predicate.",
        "results": results}, indent=2) + "\n")
    assert all(r["passed"] for r in results), "Underlayer intersection failures; see report"


if __name__ == "__main__":
    main()
