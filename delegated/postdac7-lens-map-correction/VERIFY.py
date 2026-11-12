#!/usr/bin/env python3
"""Verify an additive audit-evidence correction, without product execution."""
import gzip
import hashlib
import io
import json
import subprocess
import sys
import tarfile
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence verification requires assertions")
here = Path(__file__).resolve().parent
root = here.parent.parent
sha = lambda raw: hashlib.sha256(raw).hexdigest()
record = json.loads((here / "RECORD.json").read_bytes())
assert record["schema"] == "726-postdac7-lens-map-amendment-v1"
for name, expected in record["manifest_sha256"].items():
    assert sha((here / name / "MANIFEST.json").read_bytes()) == expected
    manifest = json.loads((here / name / "MANIFEST.json").read_bytes())
    files = {row["path"]: row for row in manifest["files"]}
    actual = {
        p.relative_to(here / name).as_posix()
        for p in (here / name).rglob("*")
        if p.is_file() and p.name != "MANIFEST.json"
    }
    assert actual == set(files)
    for filename, row in files.items():
        data = (here / name / filename).read_bytes()
        assert (len(data), sha(data)) == (row["bytes"], row["sha256"])

q2968 = root / "delegated/postdac7-followups/later-q2968"
subprocess.run([sys.executable, "-B", str(here / "mapping/verify.py"),
                "--q2968", str(q2968)], check=True)
subprocess.run([sys.executable, "-B", str(here / "field-note/verify.py"),
                "--q2968", str(q2968), "--map-correction",
                str(here / "mapping")], check=True)
baseline = gzip.decompress((root / "delegated/postdac7-main-schema-correction"
                            / "BASELINE-AFTER.json.gz").read_bytes())
mapping = json.loads((here / "mapping/CORRECTION.json").read_bytes())
assert sha(baseline) == mapping["observed_working_baseline_sha256"]
assert json.loads(baseline)["lens_names"] == mapping["lens_names"]

before = gzip.decompress((here / "CONTINUATION-BEFORE.md.gz").read_bytes())
after = gzip.decompress((here / "CONTINUATION-AFTER.md.gz").read_bytes())
assert sha(before) == record["continuation_before_sha256"]
assert sha(after) == record["continuation_after_sha256"]
assert before.count(record["anchor"].encode()) == 1
assert after == before.replace(record["anchor"].encode(),
                               record["insertion"].encode()
                               + record["anchor"].encode(), 1)
archive = (root / "delegated/postdac7-followups/application"
           / "APPLICATION-EVIDENCE.tar.gz").read_bytes()
with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as tar:
    assert tar.extractfile("after/CONTINUATION.md").read() == before
print("Verified the q2968 lens correction and continuation amendment; "
      "the later q3016 packet remains an external binding.")
