#!/usr/bin/env python3
"""Verify the retained optional evidence replay and verifier evolution."""
import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
p=R/'delegated/postc7dc-optional-replay';raw=(p/'RECORD.json').read_bytes();assert H(raw)=='df9f36dc95185f89c7a1bac68dbc343c032c911ab48b7cca68996dda5787148d';r=json.loads(raw)
assert r['exit']==0 and H((R/'verify_postc7dc_followups.py').read_bytes())==r['verifier_sha256'];log=(p/'VERIFICATION.log').read_bytes();assert (len(log),H(log))==(r['transcript_bytes'],r['transcript_sha256'])
scan=(p/'digest.json').read_bytes();assert H(scan)=='7351193682843c9e010bee25f28df19f8f876263b726b4d6bf07ace5c256dfeb';s=json.loads(scan);assert s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions'];assert H((p/'pattern.log').read_bytes())=='e2e7ddf956ff69567faa4439cb7df72535c37d0576b39c0bf0f7a7a345366ac4'
e=R/'delegated/postc7dc-verifier-evolution';manifest_raw=(e/'MANIFEST.json').read_bytes();assert H(manifest_raw)=='0818bd7c7969ba817f658070ee78543255899d56228a0a61a5c1aa7ff2b546b3';manifest=json.loads(manifest_raw);names=set()
for row in manifest['files']:
 f=e/row['path'];assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256']);names.add(row['path'])
assert {f.name for f in e.iterdir() if f.is_file()}==names|{'MANIFEST.json'}
correction=json.loads((e/'EVOLUTION.json').read_bytes());assert H((e/'verify-initial.py.txt').read_bytes())==correction['initial_verifier_sha256'] and correction['successor_verifier_sha256']==r['verifier_sha256']
print('Verified the preserved failed evidence-checker version and optional same-machine replay receipt; external originals and product behavior remain separately qualified.')
