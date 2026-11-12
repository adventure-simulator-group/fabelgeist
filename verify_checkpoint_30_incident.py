#!/usr/bin/env python3
"""Bind original and altered scratch-verifier outputs as an additive record."""
import hashlib
import json
import tarfile
from pathlib import Path
R = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
cp = json.loads((R / 'DELEGATED-EVIDENCE-CHECKPOINT-30.json').read_text())
b = next(x for x in cp['bundles'] if x['bundle_id'] == 'mixed-quality-2434-2452-2470-2475')
rows = {x['path']: x for x in b['members']}
with tarfile.open(R / b['archive']) as t:
    record = json.load(t.extractfile('incident.json'))
    assert len(record['entries']) == 2 and 'not restored in place' in record['qualification']
    for e, prior, manifest in zip(record['entries'],
            ['delegated/p17-metrics-evidence.tar.gz', 'delegated/rig-quality-export-hair-evidence.tar.gz'],
            ['BUNDLE-MANIFEST.json', 'packet-manifest.json']):
        assert sha((R / prior).read_bytes()) == e['published_archive_sha256']
        original = t.extractfile(e['original_copy']).read()
        altered = t.extractfile(e['altered_copy']).read()
        assert (sha(original), len(original)) == (e['old_recorded_sha256'], e['old_recorded_bytes'])
        assert (sha(altered), len(altered)) == (e['new_sha256'], e['new_bytes'])
        assert sha(original) != sha(altered) and not rows[e['original_copy']]['changed'] and not rows[e['altered_copy']]['changed']
        with tarfile.open(R / prior) as old:
            assert old.extractfile('evidence-verification.json').read() == original
            frozen = json.load(old.extractfile(manifest))
            row = next(x for x in frozen['files'] if x['path'] == 'evidence-verification.json')
            assert (row['sha256'], row['bytes']) == (sha(original), len(original))
assert rows['incident.json']['changed'] and rows['incident.json']['original_sha256'] == '96526bcb5762ce79c640b24938a36e13a3485eb5f4c0a201433a50c74252199a'
print('Verified two altered scratch outputs and exact original copies against earlier published archive manifests; no in-place restoration claimed.')
