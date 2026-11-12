#!/usr/bin/env python3
import argparse,pathlib,json,hashlib
P=pathlib.Path(__file__).resolve().parent
a=argparse.ArgumentParser();a.add_argument('packet',type=pathlib.Path);args=a.parse_args()
manifest=json.loads((P/'MANIFEST.json').read_text());files={f.name for f in P.iterdir() if f.is_file() and f.name!='MANIFEST.json'};assert files==set(manifest['files']) and all(not f.is_symlink() for f in P.iterdir())
for name,entry in manifest['files'].items():
 b=(P/name).read_bytes();assert len(b)==entry['bytes'] and hashlib.sha256(b).hexdigest()==entry['sha256']
anchor=json.loads((P/'PACKET-ANCHOR.json').read_text());b=(args.packet/'MANIFEST.json').read_bytes();assert len(b)==anchor['packet_manifest_bytes'] and hashlib.sha256(b).hexdigest()==anchor['packet_manifest_sha256']
for tag in ('STANDALONE','FULL','FINAL-PRIVATE','FINAL-PATTERN'):
 assert (P/f'{tag}.exit').read_text().strip()=='0' and not (P/f'{tag}.stderr').read_bytes()
private=json.loads((P/'FINAL-PRIVATE.stdout').read_text());pattern=json.loads((P/'FINAL-PATTERN.stdout').read_text());assert private['forbidden_digest_total']==0 and not private['other_candidate_counts'] and not private['exceptions'];assert not pattern['candidate_counts'] and not pattern['exceptions']
print('PASS scan receipt, packet manifest anchor, evidence-only replay exits and bounded privacy results.')
print('LIMITS: restricted-corpus input and Git/transport replay inputs are not independent public custody; this checks recorded scan results, not all secrets or product behavior.')
