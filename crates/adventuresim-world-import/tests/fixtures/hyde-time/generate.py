#!/usr/bin/env python3
"""Generate synthetic HYDE-shaped NetCDF inputs; this does not run product code."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import zipfile

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
rows, columns = 2160, 4320
latitude = 90.0 - (np.arange(rows, dtype=np.float64) + 0.5) / 12.0
longitude = -180.0 + (np.arange(columns, dtype=np.float64) + 0.5) / 12.0
components = [
    ("cropland.nc", "cropland", [10.0, 30.0]),
    ("grazing_land.nc", "grazing_land", [20.0, 20.0]),
    ("urban_area.nc", "urban_area", [1.0, 1.0]),
]
scenarios = {
    "ascending": {"times": [547135.0, 583635.0]},
    "descending": {"times": [583635.0, 547135.0], "reverse": True},
    "negative": {"times": [-365.0, 365.0]},
    "valid_range": {"times": [-365.0, 365.0], "attrs": {"valid_range": [-365.0, 365.0]}},
    "identity_transform": {"times": [-365.0, 365.0], "attrs": {"scale_factor": 1.0, "add_offset": 0.0}},
    "unused_markers": {"times": [-365.0, 365.0], "attrs": {"missing_value": [-1095.0, -730.0]}},
    "float_valid_range": {"times": [-365.0, 365.0], "dtype": "f4", "attrs": {"valid_range": [-365.0, 365.0]}},
    "wide_offsets": {"times": [float(-(2**31) * 365), float((2**31 - 1) * 365)]},
    "small_negative_days": {"times": [-0.0001, 365.0]},
    "duplicate": {"times": [547135.0, 547135.0001, 583635.0], "first_only": True},
    "positive_extreme": {"times": [0.0, sys.float_info.max], "first_only": True},
    "negative_extreme": {"times": [-sys.float_info.max, 0.0], "first_only": True},
    "nonfinite": {"times": [0.0, float("nan")], "first_only": True},
    "fill_marker": {"times": [-365.0, 365.0], "time_fill": -365.0, "first_only": True},
    "missing_second_marker": {"times": [-365.0, 365.0], "attrs": {"missing_value": [-1095.0, -365.0]}, "first_only": True},
    "invalid_minimum": {"times": [-365.0, 365.0], "attrs": {"valid_min": 0.0}, "first_only": True},
    "reversed_range": {"times": [-365.0, 365.0], "attrs": {"valid_range": [365.0, -365.0]}, "first_only": True},
    "nonidentity_transform": {"times": [-365.0, 365.0], "attrs": {"scale_factor": 2.0}, "first_only": True},
    "component_mismatch": {"times": [-365.0, 365.0], "mismatch": True},
}

area_zip = out / "general_files.zip"
with zipfile.ZipFile(area_zip, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    info = zipfile.ZipInfo("general_files/garea_cr.asc", date_time=(1980, 1, 1, 0, 0, 0))
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = 0o600 << 16
    with archive.open(info, "w") as grid:
        grid.write(b"ncols 4320\nnrows 2160\nxllcorner -180\nyllcorner -90\ncellsize 0.08333333333333333\nNODATA_value -9999\n")
        row = b"100 " * columns + b"\n"
        for _ in range(rows):
            grid.write(row)

for scenario, config in scenarios.items():
    directory = out / scenario
    directory.mkdir(mode=0o700)
    chosen = components[:1] if config.get("first_only") else components
    for filename, variable_name, endpoints in chosen:
        times = list(config["times"])
        if config.get("mismatch") and variable_name == "grazing_land":
            times = [-365.0, 730.0]
        dtype = config.get("dtype", "f8")
        scalar = np.float32 if dtype == "f4" else np.float64
        with netCDF4.Dataset(directory / filename, "w", format="NETCDF4") as file:
            file.createDimension("time", len(times))
            file.createDimension("lat", rows)
            file.createDimension("lon", columns)
            fill = scalar(config["time_fill"]) if "time_fill" in config else False
            time = file.createVariable("time", dtype, ("time",), fill_value=fill)
            time.setncattr("units", "days since 1-5-1 00:00:00")
            time.setncattr("calendar", "365_day")
            time[:] = np.asarray(times, dtype=scalar)
            for key, value in config.get("attrs", {}).items():
                time.setncattr(key, np.asarray(value, dtype=scalar))
            lat = file.createVariable("lat", "f8", ("lat",), fill_value=False)
            lon = file.createVariable("lon", "f8", ("lon",), fill_value=False)
            lat[:] = latitude
            lon[:] = longitude
            area = file.createVariable(
                variable_name, "f4", ("time", "lat", "lon"),
                fill_value=np.float32(-9999.0), chunksizes=(1, 12, 120),
            )
            values = list(reversed(endpoints)) if config.get("reverse") else endpoints
            for index in range(len(times)):
                area[index, 1080, 2160] = values[min(index, len(values) - 1)]
    if not config.get("first_only"):
        try:
            os.link(area_zip, directory / "general_files.zip")
        except OSError:
            shutil.copyfile(area_zip, directory / "general_files.zip")

entries = []
for file in sorted(out.rglob("*")):
    if file.is_file():
        data = file.read_bytes()
        entries.append({"path": str(file.relative_to(out)), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
manifest = {
    "kind": "synthetic HYDE-shaped fixture inputs; not actual c9 data or a product test",
    "netCDF4": netCDF4.__version__,
    "netCDF_C": netCDF4.getlibversion(),
    "HDF5": netCDF4.__hdf5libversion__,
    "numpy": np.__version__,
    "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    "cell": {"row": 1080, "column": 2160, "bounds": [0.02, -0.06, 0.06, -0.02]},
    "scenarios": {name: repr(config) for name, config in scenarios.items()},
    "files": entries,
}
(out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, allow_nan=False) + "\n")
print(f"generated {len(scenarios)} scenarios, {len(entries)} files under {out}")
print("not actual HYDE data; no product or test was executed")
