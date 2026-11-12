#!/usr/bin/env python3
"""Resolve this checkpoint's source packet links without transferring read credit."""
import glob
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda n: json.loads((root / n).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
checkpoints = []
for p in sorted(root.glob('DELEGATED-EVIDENCE-CHECKPOINT*.json')):
    d = json.loads(p.read_text())
    if isinstance(d.get('bundles'), list):
        checkpoints.append(d)
manifest_to_bundle = {}
for cp in checkpoints:
    for b in cp['bundles']:
        for m in b['members']:
            if m['path'] in ('MANIFEST.json', 'BUNDLE-MANIFEST.json', 'packet-manifest.json', 'manifest.json'):
                manifest_to_bundle[m['original_sha256']] = b
current = load('DELEGATED-EVIDENCE-CHECKPOINT-13.json')
counts = {'prior_packet_links': 0, 'copied_original_identities': 0,
          'prior_member_links': 0, 'published_file_links': 0}
for b in current['bundles']:
    members = {m['path']: m for m in b['members']}
    with tarfile.open(root / b['archive'], 'r:gz') as t:
        names = set(t.getnames())
        if 'published-references.json' in names:
            d = json.load(t.extractfile('published-references.json'))
            for entry in d['entries']:
                if 'preserved_bytes' in entry:
                    m = members[entry['preserved_bytes']]
                    assert m['original_sha256'] == entry['sha256']
                    assert m['original_bytes'] == entry['bytes']
                    counts['copied_original_identities'] += 1
                    continue
                p = (root / entry.get('repository_relative_path', entry.get('path'))).resolve()
                assert p.is_relative_to(root.resolve()) and p.is_file()
                raw = p.read_bytes()
                assert len(raw) == entry['bytes'] and sha(raw) == entry['sha256']
                counts['published_file_links'] += 1
        if 'prior-references.json' in names:
            d = json.load(t.extractfile('prior-references.json'))
            if 'manifest_sha256' in d:
                h = d['manifest_sha256']
                assert h in manifest_to_bundle
                assert members[d['manifest_copy']]['original_sha256'] == h
                prior = {m['path']: m for m in manifest_to_bundle[h]['members']}
                for ref in d['references']:
                    m = prior[ref['prior_manifest_path']]
                    assert m['original_sha256'] == ref['sha256'] and m['original_bytes'] == ref['bytes']
                    counts['prior_member_links'] += 1
                counts['prior_packet_links'] += 1
            else:
                for ref in d['references']:
                    h = ref['manifest_sha256']
                    assert h in manifest_to_bundle
                    manifest = ref.get('copied_manifest', ref.get('manifest'))
                    report = ref.get('copied_report', ref.get('report'))
                    assert members[manifest]['original_sha256'] == h
                    assert members[report]['original_sha256'] == ref['report_sha256']
                    counts['prior_packet_links'] += 1
                    counts['copied_original_identities'] += 2
        if 'later-hair-ops-reference.json' in names:
            d = json.load(t.extractfile('later-hair-ops-reference.json'))
            recovery = load('RECOVERY-PHYSICAL-P16-PATCH-20261111.json')
            expected = {recovery['later_agent_inventory']['original_sha256'],
                        recovery['live_comparison']['original_report_sha256']}
            assert {f['sha256'] for f in d['files']} == expected
            counts['copied_original_identities'] += len(expected)
assert counts == {'prior_packet_links': 7, 'copied_original_identities': 25,
                  'prior_member_links': 8, 'published_file_links': 21}, counts
print('Verified qualified cross-packet and previously published file links:', counts,
      '; original hashes do not imply unredacted published bytes or read credit.')
