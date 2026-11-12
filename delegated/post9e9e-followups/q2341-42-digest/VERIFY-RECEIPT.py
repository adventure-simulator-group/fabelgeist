#!/usr/bin/env python3
import pathlib,json,hashlib,argparse
P=pathlib.Path(__file__).resolve().parent;p=argparse.ArgumentParser();p.add_argument('packet',type=pathlib.Path);p.add_argument('scan_receipt',type=pathlib.Path);a=p.parse_args();m=json.loads((P/'MANIFEST.json').read_text());assert {x.name for x in P.iterdir() if x.is_file() and x.name!='MANIFEST.json'}==set(m['files']) and all(not x.is_symlink() for x in P.iterdir())
for name,x in m['files'].items():
 b=(P/name).read_bytes();assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256']
anchors=json.loads((P/'ANCHORS.json').read_text())
for key,root in [('source_packet',a.packet),('scan_receipt',a.scan_receipt)]:
 b=(root/'MANIFEST.json').read_bytes();assert len(b)==anchors[key]['manifest_bytes'] and hashlib.sha256(b).hexdigest()==anchors[key]['manifest_sha256']
for tag in ('SEAL-VERIFY','SEAL-PRIVATE','SEAL-PATTERN','SEAL-DIGEST'):
 assert (P/f'{tag}.exit').read_text().strip()=='0' and not (P/f'{tag}.stderr').read_bytes()
private=json.loads((P/'SEAL-PRIVATE.stdout').read_text());pattern=json.loads((P/'SEAL-PATTERN.stdout').read_text());assert private['forbidden_digest_total']==0 and not private['other_candidate_counts'] and not private['exceptions'] and not pattern['candidate_counts'] and not pattern['exceptions']
assert 'PASS private-input removed-value and directory-prefix digest search' in (P/'SEAL-DIGEST.stdout').read_text()
print('PASS scan receipt and packet anchors, bounded scan and private-input digest receipt; no independent custody claim.')
