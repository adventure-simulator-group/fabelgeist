#!/usr/bin/env python3
"""Restore pinned draft commits to a new isolated Git repository."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


def run(*args):
    subprocess.run(list(args), check=True)


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: restore_drafts.py SOURCE_REPOSITORY NEW_DESTINATION")
    source = Path(sys.argv[1]).absolute()
    dest = Path(sys.argv[2]).absolute()
    here = Path(__file__).resolve().parent
    manifest = json.loads((here / "MANIFEST.json").read_text(encoding="utf-8"))
    bundle = here / "drafts.bundle"
    if not source.is_dir():
        raise ValueError("Source repository directory does not exist")
    if hashlib.sha256(bundle.read_bytes()).hexdigest() != manifest["bundle_sha256"]:
        raise ValueError("Draft bundle hash mismatch")
    anchor = manifest["base_commit"]
    if re.fullmatch(r"[0-9a-f]{40}", anchor) is None:
        raise ValueError("Invalid base commit")
    if len(manifest["drafts"]) != 8:
        raise ValueError("Unexpected draft count")
    for item in manifest["drafts"]:
        if (re.fullmatch(r"codex/[a-z0-9-]+", item["branch"]) is None
                or re.fullmatch(r"[0-9a-f]{40}", item["commit"]) is None):
            raise ValueError("Invalid draft reference")
    dest.mkdir(parents=False, exist_ok=False)
    marker = dest / "RECOVERY-INCOMPLETE"
    marker.write_text("Draft restoration did not complete.\n", encoding="utf-8")
    run("git", "init", "--quiet", str(dest))
    common = subprocess.check_output(
        ["git", "-C", str(source), "rev-parse", "--path-format=absolute", "--git-common-dir"],
        text=True,
    ).strip()
    objects = (Path(common) / "objects").resolve()
    if not objects.is_dir() or "\n" in str(objects):
        raise ValueError("Source object directory unavailable")
    alternates = dest / ".git" / "objects" / "info" / "alternates"
    alternates.parent.mkdir(parents=True, exist_ok=True)
    alternates.write_bytes((objects.as_posix() + "\n").encode("utf-8"))
    run("git", "-C", str(dest), "cat-file", "-e", anchor + "^{commit}")
    run("git", "-C", str(dest), "update-ref", "refs/heads/recovery-base", anchor)
    run("git", "-C", str(dest), "bundle", "verify", str(bundle))
    specs = ["+refs/remotes/origin/" + x["branch"] + ":refs/heads/" + x["branch"]
             for x in manifest["drafts"]]
    run("git", "-C", str(dest), "fetch", "--quiet", "--no-tags", str(bundle), *specs)
    for item in manifest["drafts"]:
        actual = subprocess.check_output(
            ["git", "-C", str(dest), "rev-parse", "refs/heads/" + item["branch"] + "^{commit}"],
            text=True,
        ).strip()
        if actual != item["commit"]:
            raise ValueError("Draft commit mismatch: " + item["branch"])
    marker.unlink()
    print("Verified", len(manifest["drafts"]), "draft commit refs; no branch checked out")


if __name__ == "__main__":
    main()
