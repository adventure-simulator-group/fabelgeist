#!/usr/bin/env python3
"""Verify or restore encrypted audit evidence; never overwrite an existing path."""
import argparse
import gzip
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
ap = argparse.ArgumentParser()
ap.add_argument('--key', type=Path, help='separately held 32-byte AES key')
ap.add_argument('--restore-root', type=Path, help='new, nonexistent destination for recovered evidence')
args = ap.parse_args()
assert not args.restore_root or args.key, 'Restoration requires the separately held key'
manifest = json.loads((P / 'MANIFEST.json').read_bytes())
assert manifest['schema'] == '726-encrypted-private-evidence-v1'
rows = manifest['objects']
labels = set()
for row in rows:
    label = row['label']
    assert label not in labels and label and all(c.isalnum() or c in '-_' for c in label)
    labels.add(label)
    name = PurePosixPath(row['ciphertext'])
    assert len(name.parts) == 1 and name.name == label + '.aesgcm'
    p = P / name
    assert p.is_file() and not p.is_symlink()
    blob = p.read_bytes()
    assert (len(blob), H(blob)) == (row['ciphertext_bytes'], row['ciphertext_sha256'])
    assert row['associated_data'] == '726-audit-private-evidence:' + label + ':v1'
assert {x.name for x in P.iterdir()} == {r['ciphertext'] for r in rows} | {'MANIFEST.json', 'VERIFY.py', 'README.md'}
if not args.key:
    print('Verified encrypted payload identities; NOT REPEATED: decryption, plaintext archive checks and original restoration require the separately held key.')
    raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
key = args.key.read_bytes()
assert len(key) == 32
verified = []
for row in rows:
    blob = (P / row['ciphertext']).read_bytes()
    plain = AESGCM(key).decrypt(blob[:12], blob[12:], row['associated_data'].encode())
    assert (len(plain), H(plain)) == (row['plaintext_archive_bytes'], row['plaintext_archive_sha256'])
    seen = set()
    contents = []
    with tarfile.open(fileobj=io.BytesIO(plain), mode='r:gz') as archive:
        for member in archive.getmembers():
            name = PurePosixPath(member.name)
            assert not name.is_absolute() and name.parts and name.parts[0] == 'evidence' and '..' not in name.parts
            assert member.name not in seen and (member.isfile() or member.isdir())
            seen.add(member.name)
            body = archive.extractfile(member).read() if member.isfile() else None
            if body is not None:
                assert len(body) == member.size
            contents.append((name, member.mode, body))
    assert sum(body is not None for _, _, body in contents) == row['regular_files']
    assert sum(body is None for _, _, body in contents) == row['directories']
    verified.append((row['label'], contents))
if args.restore_root:
    root = args.restore_root
    assert not root.exists() and not root.is_symlink(), 'Destination must not already exist'
    root.mkdir(mode=0o700)
    for label, contents in verified:
        dest = root / label
        dest.mkdir(mode=0o700)
        for name, mode, body in contents:
            target = dest.joinpath(*name.parts)
            if body is None:
                target.mkdir(parents=True, exist_ok=True)
                target.chmod(mode)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with target.open('xb') as f:
                    f.write(body)
                target.chmod(mode)
    print('Restored authenticated evidence into a newly created destination; no existing file was overwritten.')
else:
    print('Authenticated and verified all plaintext archives in memory; no evidence was restored.')
print('Same-machine key custody, independent recovery, product behavior and audit closure remain separately qualified.')
