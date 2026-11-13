#!/usr/bin/env python3
"""Create a private identity inventory for an explicitly selected Git alternate."""
import argparse
import hashlib
import json
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
ap = argparse.ArgumentParser()
ap.add_argument("--objects", type=Path, required=True)
ap.add_argument("--output", type=Path, required=True)
a = ap.parse_args()
root, output = a.objects, a.output
assert root.is_absolute() and root.resolve() == root and root.is_dir() and not root.is_symlink()
assert output.is_absolute() and output.parent.resolve() == output.parent and output.parent.is_dir()
assert output != root and root not in output.parents
sha = lambda b: hashlib.sha256(b).hexdigest()
def snapshot():
    files, directories = {}, []
    for path in sorted(root.rglob("*")):
        assert not path.is_symlink()
        name = path.relative_to(root).as_posix()
        if path.is_file():
            data = path.read_bytes()
            files[name] = {"bytes": len(data), "sha256": sha(data)}
        else:
            assert path.is_dir()
            directories.append(name)
    return files, directories
before = snapshot()
assert before == snapshot()
with output.open("x") as f:
    json.dump({"schema": "726-supplied-git-object-alternate-inventory-v1", "files": before[0], "directories": before[1]}, f, indent=2)
    f.write("\n")
assert before == snapshot()
print(json.dumps({"files": len(before[0]), "directories": len(before[1]), "source_unchanged": True}))
