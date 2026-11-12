#!/usr/bin/env python3
import subprocess,sys
from pathlib import Path
p=Path(__file__).resolve().parent / "delegated/independent-evidence-reviews/verify-public.py"
subprocess.run([sys.executable,str(p)],check=True)
