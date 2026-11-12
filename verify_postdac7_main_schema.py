#!/usr/bin/env python3
import subprocess,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
r=Path(__file__).resolve().parent
for name in ("VERIFY.py","VERIFY-RESTORE.py"):
 subprocess.run([sys.executable,"-B",str(r/"delegated/postdac7-main-schema-correction"/name)],cwd=r,check=True)
