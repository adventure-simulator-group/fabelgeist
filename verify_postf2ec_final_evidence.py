#!/usr/bin/env python3
"""Evidence-only verification of the post-f2ec successor and additive guard."""
from pathlib import Path
import subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
r=Path(__file__).resolve().parent;s=r/'delegated/postf2ec-followups'
for n in ['final-additions','final-guard']:
 subprocess.run([sys.executable,'-B',str(s/n/'VERIFY.py'),'--publication',str(r)],check=True)
subprocess.run([sys.executable,'-B',str(s/'guard-encrypted/VERIFY.py')],check=True)
print('Successor evidence checks passed; same-machine key custody, missing historical evidence, thirteen incomplete lenses and product validation remain separately qualified.')
