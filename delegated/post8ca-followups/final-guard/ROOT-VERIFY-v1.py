#!/usr/bin/env python3
"""Verify this audit-evidence checkpoint without running product code."""
from pathlib import Path
import argparse,os,subprocess,sys

if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
root=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--predecessor',type=Path);a=ap.parse_args();env=os.environ.copy();env.pop('PYTHONOPTIMIZE',None);env['PYTHONNOUSERSITE']='1';env['PYTHONDONTWRITEBYTECODE']='1'
stage=root/'delegated/post8ca-followups';cmd=[str(stage/'final-guard/VERIFY.py'),'--publication',str(root)]
if a.predecessor:cmd+=['--predecessor',str(a.predecessor.resolve())]
subprocess.run([sys.executable,'-B',*cmd],cwd=root,env=env,check=True)
closing=stage/'closing/VERIFY.py'
if closing.exists():subprocess.run([sys.executable,'-B',str(closing)],cwd=root,env=env,check=True)
print('Successor evidence checks passed. Main is point-in-time; same-machine custody, historical Git objects, missing checkpoint3821, semantic closure and product behavior remain separately qualified.')
