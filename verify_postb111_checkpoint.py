#!/usr/bin/env python3
"""Verify this evidence checkpoint without executing product code."""
from pathlib import Path
import argparse
import os
import subprocess
import sys

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
root = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--predecessor', type=Path)
args = parser.parse_args()
env = dict(os.environ)
env.pop('PYTHONOPTIMIZE', None)
env['PYTHONNOUSERSITE'] = '1'
env['PYTHONDONTWRITEBYTECODE'] = '1'
final = ['delegated/postb111-followups/final-guard/VERIFY.py', '--publication', str(root)]
if args.predecessor:
    final += ['--predecessor', str(args.predecessor)]
for command in [final,
                ['delegated/posta422-followups/final-guard/VERIFY.py'],
                ['delegated/posta422-followups/closing/VERIFY.py'],
                ['delegated/postb111-followups/closing/VERIFY.py']]:
    subprocess.run([sys.executable, '-B', *command], cwd=root, env=env, check=True)
print('Successor evidence checks passed. Earlier snapshot-specific root checks '
      'require their exact predecessor checkout. Main is point-in-time; '
      'independent custody, checkpoint 3821 recovery, semantic closure and '
      'product behavior remain open.')
