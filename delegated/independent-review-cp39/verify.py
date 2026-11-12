#!/usr/bin/env python3
"""Verify the review packet and optionally replay read-only evidence checks."""
import argparse,hashlib,json,pathlib,subprocess,sys
p=pathlib.Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--full',action='store_true')
for n in ('public','local','source2623','source2625','source2626','prior','publication'):ap.add_argument('--'+n,type=pathlib.Path)
a=ap.parse_args();manifest=json.loads((p/'MANIFEST.json').read_bytes());actual={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file() or x.is_symlink()}-{'MANIFEST.json'};assert actual==set(manifest['files'])
for n,x in manifest['files'].items():
 f=p/n;assert f.is_file() and not f.is_symlink();raw=f.read_bytes();assert len(raw)==x['bytes'] and hashlib.sha256(raw).hexdigest()==x['sha256'],n
if a.full:
 names=['public','local','source2623','source2625','source2626','prior'];assert all(getattr(a,n) for n in names),'--full needs all six input roots'
 cmd=[sys.executable,str(p/'check-inputs.py')]
 for n in names:cmd += ['--'+n,str(getattr(a,n))]
 result=json.loads(subprocess.check_output(cmd));assert result==json.loads((p/'CHECK-RESULTS.json').read_bytes())
if a.publication:
 assert a.public,'--publication requires --public';result=json.loads(subprocess.check_output([sys.executable,str(p/'check-publication-git.py'),'--public',str(a.public),'--publication',str(a.publication)]));assert result==json.loads((p/'PUBLICATION-GIT-CHECK.json').read_bytes())
print('Review manifest verified; input replay '+('verified' if a.full else 'not requested')+'; publication-object replay '+('verified' if a.publication else 'not requested')+'.')
