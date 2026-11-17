#!/usr/bin/env python3
"""Make synthetic descending cases with only the later component slice invalid."""

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
scenarios = {"grazing_descending_later": "grazing_land",
             "urban_descending_later": "urban_area"}
errors = []
for scenario, component in scenarios.items():
    destination = out / scenario
    destination.mkdir(mode=0o700)
    for filename in files:
        source = base / "descending" / filename
        target = destination / filename
        if filename.endswith(".zip"):
            try:
                os.link(source, target)
            except OSError:
                shutil.copyfile(source, target)
        else:
            shutil.copyfile(source, target)
    try:
        with netCDF4.Dataset(destination / f"{component}.nc", "r+") as dataset:
            variable = dataset.variables[component]
            variable.set_auto_maskandscale(False)
            variable[0, 1080, 2160] = np.float32(-7.0)
    except Exception as error:
        errors.append({"scenario": scenario, "error": f"{type(error).__name__}: {error}"})

entries = []
for path in sorted(out.rglob("*")):
    if path.is_file():
        data = path.read_bytes()
        entries.append({"path": str(path.relative_to(out)), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
manifest = {
    "kind": "synthetic later-slice fixtures; not actual c9 data or a product test",
    "base_manifest_sha256": hashlib.sha256((base / "MANIFEST.json").read_bytes()).hexdigest(),
    "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    "netCDF4": netCDF4.__version__,
    "netCDF_C": netCDF4.getlibversion(),
    "HDF5": netCDF4.__hdf5libversion__,
    "numpy": np.__version__,
    "scenarios": scenarios,
    "errors": errors,
    "files": entries,
}
(out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(f"generated {len(scenarios)} scenarios, {len(entries)} files; errors={len(errors)}")
for error in errors:
    print(error)
if errors:
    raise SystemExit(1)
