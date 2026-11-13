#!/usr/bin/env python3
"""Check this metadata-scan receipt and its named public evidence bindings."""
import argparse,hashlib,json,stat,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('unoptimized interpreter required')
H=lambda b:hashlib.sha256(b).hexdigest()
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--ciphertext-dir',type=Path,required=True);a.add_argument('--restoration-receipt-dir',type=Path,required=True);q=a.parse_args();m=json.loads((p/'MANIFEST.json').read_bytes());assert {x['path'] for x in m['files']}=={x.name for x in p.iterdir() if x.name!='MANIFEST.json'}
for x in m['files']:
 f=p/x['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b),oct(stat.S_IMODE(f.stat().st_mode)))==(x['bytes'],x['sha256'],x['mode'])
r=json.loads((p/'SCAN-RECEIPT.json').read_bytes());assert H((q.ciphertext_dir/'MANIFEST.json').read_bytes())==r['ciphertext_manifest_sha256'];assert H((q.restoration_receipt_dir/'MANIFEST.json').read_bytes())==r['restoration_receipt_manifest_sha256'];assert r['positive_controls_pass'] and not any(r['pattern_counts'].values()) and not any(r['private_source_literal_hits'].values()) and r['finite_exceptions']==0
print('Metadata scan receipt and its two public manifest identities match; finite scans do not prove universal privacy.')
