#!/usr/bin/env python3
"""Verify recovered historical P16 evidence, without applying the patch."""
import collections
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
recovery = load('RECOVERY-PHYSICAL-P16-PATCH-20261111.json')
comparison = load(recovery['live_comparison']['published_report'])
registry_bytes = gzip.decompress((root / recovery['historical_source']['registry']).read_bytes())
registry = json.loads(registry_bytes)
snapshot = next(s for s in registry['snapshots'] if s['id'] == 'physical-p16')
assert sha(registry_bytes) == recovery['historical_source']['registry_uncompressed_sha256']
assert snapshot['base'] == snapshot['head'] == comparison['head'] == comparison['original_base']
assert comparison['registry_sha256'] == sha(registry_bytes)
assert sha((root / recovery['live_comparison']['published_report']).read_bytes()) == recovery['live_comparison']['published_report_sha256']
assert 'Reconstructed' in recovery['live_comparison']['publication_qualification']
patch = (root / recovery['recovered_patch']['path']).read_bytes()
assert len(patch) == recovery['recovered_patch']['bytes'] == 524542
assert sha(patch) == recovery['recovered_patch']['sha256'] == snapshot['patch_sha256']
assert sum(line.startswith(b'diff --git ') for line in patch.splitlines()) == 152
assert sum(line.startswith(b'deleted file mode ') for line in patch.splitlines()) == 1
assert not any(line.startswith(b'new file mode ') for line in patch.splitlines())
rows = comparison['files']
assert len(rows) == snapshot['changed_variants'] == 159
assert [row['status'] + ' ' + row['path'] for row in rows] == snapshot['status_porcelain']
assert collections.Counter(row['status'] for row in rows) == {' M': 151, ' D': 1, '??': 7}
index = {}
for item in registry['files']:
    if item['source_id'].startswith('physical-p16:'):
        index.setdefault(item['path'], []).append(item)
with tarfile.open(root / recovery['untracked_and_other_source_objects']['archive'], 'r:gz') as archive:
    objects = {member.name: member for member in archive if member.isfile()}
    for row in rows:
        matches = index[row['path']]
        if row['exists']:
            digest = row['current_sha256']
            assert row['matches_registered'] and digest in row['original_registered_sha256']
            assert any(item['sha256'] == digest for item in matches)
            assert digest in objects and sha(archive.extractfile(objects[digest]).read()) == digest
        else:
            assert row['status'] == ' D' and row['matches_recorded_deletion']
            deleted = next(item for item in matches if item['state'] == 'deleted')
            base = archive.extractfile(objects[deleted['base_sha256']]).read()
            assert sha(base) == deleted['base_sha256']
            oid = hashlib.sha1(b'blob ' + str(len(base)).encode() + b'\0' + base).hexdigest()
            assert row['git_head_entry'].split()[2] == oid
assert load(recovery['earlier_search']['record'])['search']['matches'] == 0
inventory = recovery['later_agent_inventory']
assert sha((root / inventory['published_report']).read_bytes()) == inventory['published_sha256']
assert inventory['original_sha256'] == '085b229b77f9d4a1a46529658bb0835e4bd72ba659b094dad09689339365b57a'
assert inventory['redacted_unique_paths'] > 0
assert load('CONTINUATION-PROGRESS-95E.json')['physical_p16_patch_recovery'] == 'RECOVERY-PHYSICAL-P16-PATCH-20261111.json'
print('Verified historical P16 patch digest, 159 status rows, 158 archived path objects and deleted base; no patch application or product check.')
