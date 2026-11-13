#!/usr/bin/env python3
"""Replay exact source-only checkpoint transformation and artifact identities."""
import hashlib
import io
import json
import stat
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError('Unoptimized interpreter required')
from TRANSFORM import NAMES, transform
p = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
r = json.loads((p / 'RECORD.json').read_bytes())
assert r['schema'] == '726-postd7a5-application-v2'
assert sha((p / 'TRANSFORM.py').read_bytes()) == r['transform_sha256']
raw = (p / r['archive']).read_bytes()
assert len(raw) == r['archive_bytes'] and sha(raw) == r['archive_sha256']
expected = {x['path']: x for x in r['members']}
assert len(expected) == len(r['members'])
data = {}
with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
    for item in archive:
        name = PurePosixPath(item.name)
        assert not name.is_absolute() and '..' not in name.parts
        assert str(name) in expected and str(name) not in data and item.isfile()
        row = expected[str(name)]
        assert stat.S_IMODE(item.mode) == int(row['mode'], 8)
        body = archive.extractfile(item).read()
        assert len(body) == row['bytes'] and sha(body) == row['sha256']
        data[str(name)] = body
assert set(data) == set(expected)
before = {n: data['before/' + n] for n in NAMES}
after = transform(before, json.loads(data['inputs/report.json']),
                  json.loads(data['inputs/pending.json']),
                  json.loads(data['inputs/main-observation.json']))
assert all(after[n] == data['after/' + n] for n in NAMES)
print('Verified exact before/after transformation: six partial records, two '
      'unassessed context records, six relationships, navigation successor and no semantic closure.')
