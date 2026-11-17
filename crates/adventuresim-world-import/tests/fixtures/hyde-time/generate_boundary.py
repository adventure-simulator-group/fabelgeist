#!/usr/bin/env python3
"""Make additional synthetic time-boundary and component-order fixtures."""

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

base = Path(args.base)
out = Path(args.output)
if out.exists():
    raise SystemExit("output already exists; no files changed")
os.umask(0o077)
out.mkdir(mode=0o700, parents=True)
files = ["cropland.nc", "grazing_land.nc", "urban_area.nc", "general_files.zip"]
scenarios = {
    "grazing_descending_order": {
        "base": "descending", "component": "grazing_land", "values": [-7.0, -8.0]
    },
    "urban_descending_order": {
        "base": "descending", "component": "urban_area", "values": [-7.0, -8.0]
    },
    "offset_above_i32": {
        "base": "negative", "times": [0.0, float((2**31) * 365)], "first_only": True
    },
    "offset_below_i32": {
        "base": "negative", "times": [float((-(2**31) - 1) * 365), 0.0], "first_only": True
    },
    "rounding_inside": {"base": "negative", "times": [-0.000364, 365.0]},
    "rounding_outside": {
        "base": "negative", "times": [-0.000366, 365.0], "first_only": True
    },
}
errors = []
for scenario, config in scenarios.items():
    destination = out / scenario
    destination.mkdir(mode=0o700)
    chosen = files[:1] if config.get("first_only") else files
    for filename in chosen:
        source = base / config["base"] / filename
        target = destination / filename
        if filename.endswith(".zip"):
            try:
                os.link(source, target)
            except OSError:
                shutil.copyfile(source, target)
        else:
            shutil.copyfile(source, target)
    try:
        if "component" in config:
            name = config["component"]
            with netCDF4.Dataset(destination / f"{name}.nc", "r+") as dataset:
                variable = dataset.variables[name]
                variable.set_auto_maskandscale(False)
                for index, value in enumerate(config["values"]):
                    variable[index, 1080, 2160] = np.float32(value)
        if "times" in config:
            for filename in chosen:
                if filename.endswith(".nc"):
                    with netCDF4.Dataset(destination / filename, "r+") as dataset:
                        variable = dataset.variables["time"]
                        variable.set_auto_maskandscale(False)
                        variable[:] = np.asarray(config["times"], dtype=np.float64)
    except Exception as error:
        errors.append({"scenario": scenario, "error": f"{type(error).__name__}: {error}"})

entries = []
for path in sorted(out.rglob("*")):
    if path.is_file():
        data = path.read_bytes()
        entries.append({"path": str(path.relative_to(out)), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
manifest = {
    "kind": "synthetic boundary fixtures; not actual c9 data or a product test",
    "base_manifest_sha256": hashlib.sha256((base / "MANIFEST.json").read_bytes()).hexdigest(),
    "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    "netCDF4": netCDF4.__version__,
    "netCDF_C": netCDF4.getlibversion(),
    "HDF5": netCDF4.__hdf5libversion__,
    "numpy": np.__version__,
    "cell": {"row": 1080, "column": 2160, "bounds": [0.02, -0.06, 0.06, -0.02]},
    "scenarios": scenarios,
    "errors": errors,
    "files": entries,
}
(out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, allow_nan=False) + "\n")
print(f"generated {len(scenarios)} scenarios, {len(entries)} files; errors={len(errors)}")
for error in errors:
    print(error)
if errors:
    raise SystemExit(1)
