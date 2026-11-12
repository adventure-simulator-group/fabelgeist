#!/usr/bin/env python3
import argparse,pathlib,json,hashlib
P=pathlib.Path(__file__).resolve().parent
a=argparse.ArgumentParser();a.add_argument('packet',type=pathlib.Path);args=a.parse_args()
m=json.loads((P/'MANIFEST.json').read_text());files={x.name for x in P.iterdir() if x.is_file() and x.name!='MANIFEST.json'};assert files==set(m['files']) and all(not x.is_symlink() for x in P.iterdir())
for name,x in m['files'].items():
 b=(P/name).read_bytes();assert len(b)==x['bytes'] and hashlib.sha256(b).hexdigest()==x['sha256']
a=json.loads((P/'PACKET-ANCHOR.json').read_text());b=(args.packet/'MANIFEST.json').read_bytes();assert len(b)==a['packet_manifest_bytes'] and hashlib.sha256(b).hexdigest()==a['packet_manifest_sha256']
for tag in ('STANDALONE','FULL','FINAL-PRIVATE','FINAL-PATTERN'):
 assert (P/f'{tag}.exit').read_text().strip()=='0' and not (P/f'{tag}.stderr').read_bytes()
private=json.loads((P/'FINAL-PRIVATE.stdout').read_text());pattern=json.loads((P/'FINAL-PATTERN.stdout').read_text());assert private['forbidden_digest_total']==0 and not private['other_candidate_counts'] and not private['exceptions'];assert not pattern['candidate_counts'] and not pattern['exceptions']
print('PASS packet anchor, replay exits and bounded privacy results; no product execution.')
print('LIMITS: restricted key/corpus and Git/transport inputs are separate; no independent custody or universal privacy guarantee.')
