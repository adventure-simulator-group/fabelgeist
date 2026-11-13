#!/usr/bin/env python3
"""Run all evidence-only checks for this checkpoint."""
from pathlib import Path
import subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
r=Path(__file__).resolve().parent
subprocess.run([sys.executable,'-B',str(r/'verify_postf2ec_publication_evidence.py')],check=True)
subprocess.run([sys.executable,'-B',str(r/'delegated/postf2ec-followups/final-guard/VERIFY-RECEIPT.py'),'--publication',str(r)],check=True)
print('Checkpoint evidence checks passed. The audit, independent custody, product behavior and all thirteen lenses remain open.')
