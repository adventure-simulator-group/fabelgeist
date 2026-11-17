#!/usr/bin/env python3
"""Make additional synthetic metadata cases from a retained negative-axis input."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

parser = argparse.ArgumentParser()
parser.add_argument("--writer-site", required=True)
parser.add_argument("--base", required=True)
parser.add_argument("--output", required=True)
args = parser.parse_args()
sys.path.insert(0, args.writer_site)
import netCDF4  # noqa: E402
import numpy as np  # noqa: E402

base = Path(args.base) / "negative"
out = Path(args.output)
if out.exists():
    raise SystemExit("output already exists; no files changed")
os.umask(0o077)
out.mkdir(mode=0o700, parents=True)
files = ["cropland.nc", "grazing_land.nc", "urban_area.nc", "general_files.zip"]
scenarios = {
    "later_missing_marker": [("grazing_land.nc", "missing_value", [-1095.0, -365.0], "f8")],
    "later_invalid_minimum": [("urban_area.nc", "valid_min", 0.0, "f8")],
    "missing_wrong_type": [("cropland.nc", "missing_value", -365, "i4")],
    "range_wrong_type": [("cropland.nc", "valid_range", [-365, 365], "i4")],
    "range_three_values": [("cropland.nc", "valid_range", [-365.0, 0.0, 365.0], "f8")],
    "range_conflict": [("cropland.nc", "valid_range", [-365.0, 365.0], "f8"),
                       ("cropland.nc", "valid_min", -365.0, "f8")],
    "valid_min_max": [(file, attr, value, "f8")
                      for file in files[:3]
                      for attr, value in [("valid_min", -365.0), ("valid_max", 365.0)]],
    "range_nonfinite": [("cropland.nc", "valid_range", [float("-inf"), 365.0], "f8")],
    "unused_nan_marker": [("cropland.nc", "missing_value", float("nan"), "f8")],
    "identity_float_scale": [("cropland.nc", "scale_factor", 1.0, "f4")],
    "scale_string": [("cropland.nc", "scale_factor", "invalid", "text")],
    "scale_vector": [("cropland.nc", "scale_factor", [1.0, 1.0], "f8")],
    "scale_nonfinite": [("cropland.nc", "scale_factor", float("nan"), "f8")],
    "offset_nonidentity": [("cropland.nc", "add_offset", 1.0, "f8")],
    "near_equal_component": [("grazing_land.nc", "__times__", [-364.99999925, 365.00000075], "f8")],
    "opposite_component": [("grazing_land.nc", "__times__", [365.0, -365.0], "f8")],
}
errors = []
for scenario, operations in scenarios.items():
    destination = out / scenario
    destination.mkdir(mode=0o700)
    for filename in files:
        source = base / filename
        target = destination / filename
        if filename.endswith(".zip"):
            try:
                os.link(source, target)
            except OSError:
                shutil.copyfile(source, target)
        else:
            shutil.copyfile(source, target)
    for filename, attr, value, dtype in operations:
        try:
            with netCDF4.Dataset(destination / filename, "r+") as file:
                time = file.variables["time"]
                time.set_auto_maskandscale(False)
                if attr == "__times__":
                    time[:] = np.asarray(value, dtype=np.float64)
                else:
                    converted = value if dtype == "text" else np.asarray(value, dtype=dtype)
                    time.setncattr(attr, converted)
        except Exception as error:
            errors.append({"scenario": scenario, "attribute": attr,
                           "error": f"{type(error).__name__}: {error}"})

entries = []
for path in sorted(out.rglob("*")):
    if path.is_file():
        data = path.read_bytes()
        entries.append({"path": str(path.relative_to(out)), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
manifest = {
    "kind": "additional synthetic fixtures; not actual c9 data or a product test",
    "base_manifest_sha256": hashlib.sha256((Path(args.base) / "MANIFEST.json").read_bytes()).hexdigest(),
    "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    "netCDF4": netCDF4.__version__,
    "scenarios": {name: repr(operations) for name, operations in scenarios.items()},
    "errors": errors,
    "files": entries,
}
(out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, allow_nan=False) + "\n")
print(f"generated {len(scenarios)} scenarios, {len(entries)} files; errors={len(errors)}")
for error in errors:
    print(error)
if errors:
    raise SystemExit(1)
