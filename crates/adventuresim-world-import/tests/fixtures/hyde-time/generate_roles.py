#!/usr/bin/env python3
"""Generate distinct synthetic component values and per-file fill markers."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

parser = argparse.ArgumentParser()
parser.add_argument("--writer-site", required=True)
parser.add_argument("--output", required=True)
args = parser.parse_args()
sys.path.insert(0, args.writer_site)
import netCDF4  # noqa: E402
import numpy as np  # noqa: E402

out = Path(args.output)
if out.exists():
    raise SystemExit("output already exists; no files changed")
os.umask(0o077)
out.mkdir(mode=0o700, parents=True)

components = {
    "cropland": {"fill": -101.0, "values": [10.0, 30.0]},
    "grazing_land": {"fill": -202.0, "values": [20.0, 40.0]},
    "urban_area": {"fill": -303.0, "values": [1.0, 5.0]},
}
rows, columns = 2160, 4320
latitude = 90.0 - (np.arange(rows, dtype=np.float64) + 0.5) / 12.0
longitude = -180.0 + (np.arange(columns, dtype=np.float64) + 0.5) / 12.0

for scenario, reverse in (("role_ascending", False), ("role_descending", True)):
    destination = out / scenario
    destination.mkdir(mode=0o700)
    for component, details in components.items():
        times = [547135.0, 583635.0]
        values = details["values"]
        if reverse:
            times = list(reversed(times))
            values = list(reversed(values))
        with netCDF4.Dataset(destination / f"{component}.nc", "w", format="NETCDF4") as file:
            file.createDimension("time", len(times))
            file.createDimension("lat", rows)
            file.createDimension("lon", columns)
            time = file.createVariable("time", "f8", ("time",), fill_value=False)
            time.setncattr("units", "days since 1-5-1 00:00:00")
            time.setncattr("calendar", "365_day")
            time[:] = np.asarray(times, dtype=np.float64)
            lat = file.createVariable("lat", "f8", ("lat",), fill_value=False)
            lon = file.createVariable("lon", "f8", ("lon",), fill_value=False)
            lat[:] = latitude
            lon[:] = longitude
            area = file.createVariable(
                component, "f4", ("time", "lat", "lon"),
                fill_value=np.float32(details["fill"]), chunksizes=(1, 12, 120),
            )
            for index, value in enumerate(values):
                area[index, 1080, 2160] = np.float32(value)

variants = {
    "role_crop_missing_start": ("cropland", [-101.0, 30.0]),
    "role_crop_missing_end": ("cropland", [10.0, -101.0]),
    "role_grazing_missing": ("grazing_land", [-202.0, 40.0]),
    "role_urban_missing": ("urban_area", [-303.0, 5.0]),
    "role_crop_wrong_marker": ("cropland", [-202.0, 30.0]),
    "role_grazing_wrong_marker": ("grazing_land", [-101.0, 40.0]),
    "role_urban_wrong_marker": ("urban_area", [-202.0, 5.0]),
    "role_grazing_nan": ("grazing_land", [float("nan"), 40.0]),
}
for scenario, (component, values) in variants.items():
    destination = out / scenario
    destination.mkdir(mode=0o700)
    target = destination / f"{component}.nc"
    shutil.copyfile(out / "role_ascending" / target.name, target)
    with netCDF4.Dataset(target, "r+") as file:
        area = file.variables[component]
        area.set_auto_maskandscale(False)
        for index, value in enumerate(values):
            area[index, 1080, 2160] = np.float32(value)

# Verify the stored role names, per-file markers, physical values and time order
# using the writer's reader. This is not a product-reader test.
entries = []
for path in sorted(out.glob("*/*.nc")):
    component = path.stem
    with netCDF4.Dataset(path) as file:
        area = file.variables[component]
        area.set_auto_maskandscale(False)
        time = file.variables["time"]
        time.set_auto_maskandscale(False)
        times = [float(value) for value in time[:]]
        values = [float(area[index, 1080, 2160]) for index in range(2)]
        expected_times = [583635.0, 547135.0] if path.parent.name == "role_descending" else [547135.0, 583635.0]
        expected_values = (list(reversed(components[component]["values"]))
                           if path.parent.name == "role_descending"
                           else variants.get(path.parent.name, (None, components[component]["values"]))[1])
        assert times == expected_times
        assert float(area.getncattr("_FillValue")) == components[component]["fill"]
        assert all((np.isnan(actual) and np.isnan(expected)) or actual == expected
                   for actual, expected in zip(values, expected_values))
    data = path.read_bytes()
    entries.append({"path": str(path.relative_to(out)), "bytes": len(data),
                    "sha256": hashlib.sha256(data).hexdigest(),
                    "times": times, "values": [repr(value) for value in values],
                    "fill": components[component]["fill"]})

manifest = {"kind": "synthetic component-role fixtures; not real HYDE data",
            "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "netCDF4": netCDF4.__version__, "netCDF_C": netCDF4.getlibversion(),
            "HDF5": netCDF4.__hdf5libversion__, "numpy": np.__version__,
            "files": entries}
(out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, allow_nan=False) + "\n")
print(f"generated and writer-read-back {len(entries)} synthetic component files under {out}")
