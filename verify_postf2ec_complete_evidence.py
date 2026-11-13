#!/usr/bin/env python3
"""Verify successor evidence and separately sealed late q0 packet."""
from pathlib import Path
import subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
r=Path(__file__).resolve().parent
subprocess.run([sys.executable,'-B',str(r/'verify_postf2ec_final_evidence.py')],check=True)
subprocess.run([sys.executable,'-B',str(r/'delegated/postf2ec-followups/late-evidence/VERIFY.py'),'--publication',str(r)],check=True)
print('Evidence-only successor and late q0 checks passed; no product validation or semantic closure is implied.')
