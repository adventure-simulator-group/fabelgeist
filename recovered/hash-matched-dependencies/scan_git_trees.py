#!/usr/bin/env python3
"""Compare selected Git-tree text blobs against unresolved historical hashes."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import threading

if len(sys.argv) != 3:
    raise SystemExit("usage: scan_git_trees.py SOURCE_REPOSITORY PROGRESS_REPOSITORY")
root = Path(__file__).resolve().parent.parent.parent
coverage_dir = root / "recovered/hash-matched-dependencies"
coverage = json.loads((coverage_dir / "DEPENDENCY-COVERAGE.json").read_bytes())
later = json.loads((coverage_dir / "LATER-MATCHES.json").read_bytes())
missing = {r["sha256"] for r in coverage["references"]
           if r["availability"] == "not_located_by_hash"}
missing -= {r["sha256"] for r in later["matches"]}
revisions = [(sys.argv[1], "f82adcabd9d89c4fa2dd7f99253a45297a0681ee"),
             (sys.argv[1], "eb407672813424d64ec31f208193969f5a62620c"),
             (sys.argv[2], "78b1d089cb5ee52df0b01c0bf6a020f049b8a126")]
results = []
for repo, revision in revisions:
    raw = subprocess.check_output(["git", "-C", repo, "ls-tree", "-rlz", "--full-tree", revision])
    blobs = {}
    for item in raw.split(b"\0"):
        if not item:
            continue
        header, path = item.split(b"\t", 1)
        mode, kind, oid, size = header.split()
        name = path.decode("utf8", "surrogateescape").rsplit("/", 1)[-1]
        ext = name.rsplit(".", 1)[-1].lower() if "." in name else ""
        if kind == b"blob" and ext in ("json", "py", "jsonl", "md", "txt", "") and int(size) <= 4000000:
            blobs.setdefault(oid.decode(), []).append(path.decode("utf8", "surrogateescape"))
    oids = sorted(blobs)
    proc = subprocess.Popen(["git", "-C", repo, "cat-file", "--batch"],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    def submit():
        proc.stdin.write(("\n".join(oids) + "\n").encode())
        proc.stdin.close()
    thread = threading.Thread(target=submit)
    thread.start()
    matches = []
    for oid in oids:
        header = proc.stdout.readline().decode().split()
        if header[:2] != [oid, "blob"]:
            raise ValueError("Unexpected Git object")
        size = int(header[2])
        data = proc.stdout.read(size)
        if len(data) != size or proc.stdout.read(1) != b"\n":
            raise ValueError("Incomplete Git object")
        digest = hashlib.sha256(data).hexdigest()
        if digest in missing:
            matches.append({"sha256": digest, "git_blob": oid, "paths": blobs[oid]})
    thread.join()
    if proc.wait() != 0:
        raise RuntimeError("Git object scan failed")
    results.append({"revision": revision, "distinct_blobs_checked": len(oids), "matches": matches})
print(json.dumps({"unmatched_hashes_checked": len(missing), "results": results}, indent=2))
