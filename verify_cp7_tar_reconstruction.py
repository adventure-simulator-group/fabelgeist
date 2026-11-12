#!/usr/bin/env python3
import subprocess,sys
from pathlib import Path
p=Path(__file__).resolve().parent / "delegated/cp7-tar-reconstruction/verify_cp7_reconstruction.py"
subprocess.run([sys.executable,str(p)],check=True)
