#!/usr/bin/env python3
"""Verify the independently authored frozen packaging review packets."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent / "delegated"
packets = [
    ("independent-review-followup42", "a419f34977feda95d64b2ed44bc8d76d59866036591a6c56e138e12ea4d41ecc"),
    ("independent-review-followup40-41-original", "6ee22f9152bcfeba5edc41dd39d5eef776530d2f287d534ca0e60aaf4f6b4105"),
]
for name, expected in packets:
    folder = root / name
    data = (folder / "MANIFEST.json").read_bytes()
    assert hashlib.sha256(data).hexdigest() == expected
    manifest = json.loads(data)
    records = manifest["files"]
    if isinstance(records, dict):
        records = [dict(values, path=path) for path, values in records.items()]
    expected_files = {"MANIFEST.json"}
    for row in records:
        relative = Path(row["path"])
        assert not relative.is_absolute() and ".." not in relative.parts
        payload = (folder / relative).read_bytes()
        assert len(payload) == row["bytes"]
        assert hashlib.sha256(payload).hexdigest() == row["sha256"]
        expected_files.add(relative.as_posix())
    actual_files = {p.relative_to(folder).as_posix() for p in folder.rglob("*") if p.is_file()}
    assert actual_files == expected_files
    print(f"Verified {name}: {len(records)} independently recorded files")
print("Packet identity alone does not rerun its historical private replay or establish product behavior.")
