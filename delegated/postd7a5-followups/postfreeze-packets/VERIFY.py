#!/usr/bin/env python3
"""Verify exact pending source-evidence transports and their member bytes."""
import hashlib
import io
import json
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError('Evidence verification requires assertions')
p = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
m = json.loads((p / 'MANIFEST.json').read_bytes())
assert m['schema'] == '726-postd7a5-postfreeze-public-transports-v1'
assert m['source_revision'] == '95e63628f9d70b8dd6e6fede64334696c6995009'
assert m['parent_checkpoint'] == 'd7a5e77ebdbd44a05cde30881493d6fc22ef025d'
names = {'MANIFEST.json', 'VERIFY.py'}
count = 0
for row in m['bundles']:
    name = row['file']
    assert name == row['label'] + '.tar.gz' and name not in names
    names.add(name)
    raw = (p / name).read_bytes()
    assert (len(raw), sha(raw)) == (row['archive_bytes'], row['archive_sha256'])
    wanted = {x['path']: x for x in row['members']}
    assert len(wanted) == len(row['members'])
    seen = set()
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
        for member in archive:
            name = PurePosixPath(member.name)
            assert not name.is_absolute() and '..' not in name.parts
            assert member.isfile() and member.name in wanted and member.name not in seen
            seen.add(member.name)
            item = wanted[member.name]
            body = archive.extractfile(member).read()
            assert (len(body), sha(body), oct(member.mode)) == (item['bytes'], item['sha256'], item['mode'])
    assert seen == set(wanted)
    count += len(seen)
assert {x.name for x in p.iterdir()} == names
print(f'Verified {len(m["bundles"])} post-freeze transports and {count} exact members; '
      'restricted originals and central reconciliation remain separate.')
