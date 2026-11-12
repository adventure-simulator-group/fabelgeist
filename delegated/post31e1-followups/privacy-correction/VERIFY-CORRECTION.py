#!/usr/bin/env python3
"""Bind the path-safe correction to its optional authenticated originals."""
import argparse
import hashlib
import io
import json
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P = Path(__file__).resolve().parent
H = lambda b: hashlib.sha256(b).hexdigest()
record = json.loads((P / 'RECORD.json').read_bytes())
manifest_bytes = (P / 'encrypted-originals/MANIFEST.json').read_bytes()
assert H(manifest_bytes) == record['encrypted_original_manifest_sha256']
manifest = json.loads(manifest_bytes)
assert len(manifest['objects']) == 1
row = manifest['objects'][0]
blob = (P / 'encrypted-originals' / row['ciphertext']).read_bytes()
assert (len(blob), H(blob)) == (row['ciphertext_bytes'], row['ciphertext_sha256'])
excluded = record['excluded_original_files']
assert len(excluded) == record['excluded_file_count'] == 15
for x in excluded:
    path = PurePosixPath(x['path'])
    assert not path.is_absolute() and '..' not in path.parts and not (P.parent / path).exists()
ap = argparse.ArgumentParser()
ap.add_argument('--key', type=Path, help='separately held 32-byte private key')
args = ap.parse_args()
if not args.key:
    print('Verified omission list and ciphertext identity; NOT REPEATED: original contents require the separately held key.')
    raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
key = args.key.read_bytes()
assert len(key) == 32
plain = AESGCM(key).decrypt(blob[:12], blob[12:], row['associated_data'].encode())
assert (len(plain), H(plain)) == (row['plaintext_archive_bytes'], row['plaintext_archive_sha256'])
with tarfile.open(fileobj=io.BytesIO(plain), mode='r:gz') as tar:
    members = {m.name: m for m in tar.getmembers() if m.isfile()}
    expected = {'evidence/excluded/' + x['path'] for x in excluded}
    assert {name for name in members if name.startswith('evidence/excluded/')} == expected
    for x in excluded:
        data = tar.extractfile(members['evidence/excluded/' + x['path']]).read()
        assert (len(data), H(data)) == (x['bytes'], x['whole_file_sha256'])
    digest = tar.extractfile(members['evidence/initial-digest/OUTPUT.json']).read()
    pattern = tar.extractfile(members['evidence/initial-pattern/OUTPUT.log']).read()
    assert H(digest) == record['initial_digest_scan_sha256']
    assert H(pattern) == record['initial_pattern_scan_sha256']
print('Authenticated and bound every omitted original and both exact initial scan outputs in memory.')
print('Same-machine key custody, independent recovery and product behavior remain separately qualified.')
