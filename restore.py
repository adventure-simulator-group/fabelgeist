"""Recover the original #726 package and later public summaries, not lost state."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tempfile

REPOSITORY = "https://github.com/adventure-simulator-group/fabelgeist.git"
ORIGINAL = "eb407672813424d64ec31f208193969f5a62620c"
PROGRESS = "78b1d089cb5ee52df0b01c0bf6a020f049b8a126"
ORIGINAL_DIR = "handoff/726-audit-20261030-adler"
ORIGINAL_HASHES = {
    "MANIFEST.json": "c3e04095a9e4cb3e2da6ee2a4f561c811ebeca445a50461ddb63b2d426ed6858",
    "PUBLICATION-MANIFEST.json": "3cb9743321d6bb8561e6ac19edf1726c6c6a03f2ab8b4049580885564e65fc7b",
    "restore.py": "218fefabe6985cc847ab0c899dce262d091bdd0a1c64ccf00767d096db169ba7",
}


def sha256(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def run(*args, cwd=None):
    subprocess.run(args, cwd=cwd, check=True)


def output(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def check_case_sensitive(parent):
    with tempfile.TemporaryDirectory(prefix="726-case-", dir=parent) as name:
        directory = Path(name)
        (directory / "case-check").write_text("x", encoding="utf-8")
        return not (directory / "CASE-CHECK").exists()


def clone_pinned(destination, branch, commit, sparse=False):
    run("git", "-c", "core.autocrlf=false", "clone", "--depth", "1",
        "--filter=blob:none", "--no-checkout", "--single-branch", "--branch",
        branch, REPOSITORY, str(destination))
    if sparse:
        run("git", "-C", str(destination), "sparse-checkout", "init", "--no-cone")
        run("git", "-C", str(destination), "sparse-checkout", "set", "--no-cone",
            "/.gitignore", "/" + ORIGINAL_DIR + "/")
    if subprocess.run(["git", "-C", str(destination), "cat-file", "-e",
                       commit + "^{commit}"], stdout=subprocess.DEVNULL,
                      stderr=subprocess.DEVNULL).returncode:
        run("git", "-C", str(destination), "fetch", "--depth", "1", "origin", commit)
    run("git", "-C", str(destination), "checkout", "--detach", commit)
    if output("git", "-C", str(destination), "rev-parse", "HEAD") != commit:
        raise RuntimeError("Git commit mismatch")


def safe_path(value):
    p = PurePosixPath(value)
    if not value or p.is_absolute() or ".." in p.parts or "\\" in value or ":" in value:
        raise RuntimeError("Unsafe manifest path: " + value)
    return p


def verify_progress(root, manifest):
    if manifest["commit"] != PROGRESS or len(manifest["files"]) != manifest["file_count"]:
        raise RuntimeError("Invalid progress manifest")
    names = set()
    for row in manifest["files"]:
        rel = safe_path(row["path"])
        if row["path"] in names:
            raise RuntimeError("Duplicate progress path")
        names.add(row["path"])
        p = root.joinpath(*rel.parts)
        if p.is_symlink() or not p.is_file() or p.stat().st_size != row["bytes"]:
            raise RuntimeError("Missing or wrong-size progress file: " + row["path"])
        if sha256(p) != row["sha256"]:
            raise RuntimeError("Progress hash mismatch: " + row["path"])
    tracked = set(output("git", "-C", str(root), "ls-files").splitlines())
    if names != tracked:
        raise RuntimeError("Progress tracked-file inventory mismatch")
    checkpoint = json.loads((root / "CHECKPOINT-INCREMENT-348.json").read_text(encoding="utf-8"))
    if checkpoint["report_sha256"] != sha256(root / "PROGRESS.md"):
        raise RuntimeError("Latest report hash mismatch")
    print("Verified progress files:", len(names), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", help="new directory; never overwrites an existing path")
    parser.add_argument("--verify-only", action="store_true",
                        help="verify original transport without extracting its 12 GB payload")
    args = parser.parse_args()
    dest = Path(args.destination).absolute()
    if dest.exists() or dest.is_symlink():
        raise RuntimeError("Refusing existing destination: " + str(dest))
    if not dest.parent.is_dir():
        raise RuntimeError("Destination parent must already exist")
    dest.mkdir()
    marker = dest / "RECOVERY-INCOMPLETE"
    marker.write_text("Recovery did not finish. Do not treat this as complete.\n", encoding="utf-8")
    if not args.verify_only and not check_case_sensitive(dest):
        raise RuntimeError("Original evidence contains case-colliding paths; use a case-sensitive filesystem")
    original = dest / "original"
    progress = dest / "progress"
    clone_pinned(original, "codex/726-audit-handoff-20261030-adler", ORIGINAL, sparse=True)
    clone_pinned(progress, "codex/726-bespoke-type-progress", PROGRESS)
    package = original / ORIGINAL_DIR
    for name, expected in ORIGINAL_HASHES.items():
        if sha256(package / name) != expected:
            raise RuntimeError("Original package identity mismatch: " + name)
    manifest = json.loads((Path(__file__).resolve().parent / "PROGRESS-MANIFEST.json").read_text(encoding="utf-8"))
    verify_progress(progress, manifest)
    adapter = Path(__file__).resolve().parent / "run_original.py"
    command = [sys.executable, str(adapter)]
    if args.verify_only:
        command.append("--verify-only")
    else:
        command += ["--destination", "target/726-recovered-base"]
    run(*command, cwd=original)
    status = {"schema": "726-recovery-result-v1", "original_commit": ORIGINAL,
              "progress_commit": PROGRESS, "verify_only": args.verify_only,
              "latest_authoritative_continuation_recovered": False,
              "warning": "Original checkpoint and public summaries only; later authority is missing."}
    (dest / "RECOVERY-STATUS.json").write_text(json.dumps(status, indent=2) + "\n", encoding="utf-8")
    marker.unlink()
    print("Recovery verified. The later authoritative continuation remains missing.", flush=True)


if __name__ == "__main__":
    main()
