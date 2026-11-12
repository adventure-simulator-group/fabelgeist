#!/usr/bin/env python3
import subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent
p=r / "delegated/independent-review-13k-rig/verify-public.py"
subprocess.run([sys.executable,str(p),"--proof",str(r / "delegated/central-13k-windows-proof.tar.gz")],check=True)
