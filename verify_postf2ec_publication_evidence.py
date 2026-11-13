#!/usr/bin/env python3
"""Run the checkpoint's evidence-only snapshot and custody checks."""
from pathlib import Path
import subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
r=Path(__file__).resolve().parent
subprocess.run([sys.executable,'-B',str(r/'verify_postf2ec_guard_all.py')],check=True)
subprocess.run([sys.executable,'-B',str(r/'delegated/postf2ec-followups/final-guard/VERIFY-TAIL.py'),'--publication',str(r)],check=True)
print('Checkpoint evidence checks passed; no product behavior, independent custody or semantic audit closure is asserted.')
