#!/usr/bin/env python3
"""Verify public reconstruction proposals and retained historical checks."""
import subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parent
cmds=[['delegated/privacy-baseline059-proposal/verify.py'],['delegated/privacy-historical-objects-proposal/verify-proposal.py'],['delegated/privacy-baseline059-inventory-supplement/verify.py','--publication',str(R)],['delegated/privacy-historical-objects-review/verify-expanded-privacy.py',str(R/'delegated/privacy-historical-objects-proposal')]]
for cmd in cmds:subprocess.run([sys.executable,str(R/cmd[0]),*cmd[1:]],cwd=R,check=True)
print('Verified privacy evidence proposals and historical-script identities; public copies are not original path-bearing bytes and history remains unchanged.')
