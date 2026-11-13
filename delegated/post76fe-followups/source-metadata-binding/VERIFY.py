#!/usr/bin/env python3
"""Verify the additive evidence-metadata and correction binding."""
from pathlib import Path
import argparse,hashlib,json,stat,sys
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
H=lambda b:hashlib.sha256(b).hexdigest()
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser()
for flag in ('ciphertext','restoration','scan-predecessor','scan-successor'):a.add_argument('--'+flag,type=Path,required=True)
q=a.parse_args();roots={'ciphertext':q.ciphertext,'restoration':q.restoration,'scan-predecessor':q.scan_predecessor,'scan-successor':q.scan_successor};m=json.loads((p/'MANIFEST.json').read_bytes());assert {x['path'] for x in m['files']}=={x.name for x in p.iterdir() if x.name!='MANIFEST.json'}
for x in m['files']:
 f=p/x['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b),oct(stat.S_IMODE(f.stat().st_mode)))==(x['bytes'],x['sha256'],x['mode'])
for x in json.loads((p/'METADATA-BINDING.json').read_bytes())['files']:
 f=roots[x['group']]/x['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b),oct(stat.S_IMODE(f.stat().st_mode)))==(x['bytes'],x['sha256'],x['mode'])
a=json.loads((q.scan_predecessor/'SCAN-RECEIPT.json').read_bytes());b=json.loads((q.scan_successor/'SCAN-RECEIPT.json').read_bytes());assert [k for k in sorted(set(a)|set(b)) if a.get(k)!=b.get(k)]==['at_utc'] and (q.scan_predecessor/'VERIFY.py').read_bytes()==(q.scan_successor/'VERIFY.py').read_bytes()
c=json.loads((p/'SCAN-CORRECTION.json').read_bytes());assert H((q.scan_predecessor/'MANIFEST.json').read_bytes())==c['predecessor_manifest_sha256'] and H((q.scan_successor/'MANIFEST.json').read_bytes())==c['successor_manifest_sha256']
print('Exact public metadata and separately preserved scan-receipt correction match; ciphertext authentication requires its separately held key.')
