#!/usr/bin/env python3
"""Restore registered source objects without changing the source repository."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: restore_sources.py REPOSITORY DESTINATION")
    repo = Path(sys.argv[1]).absolute()
    if not repo.is_dir():
        raise ValueError("Repository directory does not exist")
    dest = Path(sys.argv[2]).absolute()
    manifest = json.loads((Path(__file__).parent / "MANIFEST.json").read_text(encoding="utf-8"))
    dest.mkdir(parents=False, exist_ok=False)
    marker = dest / "RECOVERY-INCOMPLETE"
    marker.write_text("Source restoration did not complete.\n", encoding="utf-8")
    objects = dest / "objects"
    objects.mkdir()
    verified = set()
    for row in manifest:
        sha = row["sha256"]
        if len(sha) != 64 or any(c not in "0123456789abcdef" for c in sha):
            raise ValueError("Invalid SHA-256 in manifest")
        result = subprocess.run(
            ["git", "-C", str(repo), "show", row["revision"] + ":" + row["git_path"]],
            check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        if hashlib.sha256(result.stdout).hexdigest() != sha:
            raise ValueError("Source hash mismatch: " + row["git_path"])
        path = objects / sha
        if not path.exists():
            path.write_bytes(result.stdout)
        elif path.read_bytes() != result.stdout:
            raise ValueError("Conflicting content for " + sha)
        verified.add(sha)
    marker.unlink()
    print("Verified", len(manifest), "identities and", len(verified), "source objects")


if __name__ == "__main__":
    main()
