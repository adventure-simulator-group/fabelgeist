#!/usr/bin/env python3
"""Read-only verification of receipt and specified frozen stage."""
import argparse,hashlib,json
from pathlib import Path
a=argparse.ArgumentParser();a.add_argument('stage');args=a.parse_args();p=Path(__file__).resolve().parent;s=Path(args.stage).resolve();h=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((p/'MANIFEST.json').read_bytes());files={f.relative_to(p).as_posix() for f in p.rglob('*') if f.is_file()}-{'MANIFEST.json'};assert files=={x['path'] for x in m['files']}
for x in m['files']:
 b=(p/x['path']).read_bytes();assert (len(b),h(b))==(x['bytes'],x['sha256'])
r=json.loads((p/'ATTESTATION.json').read_bytes());raw=(s/'STAGE-MANIFEST.json').read_bytes();assert h(raw)==r['stage_manifest_sha256']=='5dfe0bfe427cf9089f636ac8e52ae1a7e479e5ce0372e2fa8d884cd3f6c07e15';stage=json.loads(raw);actual={f.relative_to(s).as_posix() for f in s.rglob('*') if f.is_file()}-{'STAGE-MANIFEST.json'};assert actual=={x['path'] for x in stage['files']}
for x in stage['files']:
 b=(s/x['path']).read_bytes();assert (len(b),h(b))==(x['bytes'],x['sha256'])
scan=(p/'SCAN-RESULT.json').read_bytes();assert h(scan)==r['scan_result_sha256'] and json.loads(scan)==json.loads((s/'receipts/successor-privacy-scan.json').read_bytes());x=json.loads(scan);assert x['forbidden_digest_total']==0 and not x['exceptions'] and x['other_candidate_counts']=={'email':30}
assert h((s/'scan-private-digests.py').read_bytes())==r['scanner_sha256'];print('PASS: receipt and frozen stage file identities; scanner result bound to frozen manifest. No product execution or complete privacy claim.')
