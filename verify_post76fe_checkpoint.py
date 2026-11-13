#!/usr/bin/env python3
"""Verify this evidence checkpoint without executing product code."""
from pathlib import Path
import argparse,os,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
root=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--predecessor',type=Path);a=ap.parse_args();env=os.environ.copy();env.pop('PYTHONOPTIMIZE',None);env['PYTHONNOUSERSITE']='1';env['PYTHONDONTWRITEBYTECODE']='1'
cmd=[str(root/'delegated/post76fe-followups/final-guard/VERIFY.py'),'--publication',str(root)]
if a.predecessor:cmd+=['--predecessor',str(a.predecessor.resolve())]
for args in (cmd,[str(root/'delegated/post76fe-followups/closing/VERIFY.py')]):subprocess.run([sys.executable,'-B',*args],cwd=root,env=env,check=True)
print('Successor evidence checks passed; exact predecessor checks need a clean 76fe checkout and historical Git objects. Main is point-in-time; independent custody, checkpoint3821 recovery, semantic closure and product behavior remain open.')
