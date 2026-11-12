#!/usr/bin/env python3
import subprocess,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
r=Path(__file__).resolve().parent;subprocess.run([sys.executable,"-B",str(r/"delegated/postdac7-diff-check/VERIFY.py")],cwd=r,check=True)
