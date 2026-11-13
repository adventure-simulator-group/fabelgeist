#!/usr/bin/env python3
"""Verify evidence snapshots and the separately captured late packet."""
from pathlib import Path
import subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
r=Path(__file__).resolve().parent
subprocess.run([sys.executable,'-B',str(r/'verify_postf2ec_complete_evidence.py')],check=True)
subprocess.run([sys.executable,'-B',str(r/'delegated/postf2ec-followups/final-guard/VERIFY-LATE.py'),'--publication',str(r)],check=True)
print('Verified published evidence and separate late snapshot; final guard receipt remains separately qualified.')
