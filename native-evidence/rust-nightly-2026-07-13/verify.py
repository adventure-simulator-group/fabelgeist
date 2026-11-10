"""Verify preserved evidence members; this is not product validation."""
import hashlib
import json
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
manifest = json.loads((root / 'MANIFEST.json').read_text())
for item in manifest['files']:
    relative = PurePosixPath(item['path'])
    if relative.is_absolute() or '..' in relative.parts or relative.as_posix() != item['path']:
        raise ValueError('unsafe evidence path')
    data = (root / relative).read_bytes()
    if len(data) != item['bytes'] or hashlib.sha256(data).hexdigest() != item['sha256']:
        raise ValueError('evidence mismatch: ' + item['path'])
print('verified', len(manifest['files']), 'preserved archive members')
