#!/usr/bin/env python3
import subprocess,sys
from pathlib import Path
r=Path(__file__).resolve().parent
p=r / "delegated/sanitizer-additive-corrections/verify-reconstruction.py"
subprocess.run([sys.executable,str(p),"--publication-root",str(r),"--stage36-root",str(r),"--stage37-root",str(r),"--stage38-root",str(r)],check=True)
