"""Generate a bounded construction-parameter sweep on a saved review body.

Build the character creator's armor_fit_review example, then supply its path.
These candidates test the body and plate construction without torso neighbors.
Use check_armor_clearance.py separately for exported layered morph configurations.
"""
import argparse
import copy
import json
from pathlib import Path
import subprocess


def variants(base):
    yield "default", base
    limits = dict(front_reach=[40, 120], rear_reach=[60, 145], front_drop=[0, 60],
                  rear_drop=[0, 75], neck_reach=[20, 60], arm_length=[65, 125],
                  upper_lames=[1, 3], lower_lames=[3, 7], crown_height=[1000, 1450])
    for field, values in limits.items():
        for value in values:
            design = copy.deepcopy(base)
            design["Limb"]["Pauldron"][field] = value
            yield f"{field}-{value}", design
    for thickness in [1, 3]:
        design = copy.deepcopy(base)
        design["Limb"]["Pauldron"].update(
            gauge=dict(clearance=2, thickness=thickness), lower_lames=7,
            crown_height=1000, arm_allowance=0)
        yield f"thick-seven-{thickness}", design
    for count in [2, 24]:
        design = copy.deepcopy(base)
        design["Limb"]["Pauldron"].update(lower_lames=7, fluting=dict(
            count=count, width=850, depth=4, spread=850, lower_spread=900,
            start=150, end=800, fade=100))
        yield f"fluted-{count}", design


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("body", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--fitter", type=Path, required=True)
    parser.add_argument("--recipes", type=Path, default=Path("assets_src/equipment/armor-designs.json"))
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "body.json").write_bytes(args.body.read_bytes())
    base = json.loads(args.recipes.read_text())["pauldron"]
    for name, design in variants(base):
        source = args.output / f"{name}.json"
        source.write_text(json.dumps(design, indent=2))
        for side in ["left", "right"]:
            output = args.output / f"{name}--{side}.json"
            subprocess.run([str(args.fitter.resolve()), str(args.body), str(source),
                            side, str(output)], check=True)


if __name__ == "__main__":
    main()
