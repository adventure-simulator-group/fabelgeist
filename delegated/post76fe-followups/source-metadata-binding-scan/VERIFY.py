#!/usr/bin/env python3
from pathlib import Path
import argparse,hashlib,json,stat,sys
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
H=lambda b:hashlib.sha256(b).hexdigest();p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--binding-dir',type=Path,required=True);q=a.parse_args();m=json.loads((p/'MANIFEST.json').read_bytes());assert {r['path'] for r in m['files']}=={z.name for z in p.iterdir() if z.name!='MANIFEST.json'}
for r in m['files']:
 z=p/r['path'];assert z.is_file() and not z.is_symlink();b=z.read_bytes();assert (len(b),H(b),oct(stat.S_IMODE(z.stat().st_mode)))==(r['bytes'],r['sha256'],r['mode'])
r=json.loads((p/'SCAN-RECEIPT.json').read_bytes());assert H((q.binding_dir/'MANIFEST.json').read_bytes())==r['binding_manifest_sha256'] and r['positive_controls_pass'] and not any(r['pattern_counts'].values()) and not any(r['private_source_literal_hits'].values()) and r['finite_exceptions']==0
print('Metadata-binding scan receipt and bound manifest match; finite scans do not prove universal privacy.')
