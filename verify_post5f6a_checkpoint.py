#!/usr/bin/env python3
"""Verify the current evidence checkpoint; never execute product code."""
from pathlib import Path
import argparse,os,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
root=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--predecessor',type=Path);a=ap.parse_args();env=dict(os.environ);env.pop('PYTHONOPTIMIZE',None);env['PYTHONNOUSERSITE']='1';env['PYTHONDONTWRITEBYTECODE']='1'
app=['delegated/post5f6a-followups/application/VERIFY.py','--publication',str(root)]
if a.predecessor:app+=['--predecessor',str(a.predecessor)]
for cmd in [app,['delegated/post5f6a-followups/VERIFY.py','--publication',str(root)],['delegated/posta422-followups/final-guard/VERIFY.py'],['delegated/posta422-followups/closing/VERIFY.py']]:subprocess.run([sys.executable,'-B',*cmd],cwd=root,env=env,check=True)
print('Successor evidence checks passed; historical checkpoint-specific root checks require the exact predecessor checkout. Independent custody, checkpoint3821 recovery, all semantic closure and product behavior remain open.')
