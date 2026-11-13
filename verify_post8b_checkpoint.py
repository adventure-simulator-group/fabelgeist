#!/usr/bin/env python3
"""Run the post-8b evidence guard, optionally with exact predecessors."""
import argparse
import hashlib
import os
import subprocess
import sys
from pathlib import Path
if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
root = Path(__file__).resolve().parent
ap = argparse.ArgumentParser()
ap.add_argument("--predecessor", type=Path)
ap.add_argument("--ancestor", type=Path)
ap.add_argument("--object-alternate", type=Path)
ap.add_argument("--object-inventory", type=Path)
ap.add_argument("--object-inventory-sha256")
a = ap.parse_args()
assert not a.ancestor or a.predecessor
assert all((a.object_alternate, a.object_inventory, a.object_inventory_sha256)) or not any((a.object_alternate, a.object_inventory, a.object_inventory_sha256))
assert not a.object_alternate or a.predecessor
guard = root / "delegated/post8b-followups/checkpoint-guard/VERIFY.py"
assert guard.is_file() and not guard.is_symlink()
assert hashlib.sha256(guard.read_bytes()).hexdigest() == "d1d9ce1956b71365dcab5ef3cbd466fc55310bae740c49141c73528b29ab0f10"
args = [sys.executable, "-B", str(guard), "--publication", str(root)]
if a.predecessor:
    args += ["--predecessor", str(a.predecessor.resolve())]
if a.ancestor:
    args += ["--ancestor", str(a.ancestor.resolve())]
if a.object_alternate:
    args += ["--object-alternate", str(a.object_alternate), "--object-inventory", str(a.object_inventory),
             "--object-inventory-sha256", a.object_inventory_sha256]
env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
env.update(PYTHONDONTWRITEBYTECODE="1", PYTHONNOUSERSITE="1")
env.pop("PYTHONOPTIMIZE", None)
subprocess.run(args, cwd=root, env=env, check=True)
