#!/usr/bin/env python3
"""Recreate the exact earlier staged snapshot from a published successor."""
import argparse,hashlib,json,os,sys
from pathlib import Path, PurePosixPath
if sys.flags.optimize:raise RuntimeError('assertions required')
a=argparse.ArgumentParser();a.add_argument('--publication',type=Path,required=True);a.add_argument('--output',type=Path,required=True);a=a.parse_args();p=Path(__file__).resolve().parent;raw=(p/'STAGED-CAPTURE.json').read_bytes();j=json.loads(raw);assert j['count']==len(j['rows'])==139 and not a.output.exists() and not a.output.is_symlink();a.output.mkdir(mode=0o700);tree=a.output/'tree';tree.mkdir(mode=0o700)
for r in j['rows']:
 name=PurePosixPath(r['path']);assert not name.is_absolute() and '..' not in name.parts;src=a.publication.joinpath(*name.parts);assert src.is_file() and not src.is_symlink();b=src.read_bytes();oid=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();assert (len(b),hashlib.sha256(b).hexdigest(),oid)==(r['bytes'],r['sha256'],r['git_blob']);dest=tree.joinpath(*name.parts);dest.parent.mkdir(parents=True,exist_ok=True)
 with dest.open('xb') as f:f.write(b)
 dest.chmod(0o755 if r['mode']=='100755' else 0o644)
with (a.output/'CAPTURE.json').open('xb') as f:f.write(raw)
print('Recreated 139 captured staged files in a new destination; no existing files overwritten.')
