#!/usr/bin/env python3
"""Pack the files used by the HYDE time-axis integration test."""

import argparse
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

parser = argparse.ArgumentParser()
parser.add_argument("--base", type=Path, required=True)
parser.add_argument("--extra", type=Path, required=True)
parser.add_argument("--boundary", type=Path, required=True)
parser.add_argument("--later", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()

components = ("cropland.nc", "grazing_land.nc", "urban_area.nc")
complete = (
    "ascending", "descending", "negative", "valid_range", "identity_transform",
    "unused_markers", "float_valid_range", "wide_offsets", "small_negative_days",
)
early_errors = (
    "duplicate", "positive_extreme", "negative_extreme", "nonfinite", "fill_marker",
    "missing_second_marker", "invalid_minimum", "reversed_range", "nonidentity_transform",
)
extra = {
    "later_missing_marker": ("grazing_land.nc",),
    "later_invalid_minimum": ("urban_area.nc",),
    "missing_wrong_type": ("cropland.nc",),
    "range_wrong_type": ("cropland.nc",),
    "range_three_values": ("cropland.nc",),
    "range_conflict": ("cropland.nc",),
    "valid_min_max": components,
    "range_nonfinite": ("cropland.nc",),
    "unused_nan_marker": ("cropland.nc",),
    "identity_float_scale": ("cropland.nc",),
    "scale_string": ("cropland.nc",),
    "scale_vector": ("cropland.nc",),
    "scale_nonfinite": ("cropland.nc",),
    "offset_nonidentity": ("cropland.nc",),
    "near_equal_component": ("grazing_land.nc",),
    "opposite_component": ("grazing_land.nc",),
}

files = {"area/general_files.zip": args.base / "general_files.zip"}
for scenario in complete:
    for filename in components:
        files[f"{scenario}/{filename}"] = args.base / scenario / filename
for scenario in early_errors:
    files[f"{scenario}/cropland.nc"] = args.base / scenario / "cropland.nc"
files["component_mismatch/grazing_land.nc"] = args.base / "component_mismatch/grazing_land.nc"
for scenario, filenames in extra.items():
    for filename in filenames:
        files[f"{scenario}/{filename}"] = args.extra / scenario / filename
for scenario in ("grazing_descending_order", "urban_descending_order", "rounding_inside"):
    for filename in components:
        files[f"{scenario}/{filename}"] = args.boundary / scenario / filename
for scenario in ("offset_above_i32", "offset_below_i32", "rounding_outside"):
    files[f"{scenario}/cropland.nc"] = args.boundary / scenario / "cropland.nc"
for scenario in ("grazing_descending_later", "urban_descending_later"):
    for filename in components:
        files[f"{scenario}/{filename}"] = args.later / scenario / filename

if args.output.exists():
    raise SystemExit("output already exists; no files changed")
with ZipFile(args.output, "w", compression=ZIP_DEFLATED, compresslevel=9) as archive:
    for name, source in sorted(files.items()):
        info = ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = ZIP_DEFLATED
        info.external_attr = 0o600 << 16
        archive.writestr(info, source.read_bytes(), compresslevel=9)
print(f"packed {len(files)} synthetic files into {args.output}")
