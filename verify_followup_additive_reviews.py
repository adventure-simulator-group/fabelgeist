#!/usr/bin/env python3
"""Check frozen review transports; private and unscanned material is separate."""
import argparse
import hashlib
import json
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
sha = lambda data: hashlib.sha256(data).hexdigest()
packets = [
    ("delegated/followup-40-41-sealed-scan", "4298042c65c25f66f07234cd30961b49a429ab4718de7790c8f9994391175313"),
    ("delegated/published-path-digest-independent-review", "4fecad2be738eb386216a94b3cb124ddba50b149bde0c7826d934b09e3c08fed"),
    ("delegated/independent-review-13m-collider", "6221807cee4af7240841b2ed0a883d3e9f262071c4b65a93cb5b8b6cf77b7fcf"),
    ("delegated/independent-review-13m-safety-receipts", "d83978aea2932bc08b04dbe31cf66eb6c0e055b6fbceef0e8493139ceb04f7ca"),
    ("delegated/independent-review-13m-handoff-qualification", "884f85c8825864bbf0c35acfbcffc58afcc2e7fd7175237e974e57a50fea74e9"),
    ("delegated/independent-review-followup4041-successor", "d49077d3159298c58fd68e3930b31f4049b9c595b2b4265f24e788ceb753106a"),
]
for name, expected in packets:
    folder = root / name
    data = (folder / "MANIFEST.json").read_bytes()
    assert sha(data) == expected
    records = json.loads(data)["files"]
    if isinstance(records, dict):
        records = [dict(values, path=path) for path, values in records.items()]
    actual = {p.relative_to(folder).as_posix() for p in folder.rglob("*") if p.is_file()}
    assert actual == {"MANIFEST.json"} | {row["path"] for row in records}
    for row in records:
        path = PurePosixPath(row["path"])
        assert not path.is_absolute() and ".." not in path.parts
        payload = (folder / path).read_bytes()
        assert (len(payload), sha(payload)) == (row["bytes"], row["sha256"])

q = json.loads((root / "INDEPENDENT-REVIEW-13M-PRIVATE-RETENTION.json").read_bytes())
stage = root / q["public_stage"]
archive = root / q["public_archive"]
assert sha(archive.read_bytes()) == q["public_archive_sha256"]
files = {p.relative_to(stage).as_posix() for p in stage.rglob("*") if p.is_file()}
with tarfile.open(archive, "r:gz") as tar:
    members = tar.getmembers()
    assert len(members) == len(files) == 172 and {m.name for m in members} == files
    for member in members:
        path = PurePosixPath(member.name)
        assert member.isfile() and not path.is_absolute() and ".." not in path.parts
        assert tar.extractfile(member).read() == (stage / path).read_bytes()
report = json.loads((stage / "REPORT.json").read_bytes())
assert report["corrected_assessment_sha256"] == sha((root / "ASSESSMENT-CENTRAL-COLLIDER-13M-95E.json").read_bytes())
subprocess.run([sys.executable, "-B", str(stage / "verify-public-evidence.py")], cwd=root, check=True)
subprocess.run([sys.executable, "-B", str(root / "delegated/independent-review-13m-safety-receipts/verify-safety-receipts.py")], cwd=root, check=True)
subprocess.run([sys.executable, "-B", str(root / "delegated/followup-40-41-sealed-scan/verify-receipt.py"), str(root / "delegated/followup-40-41-qualified")], cwd=root, check=True)
subprocess.run([sys.executable, "-B", str(root / "delegated/independent-review-followup4041-successor/verify-review.py"), "--stage", str(root / "delegated/followup-40-41-qualified")], cwd=root, check=True)

capture = json.loads((root / "QUALIFICATION-FOLLOWUP4041-CAPTURE-HASH.json").read_bytes())
observation = json.loads((root / "FOLLOWUP-4041-PRIVATE-REPLAY-OBSERVATION.json").read_bytes())
assert capture["capture_actual_sha256"] == observation["private_capture_sha256"]
assert capture["stage_manifest_sha256"] == observation["stage_manifest_sha256"]
assert sha((root / "delegated/followup-40-41-qualified/STAGE-MANIFEST.json").read_bytes()) == observation["stage_manifest_sha256"]
parser = argparse.ArgumentParser()
parser.add_argument("--private-root", type=Path)
args = parser.parse_args()
if args.private_root:
    private = args.private_root
    assert sha((private / "followup-4041-safe-successor/CAPTURE.json").read_bytes()) == observation["private_capture_sha256"]
    assert sha((private / "followup-4041-safe-successor/PRIVATE-REPLAY-RECEIPT.json").read_bytes()) == observation["private_replay_receipt_sha256"]
    assert sha((private / "central-13m-independent-review/PRIVATE-ORIGINAL-MANIFEST.json").read_bytes()) == q["private_original_manifest_sha256"]
    print("Selected private manifest and receipt identities checked; this does not replay all private transformations.")
else:
    print("Private manifests were not supplied and are not checked in this invocation.")
print("Additive review packets, public archive and sealed receipt verified; older Git history and unread archives remain separately qualified.")
