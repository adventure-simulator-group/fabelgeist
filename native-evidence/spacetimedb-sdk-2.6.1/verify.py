"""Verify a preserved source archive and selected members; not a product test."""
import hashlib
import json
import tarfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
manifest = json.loads((root / 'MANIFEST.json').read_text())
archive = root / manifest['archive']
raw = archive.read_bytes()
if len(raw) != manifest['bytes'] or hashlib.sha256(raw).hexdigest() != manifest['sha256']:
    raise ValueError('archive mismatch')
with tarfile.open(archive, 'r:gz') as package:
    for entry in manifest['inspected_members']:
        name = PurePosixPath(entry['path'])
        if name.is_absolute() or '..' in name.parts or name.as_posix() != entry['path']:
            raise ValueError('unsafe member path')
        member = package.extractfile(name.as_posix())
        if member is None:
            raise ValueError('missing archive member')
        data = member.read()
        if len(data) != entry['bytes'] or hashlib.sha256(data).hexdigest() != entry['sha256']:
            raise ValueError('archive member mismatch: ' + entry['path'])
print('verified archive and', len(manifest['inspected_members']), 'source or license members')
