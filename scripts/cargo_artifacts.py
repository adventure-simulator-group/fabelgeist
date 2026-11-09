#!/usr/bin/env python3
"""Measure development artifacts and preview bounded Cargo cleanup."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import math
import os
from pathlib import Path
import shlex
import stat
import subprocess
import sys
import time
import tomllib


ROOT = Path(__file__).resolve().parents[1]
GIB = 1024**3
SECONDS_PER_DAY = 24 * 60 * 60
DEFAULT_MAX_AGE_DAYS = 14
DEFAULT_MAX_SIZE_GIB = 64


@dataclass(frozen=True)
class DevelopmentArtifacts:
    manifest: Path
    target_dir: Path
    target: str | None
    path: Path
    logical_bytes: int
    unique_bytes: int
    files: int
    newest_modified: float


def cargo_output(arguments: list[str], directory: Path) -> str:
    return subprocess.check_output(
        ["cargo", *arguments], cwd=directory, text=True,
        stderr=subprocess.PIPE,
    )


def workspace_manifests(root: Path) -> list[Path]:
    """Include the main workspace and its independently built exclusions."""
    manifest = root / "Cargo.toml"
    workspace = tomllib.loads(manifest.read_text(encoding="utf-8"))["workspace"]
    manifests = [manifest]
    for excluded in workspace.get("exclude", []):
        candidate = root / excluded / "Cargo.toml"
        if candidate.is_file():
            manifests.append(candidate)
    return manifests


def redirected_directory(path: Path) -> bool:
    try:
        attributes = path.lstat()
    except FileNotFoundError:
        return False
    if stat.S_ISLNK(attributes.st_mode):
        return True
    return os.name == "nt" and bool(
        attributes.st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT
    )


def inspect_directory(
    manifest: Path, target_dir: Path, target: str | None,
) -> DevelopmentArtifacts:
    directory = target_dir / target / "debug" if target else target_dir / "debug"
    if redirected_directory(directory):
        raise ValueError(f"Refusing redirected development directory: {directory}")
    logical_bytes = unique_bytes = files = 0
    newest_modified = 0.0
    identities: set[tuple[int, int]] = set()
    if not directory.exists():
        return DevelopmentArtifacts(
            manifest, target_dir, target, directory, 0, 0, 0, 0.0,
        )

    def fail_scan(error: OSError) -> None:
        raise error

    for parent, children, names in os.walk(directory, onerror=fail_scan):
        # Never follow links into runtime data or another checkout.
        children[:] = [
            name for name in children
            if not redirected_directory(Path(parent) / name)
        ]
        for name in names:
            path = Path(parent) / name
            if path.is_symlink():
                continue
            info = path.stat()
            files += 1
            logical_bytes += info.st_size
            newest_modified = max(newest_modified, info.st_mtime)
            identity = (info.st_dev, info.st_ino)
            if identity not in identities:
                identities.add(identity)
                unique_bytes += info.st_size
    return DevelopmentArtifacts(
        manifest, target_dir, target, directory, logical_bytes, unique_bytes,
        files, newest_modified,
    )


def inventory(root: Path | None = None) -> list[DevelopmentArtifacts]:
    root = ROOT if root is None else root
    targets = set(subprocess.check_output(
        ["rustc", "--print", "target-list"], cwd=root, text=True,
    ).splitlines())
    inspected: set[Path] = set()
    results = []
    for manifest in workspace_manifests(root):
        configuration = json.loads(cargo_output([
            "-Z", "unstable-options", "config", "get", "build", "--format=json",
        ], manifest.parent))
        build = configuration["build"]
        if "CARGO_BUILD_BUILD_DIR" in os.environ or "build-dir" in build:
            raise ValueError(
                "Separate build.build-dir directories are outside this helper's "
                "measurement and cleanup policy.",
            )
        configured_target = "target-dir" in build or any(
            name in os.environ for name in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR")
        )
        if configured_target:
            metadata = json.loads(cargo_output([
                "metadata", "--offline", "--no-deps", "--format-version=1",
                "--manifest-path", str(manifest), "--locked",
            ], manifest.parent))
            target_dir = Path(metadata["target_directory"]).resolve()
        else:
            workspace = Path(cargo_output([
                "locate-project", "--workspace", "--message-format=plain",
                "--manifest-path", str(manifest),
            ], manifest.parent).strip())
            target_dir = (workspace.parent / "target").resolve()
        if target_dir in inspected:
            continue
        inspected.add(target_dir)
        results.append(inspect_directory(manifest, target_dir, None))
        if target_dir.is_dir():
            for child in sorted(target_dir.iterdir()):
                if child.name in targets and (child / "debug").is_dir():
                    if redirected_directory(child):
                        raise ValueError(f"Refusing redirected target: {child}")
                    results.append(inspect_directory(manifest, target_dir, child.name))
    return results


def cleanup_plan(
    artifacts: list[DevelopmentArtifacts], days: float, max_gib: float,
) -> list[DevelopmentArtifacts]:
    cutoff = time.time() - days * SECONDS_PER_DAY
    remaining = sum(item.unique_bytes for item in artifacts)
    budget = max_gib * GIB
    selected = []
    for item in sorted(artifacts, key=lambda item: item.newest_modified):
        if item.files and (item.newest_modified < cutoff or remaining > budget):
            selected.append(item)
            remaining -= item.unique_bytes
    return selected


def print_inventory(artifacts: list[DevelopmentArtifacts]) -> None:
    for item in artifacts:
        print(
            f"{item.unique_bytes / GIB:8.2f} GiB unique, "
            f"{item.logical_bytes / GIB:8.2f} GiB logical, "
            f"{item.files:7d} files  {item.path}",
        )
    print(
        f"Total: {sum(item.unique_bytes for item in artifacts) / GIB:.2f} GiB "
        f"after deduplicating hard links; "
        f"{sum(item.logical_bytes for item in artifacts) / GIB:.2f} GiB logical.",
    )


def nonnegative_number(value: str) -> float:
    number = float(value)
    if not math.isfinite(number) or number < 0:
        raise argparse.ArgumentTypeError("must be a finite, nonnegative number")
    return number


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["report", "prune"])
    parser.add_argument("--root", type=Path, default=ROOT, help="checkout to inspect")
    parser.add_argument("--days", type=nonnegative_number, default=DEFAULT_MAX_AGE_DAYS)
    parser.add_argument("--max-gib", type=nonnegative_number, default=DEFAULT_MAX_SIZE_GIB)
    parser.add_argument("--apply", action="store_true", help="execute the printed cleanup plan")
    parser.add_argument("--json", type=Path, help="write an inventory for measurement reports")
    args = parser.parse_args()
    artifacts = inventory(args.root.resolve())
    print_inventory(artifacts)
    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps([
            {
                "manifest": str(item.manifest), "path": str(item.path),
                "unique_bytes": item.unique_bytes,
                "logical_bytes": item.logical_bytes, "files": item.files,
                "newest_modified": item.newest_modified,
            }
            for item in artifacts
        ], indent=2) + "\n", encoding="utf-8")
    if args.command == "report":
        return 0
    plan = cleanup_plan(artifacts, args.days, args.max_gib)
    for item in plan:
        # Cleaning a profile delegates locking and layout details to Cargo.
        # Profile-only cleanup is rooted directly below --target-dir, including
        # when cleaning an explicitly selected cross-compilation directory.
        clean_root = item.target_dir / item.target if item.target else item.target_dir
        command = [
            "cargo", "clean", "--profile", "dev", "--manifest-path",
            str(item.manifest), "--target-dir", str(clean_root),
        ]
        displayed = subprocess.list2cmdline(command) if os.name == "nt" else shlex.join(command)
        print(("Cleaning: " if args.apply else "Would run: ") + displayed, flush=True)
        if args.apply:
            subprocess.run(command, cwd=item.manifest.parent, check=True)
    if args.apply:
        print_inventory(inventory(args.root.resolve()))
    else:
        print("Preview only. Add --apply to clean the listed development profiles.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except subprocess.CalledProcessError as error:
        print(error.stderr or str(error), file=sys.stderr)
        sys.exit(error.returncode)
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
