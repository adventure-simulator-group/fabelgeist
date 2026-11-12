#!/usr/bin/env python3
"""Verify evidence-scan custody and packet linkage; not product behavior."""
from pathlib import Path
import argparse,hashlib,json
P=Path(__file__).resolve().parent;p=argparse.ArgumentParser();p.add_argument('packet',type=Path);a=p.parse_args();m=json.loads((P/'MANIFEST.json').read_text());actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file() and x!=P/'MANIFEST.json'};assert actual==set(m['files'])
for name,v in m['files'].items():
 f=Path(name);assert not f.is_absolute() and '..' not in f.parts and not (P/f).is_symlink();b=(P/f).read_bytes();assert len(b)==v['bytes'] and hashlib.sha256(b).hexdigest()==v['sha256']
r=json.loads((P/'RECEIPT.json').read_text());assert hashlib.sha256((a.packet/'MANIFEST.json').read_bytes()).hexdigest()==r['packet_manifest_sha256']
for group in ['scanner_sha256','results_sha256']:
 for name,digest in r[group].items():assert hashlib.sha256((P/name).read_bytes()).hexdigest()==digest
private=json.loads((P/'FINAL-PRIVATE.json').read_text());pattern=json.loads((P/'FINAL-PATTERN.json').read_text());assert private['forbidden_digest_total']==0 and not private['other_candidate_counts'] and not private['exceptions'];assert not pattern['candidate_counts'] and not pattern['exceptions']
for name in ['FINAL-PRIVATE-EXIT.txt','FINAL-PATTERN-EXIT.txt','FULL-EXIT.txt','STANDALONE-EXIT.txt']:assert (P/name).read_text().strip()=='0'
print('PASS scan artifacts, hashes, packet-manifest binding and reported bounded results; see receipt for limits.')
