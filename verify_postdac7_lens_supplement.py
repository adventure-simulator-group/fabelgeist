#!/usr/bin/env python3
"""Replay the post-freeze lens-mapping supplement."""
import subprocess
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence verification requires assertions")
root = Path(__file__).resolve().parent
subprocess.run([sys.executable, "-B", str(root / "delegated"
                / "postdac7-lens-supplement" / "VERIFY.py")],
               cwd=root, check=True)
