#!/usr/bin/env python3
"""Verify selected prior registration variants and historical source labels."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
R = Path(__file__).resolve().parent
load = lambda p: json.loads((R / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
cp = load('DELEGATED-EVIDENCE-CHECKPOINT-32.json')
bundles = {x['bundle_id']:x for x in cp['bundles']}
transport = json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()))
by_path = {x['path']:x for x in transport['entries']}
b = bundles['reflect-admission-2563-2564-reconcile']
members = {x['path']:x for x in b['members']}
prior = next(x for x in load('DELEGATED-EVIDENCE-CHECKPOINT-13.json')['bundles'] if x['bundle_id']=='bevy-reflect-derive-hair-ops')
oldmembers = {x['path']:x for x in prior['members']}
with tarfile.open(R/b['archive']) as t:
    hist = json.load(t.extractfile('history/attribution-and-copies.json'))
    proof = json.load(t.extractfile('git-proofs.json'))
    records = proof.get('paths',proof.get('proofs',[]))
    handoff = json.loads(gzip.decompress(t.extractfile('history/older-774-handoff-manifest.json.gz').read()))
    for x in hist['records']:
        m = members[x['copy']]
        assert (m['original_sha256'],m['original_bytes']) == (x['sha256'],x['bytes'])
        if x['scope']=='059':
            y=by_path[x['path']];assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob'])
        elif x['scope']=='774-rig-f82':
            assert any(p['path']==x['transport_path'] and p.get('sha256')==x['sha256'] for p in records)
        elif x['scope']=='774-segovb-handoff':
            assert any((z['path'],z['sha256'],z['bytes'])==(x['origin'],x['sha256'],x['bytes']) for z in handoff['files'])
    comp=json.load(t.extractfile('history/published-prior-comparison.json'))
    archived=(R/comp['published_archive_path']).read_bytes();assert sha(archived)==comp['published_archive_sha256']
    with tarfile.open(R/comp['published_archive_path']) as previous:
        for x in comp['comparison']:
            m=oldmembers[x['member']];assert sha(previous.extractfile(x['member']).read())==x['published_sha256']==m['published_sha256']
            assert x['original_sha256']==m['original_sha256'] and x['byte_identical']==(m['original_sha256']==m['published_sha256'])
    saved=members['history/prior-hair-registration.json'];old=oldmembers['registration.json'];assert saved['changed'] and old['changed'] and saved['original_sha256']==old['original_sha256'] and saved['published_sha256']==old['published_sha256'] and saved['original_sha256']!=saved['published_sha256']
    assert members['history/published-hair-evidence.tar.gz']['original_sha256']==comp['published_archive_sha256']
registry=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']
for i in (2578,2579):
    assert registry[i]['snapshot']=='pinned-native-dependency' and registry[i]['source_id'].startswith('physical-p16:')
b=bundles['native-p16-2577-2580-reconcile'];members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
    d=json.load(t.extractfile('historical/evidence-index.json'))
    for path,h in d['original_059_receipts'].items():
        m=members['historical/'+path];assert m['original_sha256']==by_path[path]['sha256']==h and m['original_bytes']==by_path[path]['bytes']
    assert 'pin mismatch' in d['qualification'] and 'not substituted' in d['qualification']
print('Verified selected 059/774 links, distinct prior registration bytes, and original mixed P16 snapshot labels; no historical credit transferred.')
