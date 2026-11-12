#!/usr/bin/env python3
"""Read-only verification of the review and optional external evidence replay."""
import argparse,pathlib,json,hashlib,subprocess,sys
P=pathlib.Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--stage',type=pathlib.Path);ap.add_argument('--private',type=pathlib.Path);ap.add_argument('--scan',action='store_true');a=ap.parse_args();m=json.loads((P/'MANIFEST.json').read_bytes());actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() or x.is_symlink()}-{'MANIFEST.json'};assert actual==set(m['files'])
for n,v in m['files'].items():
 f=P/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (hashlib.sha256(b).hexdigest(),len(b))==(v['sha256'],v['bytes'])
assert bool(a.stage)==bool(a.private),'--stage and --private must be supplied together'
if a.stage:
 for prog,out in [('check.py','CHECK.json'),('replay.py','REPLAY.json'),('check-path-hash-exposure.py','PATH-DIGEST-EXPOSURE.json')]:
  got=json.loads(subprocess.check_output([sys.executable,str(P/prog),'--stage',str(a.stage),'--private',str(a.private)]));assert got==json.loads((P/out).read_bytes())
if a.scan:
 assert a.stage,'--scan requires --stage and --private';got=json.loads(subprocess.check_output([sys.executable,str(P/'scan.py'),'--stage',str(a.stage)]));assert got==json.loads((P/'SCAN-RAW.json').read_bytes())
print('Review manifest verified; input replay '+('verified' if a.stage else 'not requested')+'; bounded scan '+('verified' if a.scan else 'not requested')+'.')
