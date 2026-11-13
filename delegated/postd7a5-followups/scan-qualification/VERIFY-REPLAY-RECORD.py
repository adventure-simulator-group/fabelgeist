#!/usr/bin/env python3
"""Verify the public replay record and optionally authenticated raw originals."""
import argparse,hashlib,json,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('assertions required')
p=Path(__file__).resolve().parent;r=json.loads((p/'EVIDENCE-REPLAY.json').read_bytes());assert r['schema']=='726-postd7a5-evidence-replay-public-v1' and r['driver_exit']==0 and r['staged_capture_files']==139 and r['direct_child_assertion_preflight']=={'assertions_enabled':True,'optimization_level':0};assert hashlib.sha256((p/'STAGED-CAPTURE.json').read_bytes()).hexdigest()==r['staged_capture_sha256']
a=argparse.ArgumentParser();a.add_argument('--restored',type=Path);a=a.parse_args()
if a.restored:
 q=a.restored/'driver-evidence/evidence';b=(q/'evidence.log').read_bytes();old=json.loads((q/'RESULT.json').read_bytes());assert (len(b),len(b.splitlines()),hashlib.sha256(b).hexdigest())==(r['transcript_bytes'],r['transcript_lines'],r['transcript_sha256']);assert old['exit']==r['driver_exit'] and (q/'CAPTURE.json').read_bytes()==(p/'STAGED-CAPTURE.json').read_bytes();print('Verified restored original evidence transcript and staged capture.')
else:print('Verified public replay summary; authenticated original transcript not supplied.')
print('Same-machine custody, nested assertion flags, product behavior and audit closure remain separate.')
