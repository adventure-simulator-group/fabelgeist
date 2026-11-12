#!/usr/bin/env python3
"""Bind a scratch-adapter review without claiming semantic equivalence."""
import argparse
import ast
import hashlib
import json
import subprocess
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
packet = root / "delegated/path-digest-adapter-coordinator-review"
sha = lambda data: hashlib.sha256(data).hexdigest()
manifest = (packet / "MANIFEST.json").read_bytes()
assert sha(manifest) == "3136d59c6f0920b4fe97be4890f863bc683e33b30f03f60e9a3663636a7af9d5"
rows = json.loads(manifest)["files"]
assert {p.name for p in packet.iterdir() if p.is_file()} == {"MANIFEST.json"} | {r["path"] for r in rows}
for row in rows:
    path = PurePosixPath(row["path"])
    assert not path.is_absolute() and ".." not in path.parts
    data = (packet / path).read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
review = json.loads((packet / "REVIEW.json").read_bytes())
for row in review["reviewed_files"]:
    data = (packet / row["path"]).read_bytes()
    assert (len(data), sha(data)) == (row["bytes"], row["sha256"])
    ast.parse(data)
scan = json.loads((packet / "SCAN-RESULT.json").read_bytes())
assert scan["forbidden_digest_total"] == 0 and not scan["other_candidate_counts"] and not scan["exceptions"]
crosswalk = json.loads((root / "delegated/path-digest-correction-proposal/CROSSWALK.json").read_bytes())
assert len(crosswalk["files"]) == 15
parser = argparse.ArgumentParser()
parser.add_argument("--git-repo", type=Path)
args = parser.parse_args()
if args.git_repo:
    revision = review["observed_publication_commit"]
    for row in crosswalk["files"]:
        data = subprocess.check_output(["git", "-C", str(args.git_repo), "show", f"{revision}:{row['source_path']}"])
        assert (len(data), sha(data)) == (row["original_file_bytes"], row["original_file_sha256"])
    print("Fifteen original evidence-file identities checked against the specified commit in the supplied Git repository.")
else:
    print("Git repository not supplied; the historical target-file comparison was not repeated.")
print("Review file identities and bounded scan receipt verified; live target binding, semantic equivalence and full privacy remain open.")
