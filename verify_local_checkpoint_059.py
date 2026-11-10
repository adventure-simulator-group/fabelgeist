"""Compare a supplied evidence directory to the pinned 059 transport, read-only."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath

parser = argparse.ArgumentParser()
parser.add_argument('evidence_root', type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parent
manifest_bytes = gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes())
if hashlib.sha256(manifest_bytes).hexdigest() != 'c3e04095a9e4cb3e2da6ee2a4f561c811ebeca445a50461ddb63b2d426ed6858':
    raise ValueError('transport manifest mismatch')
manifest = json.loads(manifest_bytes)
counts = {'matched': 0, 'missing': 0, 'mismatch': 0, 'unsafe': 0}
for entry in manifest['entries']:
    relative = PurePosixPath(entry['path'])
    if relative.is_absolute() or '..' in relative.parts or relative.as_posix() != entry['path']:
        counts['unsafe'] += 1
        continue
    file = args.evidence_root.joinpath(*relative.parts)
    if any(parent.is_symlink() for parent in [file, *file.parents] if parent != args.evidence_root.parent):
        counts['unsafe'] += 1
        continue
    if not file.is_file():
        counts['missing'] += 1
        continue
    digest = hashlib.sha256()
    size = 0
    with file.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            size += len(chunk)
            digest.update(chunk)
    counts['matched' if size == entry['bytes'] and digest.hexdigest() == entry['sha256'] else 'mismatch'] += 1
print(json.dumps({'expected': manifest['file_count'], 'counts': counts, 'qualification': 'Historical transport byte verification only; no source-read credit or later authority.'}, indent=2))
if counts['matched'] != manifest['file_count'] or any(counts[k] for k in ('missing', 'mismatch', 'unsafe')):
    raise SystemExit(1)
