"""Check installed underlayers against all 22 pieces of the plate fixture.

blender --background --python-exit-code 1 --python scripts/check_underlayer_plate_interfaces.py -- UNDERLAYER_ASSETS PLATE_ASSETS REPORT.json

Default: neutral body only. --pose-trace samples up to eight actual frames and
requires neighboring armor-readiness.json and body-proportions.json to preserve
the captured morph configuration. --base-plate-assets supplies missing pieces
when PLATE_ASSETS contains a partial staging export; the report records the
resolved paths and hashes. Any triangle overlap fails, with every triangle pair
retained in the report. This conservative test is not a clearance guarantee.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_underlayer_assets import ITEMS, Surface, capture_configuration
from check_parametric_armor_assets import EXPECTED_TARGETS

PLATES = [
    "morion--worn", "gorget--worn", "cuirass--worn", "fauld--worn",
    *[f"{item}--{side}" for item in (
        "pauldron", "rerebrace", "couter", "vambrace", "mitten_gauntlet",
        "cuisse", "poleyn", "greave", "sabaton") for side in ("left", "right")],
]
POSE_SAMPLE_COUNT = 8
# The plate fixture wears a gorget instead of the optional mail standard.
PLATE_UNDERLAYERS = [item for item in ITEMS if item != "mail_standard--worn"]


def resolve_assets(underlayer_dir, plate_dir, base_plate_dir):
    paths = {name: underlayer_dir / f"{name}.glb" for name in PLATE_UNDERLAYERS}
    for name in PLATES:
        path = plate_dir / f"{name}.glb"
        if not path.is_file() and base_plate_dir is not None:
            path = base_plate_dir / f"{name}.glb"
        paths[name] = path
    missing = [str(path) for path in paths.values() if not path.is_file()]
    if missing:
        raise ValueError("Missing fixture assets: " + ", ".join(missing))
    return {name: path.resolve() for name, path in paths.items()}


def fingerprint(path):
    with path.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    return {"path": str(path.resolve()), "sha256": digest}


def configurations(pose_trace):
    if pose_trace is None:
        return [("neutral", np.zeros(len(EXPECTED_TARGETS)), np.zeros(9), {})], None
    readiness_path = pose_trace.parent / "armor-readiness.json"
    proportions_path = pose_trace.parent / "body-proportions.json"
    weights, proportions = capture_configuration(pose_trace)
    frames = [json.loads(line) for line in pose_trace.read_text().splitlines()
              if line.strip()]
    if not frames:
        raise ValueError("Pose trace is empty")
    selected = np.unique(np.linspace(0, len(frames) - 1, POSE_SAMPLE_COUNT)
                         .round().astype(int))
    samples = [(f"runtime-frame-{index}", weights, proportions,
                {"global_frame": frames[index]}) for index in selected]
    evidence = {
        "trace": fingerprint(pose_trace),
        "readiness": fingerprint(readiness_path),
        "proportions": fingerprint(proportions_path),
        "frame_indices": selected.tolist(),
    }
    return samples, evidence


def check_configuration(name, weights, proportions, pose, underlayers, plate_paths):
    trees = {item: surface.tree(surface.deform(weights, proportions, pose))
             for item, surface in underlayers.items()}
    pairs = []
    total = 0
    # Load one plate at a time; full morph arrays need not coexist for 22 plates.
    for item, path in plate_paths.items():
        surface = Surface(path)
        tree = surface.tree(surface.deform(weights, proportions, pose))
        for underlayer, underlayer_tree in trees.items():
            overlaps = sorted(tree.overlap(underlayer_tree))
            total += len(overlaps)
            pairs.append({
                "plate": item,
                "underlayer": underlayer,
                "overlap_count": len(overlaps),
                "triangle_pairs": overlaps,
            })
    return {
        "name": name,
        "passed": total == 0,
        "overlap_count": total,
        "morph_weights": weights.tolist(),
        "body_proportions": proportions.tolist(),
        "scenario": pose.get("global_frame", {}).get("scenario"),
        "scenario_frame": pose.get("global_frame", {}).get("scenario_frame"),
        "pairs": pairs,
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("underlayer_assets", type=Path)
    parser.add_argument("plate_assets", type=Path)
    parser.add_argument("report", type=Path)
    parser.add_argument("--base-plate-assets", type=Path,
                        help="Resolve pieces absent from the primary plate directory here")
    parser.add_argument("--pose-trace", type=Path,
                        help="Actual runtime global-bone-transforms.jsonl")
    if argv is None:
        argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
    args = parser.parse_args(argv)
    paths = resolve_assets(args.underlayer_assets, args.plate_assets, args.base_plate_assets)
    samples, pose_evidence = configurations(args.pose_trace)
    underlayers = {name: Surface(paths[name]) for name in PLATE_UNDERLAYERS}
    plate_paths = {name: paths[name] for name in PLATES}
    report = {
        "assets": {name: fingerprint(path) for name, path in paths.items()},
        "pose_evidence": pose_evidence,
        "skin_influences": 4,
        "triangle_pair_columns": ["plate_triangle_index", "underlayer_triangle_index"],
        "limitations": (
            "Conservative triangle overlap in the sampled configurations only; "
            "no positive-clearance, containment, body-intersection, self-intersection "
            "or continuous all-body/all-pose guarantee. Triangle indices refer to "
            "the reported GLB's sole indexed primitive."
        ),
        "passed": False,
        "results": [],
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    for name, weights, proportions, pose in samples:
        row = check_configuration(name, weights, proportions, pose, underlayers, plate_paths)
        report["results"].append(row)
        args.report.write_text(json.dumps(report, indent=2) + "\n")
        print(f"{'PASS' if row['passed'] else 'FAIL'} {name}: "
              f"{row['overlap_count']} plate/underlayer triangle overlaps", flush=True)
    report["passed"] = all(row["passed"] for row in report["results"])
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    if not report["passed"]:
        raise AssertionError(f"Plate/underlayer triangle overlaps; see {args.report}")


if __name__ == "__main__":
    main()
