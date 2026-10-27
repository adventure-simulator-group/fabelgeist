"""Ignored exact frozen-source retrieval, hashing and bounded reader only."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).parent
registry = Path('target/recovery-20261025/shared-material-successor/source-extensions-175.json')
ref = 'origin/codex/774-audit-handoff-20261025'
prefix = 'handoff/774/audit/reconcile-adler-20261023/'
if sys.argv[1] == 'resolve':
    (root / 'sources').mkdir(exist_ok=True)
    entries = json.loads(registry.read_text())['entries']
    tree = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', ref, 'handoff/774']).decode().splitlines()
    resolved = []
    for index in range(62, 175):
        entry = entries[index]
        sha = entry['sha256']
        local = root / 'sources' / sha
        candidates = [Path(entry['object']),
                      Path('target/recovery-20261025/audit03-delta/source-objects') / sha,
                      Path('target/recovery-20261025/audit04-delta/sources') / sha]
        data, origin = None, None
        for candidate in candidates:
            if candidate.is_file():
                value = candidate.read_bytes()
                if hashlib.sha256(value).hexdigest() == sha:
                    data, origin = value, str(candidate)
                    break
        if data is None:
            suffix = entry['object'].partition('/reconcile-adler-20261023/')[2]
            paths = ([prefix + suffix] if suffix and prefix + suffix in tree else [])
            paths += [path for path in tree if path.endswith('/' + sha) or path.endswith('/registered-' + sha)]
            for path in dict.fromkeys(paths):
                value = subprocess.check_output(['git', 'show', ref + ':' + path])
                if hashlib.sha256(value).hexdigest() == sha:
                    data, origin = value, ref + ':' + path
                    break
        if data is None:
            print('UNRESOLVED', index, entry['source_id'])
            continue
        local.write_bytes(data)
        resolved.append(dict(index=index, source_id=entry['source_id'], sha256=sha,
                             ranges=entry['ranges'], revision=entry.get('revision'),
                             local_object=str(local), recovery=origin,
                             line_count=len(data.decode().splitlines())))
    (root / 'resolved-source-map.json').write_text(json.dumps(resolved, indent=2) + '\n')
    print('Exact registered source objects resolved', len(resolved), 'of113; no semantic read credit')
elif sys.argv[1] == 'read':
    low, high = map(int, sys.argv[2:4])
    for source in json.loads((root / 'resolved-source-map.json').read_text()):
        if low <= source['index'] <= high:
            lines = Path(source['local_object']).read_text().splitlines()
            print('MATERIAL', source['index'], source['source_id'])
            for first, last in source['ranges']:
                for index in range(first - 1, min(last, len(lines))):
                    print(f'{index + 1}: {lines[index]}')
